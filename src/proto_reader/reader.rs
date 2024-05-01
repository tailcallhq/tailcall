use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

use anyhow::Context;
use futures_util::future::join_all;
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use protox::file::{FileResolver, GoogleFileResolver};

use crate::proto_reader::fetch::GrpcReflection;
use crate::resource_reader::{Cached, ResourceReader};
use crate::runtime::TargetRuntime;

pub struct ProtoReader {
    reader: ResourceReader<Cached>,
    runtime: TargetRuntime,
}

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
    pub async fn fetch<T: AsRef<str>>(&self, url: T) -> anyhow::Result<Vec<ProtoMetadata>> {
        let grpc_reflection = GrpcReflection::new(url.as_ref(), self.runtime.clone());

        let mut proto_metadata = vec![];
        let service_list = grpc_reflection.list_all_files().await?;
        for service in service_list {
            if service.eq("grpc.reflection.v1alpha.ServerReflection") {
                continue;
            }
            let file_descriptor_proto = grpc_reflection.get_by_service(&service).await?;
            Self::check_package(&file_descriptor_proto)?;
            let descriptors = self.resolve(file_descriptor_proto, None).await?;
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
        let resolved_protos = join_all(paths.iter().map(|v| self.read(v.as_ref())))
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(resolved_protos)
    }

    /// Reads a proto file from a path
    pub async fn read<T: AsRef<str>>(&self, path: T) -> anyhow::Result<ProtoMetadata> {
        let file_read = self.read_proto(path.as_ref(), None).await?;
        Self::check_package(&file_read)?;

        let descriptors = self
            .resolve(file_read, PathBuf::from(path.as_ref()).parent())
            .await?;
        let metadata = ProtoMetadata {
            descriptor_set: FileDescriptorSet { file: descriptors },
            path: path.as_ref().to_string(),
        };
        Ok(metadata)
    }

    /// Performs BFS to import all nested proto files
    async fn resolve(
        &self,
        parent_proto: FileDescriptorProto,
        parent_path: Option<&Path>,
    ) -> anyhow::Result<Vec<FileDescriptorProto>> {
        let mut descriptors: HashMap<String, FileDescriptorProto> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(parent_proto.clone());

        while let Some(file) = queue.pop_front() {
            let futures: Vec<_> = file
                .dependency
                .iter()
                .map(|import| self.read_proto(import, parent_path))
                .collect();

            let results = join_all(futures).await;

            for result in results {
                let proto = result?;
                if descriptors.get(proto.name()).is_none() {
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

    /// Tries to load well-known google proto files and if not found uses normal
    /// file and http IO to resolve them
    async fn read_proto<T: AsRef<str>>(
        &self,
        path: T,
        parent_dir: Option<&Path>,
    ) -> anyhow::Result<FileDescriptorProto> {
        let content = if let Ok(file) = GoogleFileResolver::new().open_file(path.as_ref()) {
            file.source()
                .context("Unable to extract content of google well-known proto file")?
                .to_string()
        } else {
            let path = Self::resolve_path(path.as_ref(), parent_dir);
            self.reader.read_file(path).await?.content
        };
        Ok(protox_parse::parse(path.as_ref(), &content)?)
    }
    /// Checks if path is absolute else it joins file path with relative dir
    /// path
    fn resolve_path(src: &str, root_dir: Option<&Path>) -> String {
        if src.starts_with("http") {
            return src.to_string();
        }

        if Path::new(&src).is_absolute() {
            src.to_string()
        } else if let Some(path) = root_dir {
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

    use crate::proto_reader::ProtoReader;
    use crate::resource_reader::{Cached, ResourceReader};

    #[tokio::test]
    async fn test_resolve() {
        // Skipping IO tests as they are covered in reader.rs
        let runtime = crate::runtime::test::init(None);
        let reader = ProtoReader::init(ResourceReader::<Cached>::cached(runtime.clone()), runtime);
        reader
            .read_proto("google/protobuf/empty.proto", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_nested_imports() -> Result<()> {
        let test_dir = Path::new(protobuf::SELF);
        let test_file = protobuf::NESTED_0;

        let runtime = crate::runtime::test::init(None);
        let file_rt = runtime.file.clone();

        let reader = ProtoReader::init(ResourceReader::<Cached>::cached(runtime.clone()), runtime);
        let helper_map = reader
            .resolve(reader.read_proto(&test_file, None).await?, Some(test_dir))
            .await?;
        let files = test_dir.read_dir()?;
        for file in files {
            let path = file?.path();
            let path = path.to_string_lossy();
            let source = file_rt.read(&path).await?;
            let expected = protox_parse::parse(&path, &source)?;
            let actual = helper_map
                .iter()
                .find(|v| v.package.eq(&expected.package))
                .unwrap();

            assert_eq!(&expected.dependency, &actual.dependency);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_proto_no_pkg() -> Result<()> {
        let runtime = crate::runtime::test::init(None);
        let reader = ProtoReader::init(ResourceReader::<Cached>::cached(runtime.clone()), runtime);
        let mut proto_no_pkg = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        proto_no_pkg.push("src/grpc/tests/proto_no_pkg.graphql");
        let config_module = reader.read(proto_no_pkg.to_str().unwrap()).await;
        assert!(config_module.is_err());
        Ok(())
    }
}
