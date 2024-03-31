use std::collections::{HashMap, VecDeque};
use std::iter::Extend;

use anyhow::{Context, Result};
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use protox::file::{FileResolver, GoogleFileResolver};
use url::Url;

use crate::config::{Config, Link, LinkType};
use crate::config_generator::from_proto::from_proto;
use crate::config_generator::source::GeneratorSource;
use crate::merge_right::MergeRight;
use crate::runtime::TargetRuntime;

pub struct GeneratorReader {
    runtime: TargetRuntime,
}

struct FileRead {
    content: String,
    source: GeneratorSource,
    path: String,
}

impl GeneratorReader {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self { runtime: runtime.clone() }
    }
    pub async fn read_all<T: AsRef<str>>(&self, files: &[T], query: &str) -> Result<Config> {
        let mut links = vec![];
        let mut config = Config::default();

        for file in files {
            let file_read = self.read_file(file).await?;
            match file_read.source {
                GeneratorSource::PROTO => {
                    let mut descriptors = FileDescriptorSet::default();

                    links.push(Link {
                        id: None,
                        src: file.as_ref().to_string(),
                        type_of: LinkType::Protobuf,
                    });

                    let parent_descriptor = self.read_proto(file_read).await?;
                    descriptors
                        .file
                        .extend(self.resolve_descriptors(parent_descriptor).await?);

                    config = config.merge_right(from_proto(&[descriptors], query));
                }
            }
        }
        config.links = links;
        Ok(config)
    }

    /// Reads a file from the filesystem or from an HTTP URL
    async fn read_file<T: AsRef<str>>(&self, file: T) -> Result<FileRead> {
        if let Ok(file) = GoogleFileResolver::new().open_file(file.as_ref()) {
            let content = file
                .source()
                .context("Unable to extract content of google well-known proto file")?
                .to_string();
            return Ok(FileRead {
                content,
                source: GeneratorSource::PROTO,
                path: file.name().to_string(),
            });
        }
        // Is an HTTP URL
        let file_read = if let Ok(url) = Url::parse(file.as_ref()) {
            if url.scheme().starts_with("http") {
                let response = self
                    .runtime
                    .http
                    .execute(reqwest::Request::new(reqwest::Method::GET, url))
                    .await?;
                let source = if let Some(content_ty) = response.headers.get("content-type") {
                    let value = content_ty.to_str()?;
                    match value {
                        "application/x-protobuf"
                        | "application/protobuf"
                        | "application/vnd.google.protobuf" => GeneratorSource::PROTO,
                        value => GeneratorSource::detect(value)
                            .unwrap_or(GeneratorSource::detect(file.as_ref())?),
                    }
                } else {
                    GeneratorSource::detect(file.as_ref())?
                };

                FileRead {
                    content: String::from_utf8(response.body.to_vec())?,
                    source,
                    path: file.as_ref().to_string(),
                }
            } else {
                // Is a file path on Windows
                let source = GeneratorSource::detect(file.as_ref())?;
                let content = self.runtime.file.read(file.as_ref()).await?;
                FileRead { content, source, path: file.as_ref().to_string() }
            }
        } else {
            // Is a file path
            let source = GeneratorSource::detect(file.as_ref())?;
            let content = self.runtime.file.read(file.as_ref()).await?;
            FileRead { content, source, path: file.as_ref().to_string() }
        };

        Ok(file_read)
    }

    /// Tries to load well-known google proto files and if not found uses normal
    /// file and http IO to resolve them
    async fn read_proto(&self, file_read: FileRead) -> Result<FileDescriptorProto> {
        Ok(protox_parse::parse(
            file_read.path.as_ref(),
            &file_read.content,
        )?)
    }

    /// Performs BFS to import all nested proto files
    pub async fn resolve_descriptors(
        &self,
        parent_proto: FileDescriptorProto,
    ) -> Result<Vec<FileDescriptorProto>> {
        let mut descriptors: HashMap<String, FileDescriptorProto> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(parent_proto.clone());

        while let Some(file) = queue.pop_front() {
            for import in file.dependency.iter() {
                let file_read = self.read_file(import).await?;
                let proto = self.read_proto(file_read).await?;
                if descriptors.get(import).is_none() {
                    queue.push_back(proto.clone());
                    descriptors.insert(import.clone(), proto);
                }
            }
        }
        let mut descriptors = descriptors
            .into_values()
            .collect::<Vec<FileDescriptorProto>>();
        descriptors.push(parent_proto);
        Ok(descriptors)
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::config_generator::source::GeneratorSource;

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    #[tokio::test]
    async fn test_read_file_http() {
        let server = start_mock_server();
        let fakeproto = "fakeproto";
        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test.proto");
            then.status(200)
                .header("Content-Type", "application/x-protobuf")
                .body(fakeproto);
        });

        let runtime = crate::runtime::test::init(None);
        let reader = GeneratorReader::init(runtime);
        let file = format!("http://localhost:{}/test.proto", server.port());
        let file_read = reader.read_file(file.as_str()).await.unwrap();
        assert_eq!(file_read.source, GeneratorSource::PROTO);
        assert_eq!(file_read.content, fakeproto);
        assert_eq!(file_read.path, file);
    }

    #[tokio::test]
    async fn test_read_all() {
        let server = start_mock_server();
        let runtime = crate::runtime::test::init(None);
        let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("config_generator")
            .join("proto");

        let news_content = runtime
            .file
            .read(test_dir.join("news.proto").to_str().unwrap())
            .await
            .unwrap();
        let greetings_a = runtime
            .file
            .read(test_dir.join("greetings_a.proto").to_str().unwrap())
            .await
            .unwrap();

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/news.proto");
            then.status(200)
                .header("Content-Type", "application/vnd.google.protobuf")
                .body(&news_content);
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/greetings_a.proto");
            then.status(200)
                .header("Content-Type", "application/protobuf")
                .body(&greetings_a);
        });

        let reader = GeneratorReader::init(runtime);
        let news = format!("http://localhost:{}/news.proto", server.port());
        let greetings_a = format!("http://localhost:{}/greetings_a.proto", server.port());
        let greetings_b = test_dir
            .join("greetings_b.proto")
            .to_str()
            .unwrap()
            .to_string();

        let config = reader
            .read_all(&[news, greetings_a, greetings_b], "Query")
            .await
            .unwrap();

        assert_eq!(config.links.len(), 3);
        assert_eq!(config.types.get("Query").unwrap().fields.len(), 3);
    }
}
