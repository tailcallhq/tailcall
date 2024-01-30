use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use anyhow::{Context, Result};
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use protox::file::{FileResolver, GoogleFileResolver};
use url::Url;

use crate::config::{Config, ConfigSet, ExprBody, Extensions};
use crate::{FileIO, HttpIO};

const NULL_STR: &str = "\0\0\0\0\0\0\0";

pub struct ConfigSetResolver {
    file_io: Arc<dyn FileIO>,
    http_io: Arc<dyn HttpIO>,
}

impl ConfigSetResolver {
    pub fn init(file_io: Arc<dyn FileIO>, http_io: Arc<dyn HttpIO>) -> Self {
        Self { file_io, http_io }
    }

    pub async fn make(&self, config: Config) -> Result<ConfigSet> {
        let mut descriptors: HashMap<String, FileDescriptorProto> = HashMap::new();
        let mut grpc_file_descriptor = FileDescriptorSet::default();
        for (_, typ) in config.types.iter() {
            for (_, fld) in typ.fields.iter() {
                let proto_path = if let Some(grpc) = &fld.grpc {
                    &grpc.proto_path
                } else if let Some(ExprBody::Grpc(grpc)) = fld.expr.as_ref().map(|e| &e.body) {
                    &grpc.proto_path
                } else {
                    NULL_STR
                };

                if proto_path != NULL_STR {
                    self.import_all(&mut descriptors, proto_path.to_string())
                        .await?;
                }
            }
        }
        for (_, v) in descriptors {
            grpc_file_descriptor.file.push(v);
        }
        let mut config_set = ConfigSet::from(config);
        let extensions = Extensions { grpc_file_descriptor, ..Default::default() };
        config_set.extensions = extensions;

        Ok(config_set)
    }

    // Performs simple BFS to include all the imported files in FileDescriptorSet
    async fn import_all(
        &self,
        descriptors: &mut HashMap<String, FileDescriptorProto>,
        proto_path: String,
    ) -> Result<()> {
        let source = self.resolve(&proto_path).await?;

        let mut queue = VecDeque::new();
        let parent_proto = protox_parse::parse(&proto_path, &source)?;
        queue.push_back(parent_proto.clone());

        while let Some(file) = queue.pop_front() {
            for import in file.dependency.iter() {
                let source = self.resolve(import).await?;
                if descriptors.get(import).is_some() {
                    continue;
                }
                let fdp = protox_parse::parse(import, &source)?;
                queue.push_back(fdp.clone());
                descriptors.insert(import.clone(), fdp);
            }
        }

        descriptors.insert(proto_path, parent_proto);

        Ok(())
    }

    // This function performs file/http IO and
    async fn resolve(&self, path: &str) -> Result<String> {
        if let Ok(file) = GoogleFileResolver::new().open_file(path) {
            return Ok(file
                .source()
                .context("Unable to extract content of google well-known proto file")?
                .to_string());
        }

        let source = match Url::parse(path) {
            Ok(url) => {
                let resp = self
                    .http_io
                    .execute(reqwest::Request::new(reqwest::Method::GET, url))
                    .await?
                    .body
                    .to_vec();
                String::from_utf8(resp.to_vec())?
            }
            Err(_) => self.file_io.read(path).await?,
        };
        Ok(source)
    }
}

#[cfg(test)]
mod test_proto_config {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};

    use crate::cli::{init_file, init_http};
    use crate::config::ConfigSetResolver;

    #[tokio::test]
    async fn test_resolve() -> Result<()> {
        // Skipping IO tests as they are covered in reader.rs

        let resolver = ConfigSetResolver::init(init_file(), init_http(&Default::default(), None));
        let empty = resolver.resolve("google/protobuf/empty.proto").await?;
        assert!(!empty.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_nested_imports() -> Result<()> {
        let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut test_dir = root_dir.join(file!());
        test_dir.pop(); // config
        test_dir.pop(); // src

        let mut root = test_dir.clone();
        root.pop();

        test_dir.push("grpc"); // grpc
        test_dir.push("tests"); // tests

        let mut test_file = test_dir.clone();

        test_file.push("nested0.proto"); // nested0.proto
        assert!(test_file.exists());
        let test_file = test_file.to_str().unwrap().to_string();

        let resolver = ConfigSetResolver::init(init_file(), init_http(&Default::default(), None));
        let mut helper_map = HashMap::new();
        resolver.import_all(&mut helper_map, test_file).await?;
        let files = test_dir.read_dir()?;
        for file in files {
            let file = file?;
            let path = file.path();
            let path_str =
                path_to_file_name(path.as_path()).context("It must be able to extract path")?;
            let source = tokio::fs::read_to_string(path).await?;
            let expected = protox_parse::parse(&path_str, &source)?;
            let actual = helper_map.get(&expected.name.unwrap()).unwrap();

            assert_eq!(&expected.dependency, &actual.dependency);
        }

        Ok(())
    }
    fn path_to_file_name(path: &Path) -> Option<String> {
        let components: Vec<_> = path.components().collect();

        // Find the index of the "src" component
        if let Some(src_index) = components.iter().position(|&c| c.as_os_str() == "src") {
            // Reconstruct the path from the "src" component onwards
            let after_src_components = &components[src_index..];
            let result = after_src_components
                .iter()
                .fold(PathBuf::new(), |mut acc, comp| {
                    acc.push(comp);
                    acc
                });
            Some(result.to_str().unwrap().to_string())
        } else {
            None
        }
    }
}
