use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use futures_util::future::{join_all, BoxFuture};
use futures_util::FutureExt;
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use protox::file::{FileResolver, GoogleFileResolver};

use crate::core::config::KeyValue;
use crate::core::proto_reader::fetch::GrpcReflection;
use crate::core::resource_reader::{Cached, ResourceReader};
use crate::core::runtime::TargetRuntime;

#[derive(Clone)]
pub struct ProtoReader {
    reader: ResourceReader<Cached>,
    runtime: TargetRuntime,
}

#[derive(Clone)]
pub struct ProtoMetadata {
    pub descriptor_set: FileDescriptorSet,
    pub path: String,
}

impl ProtoReader {
    /// Initializes the proto reader with a resource reader and target runtime
    pub fn init(reader: ResourceReader<Cached>, runtime: TargetRuntime) -> Self {
        Self { reader, runtime }
    }

    /// Fetches proto files from a grpc server (grpc reflection)
    pub async fn fetch<T: AsRef<str>>(
        &self,
        url: T,
        headers: Option<Vec<KeyValue>>,
    ) -> anyhow::Result<Vec<ProtoMetadata>> {
        let grpc_reflection = Arc::new(GrpcReflection::new(
            url.as_ref(),
            headers,
            self.runtime.clone(),
        ));

        let mut proto_metadata = vec![];
        let service_list = grpc_reflection.list_all_files().await?;
        for service in service_list {
            if service.eq("grpc.reflection.v1alpha.ServerReflection") {
                continue;
            }
            let file_descriptor_proto = grpc_reflection.get_by_service(&service).await?;
            Self::check_package(&file_descriptor_proto)?;
            let descriptors = self
                .reflection_resolve(grpc_reflection.clone(), file_descriptor_proto)
                .await?;
            let metadata = ProtoMetadata {
                descriptor_set: FileDescriptorSet { file: descriptors },
                path: url.as_ref().to_string(),
            };
            proto_metadata.push(metadata);
        }
        Ok(proto_metadata)
    }

    /// Asynchronously reads all proto files from a list of paths
    pub async fn read_all<T: AsRef<str>>(&self, paths: &[T]) -> anyhow::Result<Vec<ProtoMetadata>> {
        let resolved_protos = join_all(paths.iter().map(|v| self.read(v.as_ref(), None)))
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(resolved_protos)
    }

    /// Reads a proto file from a path
    pub async fn read<T: AsRef<str>>(
        &self,
        path: T,
        proto_paths: Option<&[String]>,
    ) -> anyhow::Result<ProtoMetadata> {
        let file_read = self.read_proto(path.as_ref(), None, None).await?;
        Self::check_package(&file_read)?;

        let descriptors = self
            .file_resolve(
                file_read,
                PathBuf::from(path.as_ref()).parent(),
                proto_paths,
            )
            .await?;
        let metadata = ProtoMetadata {
            descriptor_set: FileDescriptorSet { file: descriptors },
            path: path.as_ref().to_string(),
        };
        Ok(metadata)
    }

    /// Used as a helper file to resolve dependencies proto files
    async fn resolve_dependencies<F>(
        &self,
        parent_proto: FileDescriptorProto,
        resolve_fn: F,
    ) -> anyhow::Result<Vec<FileDescriptorProto>>
    where
        F: Fn(&str) -> BoxFuture<'_, anyhow::Result<FileDescriptorProto>>,
    {
        let mut descriptors: HashMap<String, FileDescriptorProto> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(parent_proto.clone());

        while let Some(file) = queue.pop_front() {
            let futures: Vec<_> = file
                .dependency
                .iter()
                .map(|import| resolve_fn(import))
                .collect();

            let results = join_all(futures).await;

            for result in results {
                let proto = result?;
                if !descriptors.contains_key(proto.name()) {
                    queue.push_back(proto.clone());
                    descriptors.insert(proto.name().to_string(), proto);
                }
            }
        }

        let mut descriptors_vec = descriptors
            .into_values()
            .collect::<Vec<FileDescriptorProto>>();
        descriptors_vec.push(parent_proto);
        Ok(descriptors_vec)
    }

