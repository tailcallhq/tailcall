use anyhow::Result;

use crate::config::{Config, Link, LinkType};
use crate::config_generator::from_proto::from_proto;
use crate::config_generator::source::GeneratorSource;
use crate::merge_right::MergeRight;
use crate::proto_reader::ProtoReader;
use crate::runtime::TargetRuntime;

pub struct GeneratorReader {
    proto_reader: ProtoReader,
}
impl GeneratorReader {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self { proto_reader: ProtoReader::init(runtime) }
    }

    pub async fn read_all<T: AsRef<str>>(&self, files: &[T], query: &str) -> Result<Config> {
        let mut links = vec![];
        let proto_metadata = self.proto_reader.read_all(files).await?;

        let mut config = Config::default();

        for metadata in proto_metadata {
            let source = match metadata.file_read.content_ty {
                Some(content_ty) => match GeneratorSource::detect(content_ty.as_str()) {
                    Ok(source) => source,
                    Err(_) => GeneratorSource::detect(metadata.file_read.path.as_str())?,
                },
                None => GeneratorSource::detect(metadata.file_read.path.as_str())?,
            };
            match source {
                GeneratorSource::PROTO => {
                    links.push(Link {
                        id: None,
                        src: metadata.file_read.path.clone(),
                        type_of: LinkType::Protobuf,
                    });
                    config = config.merge_right(from_proto(&[metadata.descriptor_set], query));
                }
            }
        }

        config.links = links;
        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    #[tokio::test]
    async fn test_read_all() {
        let server = start_mock_server();
        let runtime = crate::runtime::test::init(None);
        let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/config_generator/proto");

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
