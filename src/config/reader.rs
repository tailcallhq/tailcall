use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use anyhow::Context;
use futures_util::future::join_all;
use futures_util::TryFutureExt;
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use protox::file::{FileResolver, GoogleFileResolver};
use url::Url;

use super::{ConfigSet, ExprBody, Extensions, Script, ScriptOptions};
use crate::config::{Config, Source};
use crate::{FileIO, HttpIO};

const NULL_STR: &str = "\0\0\0\0\0\0\0";

/// Reads the configuration from a file or from an HTTP URL and resolves all linked extensions to create a ConfigSet.
pub struct ConfigReader {
    file_io: Arc<dyn FileIO>,
    http_io: Arc<dyn HttpIO>,
}

/// Response of a file read operation
struct FileRead {
    content: String,
    path: String,
}

impl ConfigReader {
    pub fn init(file_io: Arc<dyn FileIO>, http_io: Arc<dyn HttpIO>) -> Self {
        Self { file_io, http_io }
    }

    /// Reads a file from the filesystem or from an HTTP URL
    async fn read_file<T: ToString>(&self, file: T) -> anyhow::Result<FileRead> {
        // Is an HTTP URL
        let content = if let Ok(url) = Url::parse(&file.to_string()) {
            let response = self
                .http_io
                .execute(reqwest::Request::new(reqwest::Method::GET, url))
                .await?;

            String::from_utf8(response.body.to_vec())?
        } else {
            // Is a file path

            self.file_io.read(&file.to_string()).await?
        };

        Ok(FileRead { content, path: file.to_string() })
    }

    /// Reads all the files in parallel
    async fn read_files<T: ToString>(&self, files: &[T]) -> anyhow::Result<Vec<FileRead>> {
        let files = files.iter().map(|x| {
            self.read_file(x.to_string())
                .map_err(|e| e.context(x.to_string()))
        });
        let content = join_all(files)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(content)
    }

    /// Reads the script file and replaces the path with the content
    async fn ext_script(&self, mut config_set: ConfigSet) -> anyhow::Result<ConfigSet> {
        let config = &mut config_set.config;
        if let Some(Script::Path(ref options)) = &config.server.script {
            let timeout = options.timeout;
            let script = self.read_file(options.src.clone()).await?.content;
            config.server.script = Some(Script::File(ScriptOptions { src: script, timeout }));
        }
        Ok(config_set)
    }

    /// Reads a single file and returns the config
    pub async fn read<T: ToString>(&self, file: T) -> anyhow::Result<ConfigSet> {
        self.read_all(&[file]).await
    }

    /// Reads all the files and returns a merged config
    pub async fn read_all<T: ToString>(&self, files: &[T]) -> anyhow::Result<ConfigSet> {
        let files = self.read_files(files).await?;
        let mut config_set = ConfigSet::default();

        for file in files.iter() {
            let source = Source::detect(&file.path)?;
            let schema = &file.content;

            // Create initial config set
            let new_config_set = self.resolve(Config::from_source(source, schema)?).await?;

            // Merge it with the original config set
            config_set = config_set.merge_right(&new_config_set);
        }
        Ok(config_set)
    }

    /// Resolves all the links in a Config to create a ConfigSet
    pub async fn resolve(&self, config: Config) -> anyhow::Result<ConfigSet> {
        // Create initial config set
        let config_set = ConfigSet::from(config);

        // Extend it with the worker script
        let config_set = self.ext_script(config_set).await?;

        // Extend it with protobuf definitions for GRPC
        let config_set = self.ext_grpc(config_set).await?;

        Ok(config_set)
    }