    /// Used to resolve dependencies proto files using file reader
    async fn file_resolve(
        &self,
        parent_proto: FileDescriptorProto,
        parent_path: Option<&Path>,
        proto_paths: Option<&[String]>,
    ) -> anyhow::Result<Vec<FileDescriptorProto>> {
        self.resolve_dependencies(parent_proto, |import| {
            let parent_path = parent_path.map(|p| p.to_path_buf());
            let this = self.clone();
            let proto_paths = proto_paths.map(|paths| {
                paths
                    .iter()
                    .map(|p| Path::new(p).to_path_buf())
                    .collect::<Vec<_>>()
            });
            async move {
                this.read_proto(import, parent_path.as_deref(), proto_paths.as_deref())
                    .await
            }
            .boxed()
        })
        .await
    }

    /// Used to resolve dependencies proto files using reflection
    async fn reflection_resolve(
        &self,
        grpc_reflection: Arc<GrpcReflection>,
        parent_proto: FileDescriptorProto,
    ) -> anyhow::Result<Vec<FileDescriptorProto>> {
        self.resolve_dependencies(parent_proto, |file| {
            let grpc_reflection = Arc::clone(&grpc_reflection);
            async move { grpc_reflection.get_file(file).await }.boxed()
        })
        .await
    }

    /// Tries to load well-known google proto files and if not found uses normal
    /// file and http IO to resolve them
    async fn read_proto<T: AsRef<str>>(
        &self,
        path: T,
        parent_dir: Option<&Path>,
        proto_paths: Option<&[PathBuf]>,
    ) -> anyhow::Result<FileDescriptorProto> {
        let content = if let Ok(file) = GoogleFileResolver::new().open_file(path.as_ref()) {
            file.source()
                .context("Unable to extract content of google well-known proto file")?
                .to_string()
        } else {
            let path = Self::resolve_path(path.as_ref(), parent_dir, proto_paths);
            self.reader.read_file(path).await?.content
        };
        Ok(protox_parse::parse(path.as_ref(), &content)?)
    }
    /// Checks if path is absolute else it joins file path with relative dir
    /// path
    fn resolve_path(src: &str, root_dir: Option<&Path>, proto_paths: Option<&[PathBuf]>) -> String {
        if src.starts_with("http") {
            return src.to_string();
        }

        if Path::new(&src).is_absolute() {
            return src.to_string();
        }

        if let Some(proto_paths) = proto_paths {
            for proto_path in proto_paths {
                let path = proto_path.join(src);
                if path.exists() {
                    return path.to_string_lossy().to_string();
                }
            }
        }

        if let Some(path) = root_dir {
            path.join(src).to_string_lossy().to_string()
        } else {
            src.to_string()
        }
    }
    fn check_package(proto: &FileDescriptorProto) -> anyhow::Result<()> {
        if proto.package.is_none() {
            anyhow::bail!("Package name is required");
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_proto_config {
    use std::path::{Path, PathBuf};

    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use tailcall_fixtures::protobuf;

    use crate::core::proto_reader::ProtoReader;
    use crate::core::resource_reader::{Cached, ResourceReader};

    #[tokio::test]
    async fn test_resolve() {
        // Skipping IO tests as they are covered in reader.rs
        let runtime = crate::core::runtime::test::init(None);
        let reader = ProtoReader::init(ResourceReader::<Cached>::cached(runtime.clone()), runtime);
        reader
            .read_proto("google/protobuf/empty.proto", None, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_nested_imports() -> Result<()> {
        let test_dir = Path::new(protobuf::SELF);
        let test_file = protobuf::NESTED_0;

        let runtime = crate::core::runtime::test::init(None);
        let file_rt = runtime.file.clone();

        let reader = ProtoReader::init(ResourceReader::<Cached>::cached(runtime.clone()), runtime);
        let file_descriptors = reader
            .file_resolve(
                reader.read_proto(&test_file, None, None).await?,
                Some(test_dir),
                None,
            )
            .await?;
        for file in file_descriptors
            .iter()
            .filter(|desc| !desc.name.as_ref().unwrap().starts_with("google/protobuf/"))
        {
            let path = test_dir.join(file.name.as_ref().unwrap());
            let path = path.to_string_lossy();
            let source = file_rt.read(&path).await?;
            let expected = protox_parse::parse(&path, &source)?;

            assert_eq!(&expected.dependency, &file.dependency, "for file {path}");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_proto_no_pkg() -> Result<()> {
        let runtime = crate::core::runtime::test::init(None);
        let reader = ProtoReader::init(ResourceReader::<Cached>::cached(runtime.clone()), runtime);
        let proto_no_pkg =
            PathBuf::from(tailcall_fixtures::configs::SELF).join("proto_no_pkg.graphql");
        let config_module = reader.read(proto_no_pkg.to_str().unwrap(), None).await;
        assert!(config_module.is_err());
        Ok(())
    }
}
