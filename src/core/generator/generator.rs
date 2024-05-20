use std::collections::HashMap;

use anyhow::Result;
use futures_util::future::join_all;
use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::DescriptorPool;
use reqwest::Method;
use serde_json::Value;
use url::Url;

use crate::core::config::{Config, ConfigModule, Link, LinkType, Resolution};
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
async fn fetch_response(url: &str, runtime: &TargetRuntime) -> anyhow::Result<Value> {
    let parsed_url = Url::parse(url)?;
    let request = reqwest::Request::new(Method::GET, parsed_url);
    let resp = runtime.http.execute(request).await?;
    let body = serde_json::from_slice(&resp.body)?;
    Ok(body)
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
    ) -> Result<Vec<ConfigModule>> {
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
                Ok(vec![ConfigModule::from(config).resolve_ambiguous_types(
                    |v| Resolution { input: format!("{}Input", v), output: v.to_owned() },
                )])
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

                // create a config generation requests.
                let config_gen_reqs = results
                    .iter()
                    .zip(paths.iter())
                    .map(|(resp, url)| ConfigGenerationRequest::new(url.as_ref(), resp))
                    .collect::<Vec<ConfigGenerationRequest>>();

                // group requests with same domain name to have single config.
                // and pass each group to from_json to generate the config.
                let mut domain_groupings: HashMap<String, Vec<ConfigGenerationRequest>> =
                    HashMap::new();
                for req in config_gen_reqs {
                    let url = Url::parse(req.url)?;
                    let domain = url.host_str().unwrap();
                    domain_groupings
                        .entry(domain.to_string())
                        .or_default()
                        .push(req);
                }

                let mut configs = Vec::with_capacity(domain_groupings.len());

                for (_, same_domain_group_req) in domain_groupings {
                    configs.push(ConfigModule::from(from_json(&same_domain_group_req)?));
                }

                Ok(configs)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use tailcall_fixtures::{json, protobuf};

    use super::*;

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    fn parse_to_json(content: String) -> anyhow::Result<Value> {
        let json_content: serde_json::Value = serde_json::from_str(&content)?;
        Ok(json_content["body"].clone())
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

        assert_eq!(config.first().unwrap().links.len(), 3);
        assert_eq!(
            config
                .first()
                .unwrap()
                .types
                .get("Query")
                .unwrap()
                .fields
                .len(),
            8
        );
    }

    async fn read_json_fixtures(runtime: &TargetRuntime, fixture_path: &str) -> Value {
        let content = runtime.file.read(fixture_path).await.unwrap();

        parse_to_json(content).unwrap()
    }

    #[tokio::test]
    async fn test_read_all_with_rest_api_gen() -> anyhow::Result<()> {
        let runtime = crate::core::runtime::test::init(None);
        let server = start_mock_server();

        let list_content = read_json_fixtures(&runtime, json::LIST).await;
        let incompatible_properties =
            read_json_fixtures(&runtime, json::INCOMPATIBLE_PROPERTIES).await;

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/list");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(list_content.to_string());
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/incompatible_properties");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(incompatible_properties.to_string());
        });

        let generator = Generator::init(runtime);
        let list_url = format!("http://localhost:{}/list", server.port());
        let incompatible_properties_url =
            format!("http://localhost:{}/incompatible_properties", server.port());

        let config = generator
            .read_all(
                Source::Url,
                &[list_url, incompatible_properties_url],
                "Query",
            )
            .await
            .unwrap();

        assert_eq!(config.len(), 1);
        insta::assert_snapshot!(config.first().unwrap().to_sdl());
        Ok(())
    }

    #[tokio::test]
    async fn test_read_all_with_different_domain_rest_api_gen() -> anyhow::Result<()> {
        let server = start_mock_server();
        let runtime = crate::core::runtime::test::init(None);

        let list_content = read_json_fixtures(&runtime, json::LIST).await;
        let incompatible_properties =
            read_json_fixtures(&runtime, json::INCOMPATIBLE_PROPERTIES).await;

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/list");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(list_content.to_string());
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/incompatible_properties");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(incompatible_properties.to_string());
        });

        let generator = Generator::init(runtime);
        let list_url = format!("http://localhost:{}/list", server.port());
        let incompatible_properties_url =
            format!("http://localhost:{}/incompatible_properties", server.port());

        let jsonplaceholder_users = "http://jsonplaceholder.typicode.com/users".to_string();

        let config = generator
            .read_all(
                Source::Url,
                &[list_url, incompatible_properties_url, jsonplaceholder_users],
                "Query",
            )
            .await
            .unwrap();

        assert_eq!(config.len(), 2);
        for cfg in config.iter() {
            let base_url = cfg.upstream.base_url.clone();
            let url = Url::parse(base_url.unwrap().as_str())?;
            let host_name = url.host_str().unwrap();
            insta::assert_snapshot!(host_name, cfg.to_sdl());
        }
        Ok(())
    }
}
