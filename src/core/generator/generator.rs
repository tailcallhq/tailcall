use anyhow::Result;

use crate::core::config::{Config, ConfigModule, Link, LinkType, Resolution};
use crate::core::generator::from_proto::from_proto;
use crate::core::generator::Source;
use crate::core::merge_right::MergeRight;
use crate::core::proto_reader::ProtoReader;
use crate::core::resource_reader::ResourceReader;
use crate::core::runtime::TargetRuntime;

pub struct Generator {
    proto_reader: ProtoReader,
}
impl Generator {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self {
            proto_reader: ProtoReader::init(ResourceReader::cached(runtime.clone()), runtime),
        }
    }

    pub async fn read_all<T: AsRef<str>>(
        &self,
        input_source: Source,
        files: &[T],
        query: &str,
    ) -> Result<ConfigModule> {
        let mut links = vec![];
        let proto_metadata = self.proto_reader.read_all(files).await?;

        let mut config = Config::default();
        for metadata in proto_metadata {
            match input_source {
                Source::Proto => {
                    links.push(Link { id: None, src: metadata.path, type_of: LinkType::Protobuf });
                    config = config.merge_right(from_proto(&[metadata.descriptor_set], query));
                }
            }
        }

        config.links = links;
        Ok(
            ConfigModule::from(config).resolve_ambiguous_types(|v| Resolution {
                input: format!("IN_{}", v),
                output: format!("OUT_{}", v),
            }),
        )
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
        let runtime = crate::core::runtime::test::init(None);
        let test_dir = PathBuf::from(tailcall_fixtures::generator::proto::SELF);

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

        let reader = Generator::init(runtime);
        let news = format!("http://localhost:{}/news.proto", server.port());
        let greetings_a = format!("http://localhost:{}/greetings_a.proto", server.port());
        let greetings_b = test_dir
            .join("greetings_b.proto")
            .to_str()
            .unwrap()
            .to_string();

        let config = reader
            .read_all(Source::Proto, &[news, greetings_a, greetings_b], "Query")
            .await
            .unwrap();

        assert_eq!(config.links.len(), 3);
        assert_eq!(config.types.get("Query").unwrap().fields.len(), 8);
    }
}