    /// Returns final ConfigSet from Config
    pub async fn ext_grpc(&self, mut config_set: ConfigSet) -> anyhow::Result<ConfigSet> {
        let config = &config_set.config;
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
                    descriptors = self
                        .resolve_descriptors(descriptors, proto_path.to_string())
                        .await?;
                }
            }
        }
        for (_, v) in descriptors {
            grpc_file_descriptor.file.push(v);
        }

        config_set.extensions = Extensions { grpc_file_descriptor, ..Default::default() };
        Ok(config_set)
    }

    /// Performs BFS to import all nested proto files
    async fn resolve_descriptors(
        &self,
        mut descriptors: HashMap<String, FileDescriptorProto>,
        proto_path: String,
    ) -> anyhow::Result<HashMap<String, FileDescriptorProto>> {
        let parent_proto = self.read_proto(&proto_path).await?;
        let mut queue = VecDeque::new();
        queue.push_back(parent_proto.clone());

        while let Some(file) = queue.pop_front() {
            for import in file.dependency.iter() {
                let proto = self.read_proto(import).await?;
                if descriptors.get(import).is_none() {
                    queue.push_back(proto.clone());
                    descriptors.insert(import.clone(), proto);
                }
            }
        }

        descriptors.insert(proto_path, parent_proto);

        Ok(descriptors)
    }

    /// Tries to load well-known google proto files and if not found uses normal file and http IO to resolve them
    async fn read_proto(&self, path: &str) -> anyhow::Result<FileDescriptorProto> {
        let content = if let Ok(file) = GoogleFileResolver::new().open_file(path) {
            file.source()
                .context("Unable to extract content of google well-known proto file")?
                .to_string()
        } else {
            self.read_file(path).await?.content
        };

        Ok(protox_parse::parse(path, &content)?)
    }
}

#[cfg(test)]
mod test_proto_config {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};

    use crate::cli::{init_file, init_http};
    use crate::config::reader::ConfigReader;

    #[tokio::test]
    async fn test_resolve() {
        // Skipping IO tests as they are covered in reader.rs
        let reader = ConfigReader::init(init_file(), init_http(&Default::default(), None));
        reader
            .read_proto("google/protobuf/empty.proto")
            .await
            .unwrap();
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

        let reader = ConfigReader::init(init_file(), init_http(&Default::default(), None));
        let helper_map = reader
            .resolve_descriptors(HashMap::new(), test_file)
            .await?;
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

#[cfg(test)]
mod reader_tests {
    use anyhow::Context;
    use pretty_assertions::assert_eq;
    use tokio::io::AsyncReadExt;

    use crate::cli::{init_file, init_http};
    use crate::config::reader::ConfigReader;
    use crate::config::{Config, Script, ScriptOptions, Type, Upstream};

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    #[tokio::test]
    async fn test_all() {
        let file_io = init_file();
        let http_io = init_http(&Upstream::default(), None);

        let mut cfg = Config::default();
        cfg.schema.query = Some("Test".to_string());
        cfg = cfg.types([("Test", Type::default())].to_vec());

        let server = start_mock_server();
        let header_serv = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/bar.graphql");
            then.status(200).body(cfg.to_sdl());
        });

        let mut json = String::new();
        tokio::fs::File::open("examples/jsonplaceholder.json")
            .await
            .unwrap()
            .read_to_string(&mut json)
            .await
            .unwrap();

        let foo_json_server = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/foo.json");
            then.status(200).body(json);
        });

        let port = server.port();
        let files: Vec<String> = [
            "examples/jsonplaceholder.yml", // config from local file
            format!("http://localhost:{port}/bar.graphql").as_str(), // with content-type header
            format!("http://localhost:{port}/foo.json").as_str(), // with url extension
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let cr = ConfigReader::init(file_io, http_io);
        let c = cr.read_all(&files).await.unwrap();
        assert_eq!(
            ["Post", "Query", "Test", "User"]
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>(),
            c.types
                .keys()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
        foo_json_server.assert(); // checks if the request was actually made
        header_serv.assert();
    }

    #[tokio::test]
    async fn test_local_files() {
        let file_io = init_file();
        let http_io = init_http(&Upstream::default(), None);

        let files: Vec<String> = [
            "examples/jsonplaceholder.yml",
            "examples/jsonplaceholder.graphql",
            "examples/jsonplaceholder.json",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let cr = ConfigReader::init(file_io, http_io);
        let c = cr.read_all(&files).await.unwrap();
        assert_eq!(
            ["Post", "Query", "User"]
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>(),
            c.types
                .keys()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
    }

    #[tokio::test]
    async fn test_script_loader() {
        let file_io = init_file();
        let http_io = init_http(&Upstream::default(), None);

        let cargo_manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let reader = ConfigReader::init(file_io, http_io);

        let config = reader
            .read(&format!(
                "{}/examples/jsonplaceholder_script.graphql",
                cargo_manifest
            ))
            .await
            .unwrap();

        let path = format!("{}/examples/scripts/echo.js", cargo_manifest);
        let file = ScriptOptions {
            src: String::from_utf8(
                tokio::fs::read(&path)
                    .await
                    .context(path.to_string())
                    .unwrap(),
            )
            .unwrap(),
            timeout: None,
        };
        assert_eq!(config.server.script, Some(Script::File(file)),);
    }
}
