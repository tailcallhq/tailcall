use anyhow::Result;
use futures_util::future::join_all;
use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::DescriptorPool;
use reqwest::Method;
use url::Url;

use crate::core::config::{Config, ConfigModule, Link, LinkType};
use crate::core::generator::from_proto::from_proto;
use crate::core::generator::{from_json, ConfigGenerationRequest, Source};
use crate::core::merge_right::MergeRight;
use crate::core::proto_reader::ProtoReader;
use crate::core::resource_reader::ResourceReader;
use crate::core::runtime::TargetRuntime;

// this function resolves all the names to fully-qualified syntax in descriptors
// that is important for generation to work
// TODO: probably we can drop this in case the config_reader will use
// protox::compile instead of more low-level protox_parse::parse
fn resolve_file_descriptor_set(descriptor_set: FileDescriptorSet) -> Result<FileDescriptorSet> {
    let descriptor_set = DescriptorPool::from_file_descriptor_set(descriptor_set)?;
    let descriptor_set = FileDescriptorSet {
        file: descriptor_set
            .files()
            .map(|file| file.file_descriptor_proto().clone())
            .collect(),
    };

    Ok(descriptor_set)
}

// TODO: move this logic to ResourceReader.
async fn fetch_response(
    url: &str,
    runtime: &TargetRuntime,
) -> anyhow::Result<ConfigGenerationRequest> {
    let parsed_url = Url::parse(url)?;
    let request = reqwest::Request::new(Method::GET, parsed_url.clone());
    let resp = runtime.http.execute(request).await?;
    let body = serde_json::from_slice(&resp.body)?;
    Ok(ConfigGenerationRequest::new(parsed_url, body))
}

pub struct Generator {
    proto_reader: ProtoReader,
    runtime: TargetRuntime,
}
impl Generator {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self {
            runtime: runtime.clone(),
            proto_reader: ProtoReader::init(ResourceReader::cached(runtime.clone()), runtime),
        }
    }

    pub async fn read_all<T: AsRef<str>>(
        &self,
        input_source: Source,
        paths: &[T],
        query: &str,
    ) -> Result<ConfigModule> {
        match input_source {
            Source::Proto => {
                let mut links = vec![];
                let proto_metadata = self.proto_reader.read_all(paths).await?;

                let mut config = Config::default();
                for metadata in proto_metadata {
                    links.push(Link { id: None, src: metadata.path, type_of: LinkType::Protobuf });
                    let descriptor_set = resolve_file_descriptor_set(metadata.descriptor_set)?;
                    config = config.merge_right(from_proto(&[descriptor_set], query)?);
                }

                config.links = links;
                Ok(ConfigModule::from(config))
            }
            Source::Url => {
                let results = join_all(
                    paths
                        .iter()
                        .map(|url| fetch_response(url.as_ref(), &self.runtime)),
                )
                .await
                .into_iter()
                .collect::<anyhow::Result<Vec<_>>>()?;

                let config = from_json(&results, query)?;
                Ok(ConfigModule::from(config))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use tailcall_fixtures::protobuf;

    use super::*;

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    #[tokio::test]
    async fn test_read_all_with_grpc_gen() {
        let server = start_mock_server();
        let runtime = crate::core::runtime::test::init(None);
        let test_dir = PathBuf::from(tailcall_fixtures::protobuf::SELF);

        let news_content = runtime.file.read(protobuf::NEWS).await.unwrap();
        let greetings_a = runtime.file.read(protobuf::GREETINGS_A).await.unwrap();

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

        let generator = Generator::init(runtime);
        let news = format!("http://localhost:{}/news.proto", server.port());
        let greetings_a = format!("http://localhost:{}/greetings_a.proto", server.port());
        let greetings_b = test_dir
            .join("greetings_b.proto")
            .to_str()
            .unwrap()
            .to_string();

        let config = generator
            .read_all(Source::Proto, &[news, greetings_a, greetings_b], "Query")
            .await
            .unwrap();

        assert_eq!(config.links.len(), 3);
        assert_eq!(config.types.get("Query").unwrap().fields.len(), 8);
    }

    #[tokio::test]
    async fn test_read_all_with_rest_api_gen() {
        let runtime = crate::core::runtime::test::init(None);
        let generator = Generator::init(runtime);

        let users = "http://jsonplaceholder.typicode.com/users".to_string();
        let user = "http://jsonplaceholder.typicode.com/users/1".to_string();

        let config = generator
            .read_all(Source::Url, &[users, user], "Query")
            .await
            .unwrap();

        insta::assert_snapshot!(config.to_sdl());
    }

    #[tokio::test]
    async fn test_read_all_with_different_domain_rest_api_gen() {
        let runtime = crate::core::runtime::test::init(None);

        let generator = Generator::init(runtime);

        let user_comments = "https://jsonplaceholder.typicode.com/posts/1/comments".to_string();
        let post = "https://jsonplaceholder.typicode.com/posts/1".to_string();
        let laptops = "https://dummyjson.com/products/search?q=Laptop".to_string();

        let config = generator
            .read_all(Source::Url, &[user_comments, post, laptops], "Query")
            .await
            .unwrap();

        insta::assert_snapshot!(config.to_sdl());
    }
}
