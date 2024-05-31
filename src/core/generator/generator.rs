use anyhow::Result;
use futures_util::future::join_all;
use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::DescriptorPool;

use crate::core::generator::from_proto::from_proto;
use crate::core::generator::source::ImportSource;
use crate::core::merge_right::MergeRight;
use crate::core::proto_reader::ProtoReader;
use crate::core::resource_reader::ResourceReader;
use crate::core::runtime::TargetRuntime;
use crate::core::{
    config::{self, Config, ConfigModule, Link, LinkType, Resolution},
    resource_reader::Cached,
};

use super::config::{GeneratorConfig, InputSource};

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

pub struct Generator {
    runtime: TargetRuntime,
    reader: ResourceReader<Cached>,
    proto_reader: ProtoReader,
}

impl Generator {
    pub fn new(runtime: TargetRuntime) -> Self {
        let reader = ResourceReader::cached(runtime.clone());

        Self {
            runtime: runtime.clone(),
            reader: reader.clone(),
            proto_reader: ProtoReader::init(reader, runtime),
        }
    }

    pub async fn run(&self, gen_config: GeneratorConfig) -> Result<()> {
        let resolvers = gen_config.input.into_iter().map(|input| async {
            match input.source {
                InputSource::Config { src } => {
                    let source = config::Source::detect(&src)?;
                    let schema = self.reader.read_file(&src).await?;

                    Config::from_source(source, &schema.content)
                }
                InputSource::Import { src } => {
                    let source = ImportSource::detect(&src)?;

                    match source {
                        ImportSource::Proto => {
                            let metadata = self.proto_reader.read(&src).await?;
                            let descriptor_set =
                                resolve_file_descriptor_set(metadata.descriptor_set)?;
                            let mut config = from_proto(&[descriptor_set], "Query")?;

                            config.links.push(Link {
                                id: None,
                                src: metadata.path,
                                type_of: LinkType::Protobuf,
                            });

                            Ok(config)
                        }
                    }
                }
            }
        });

        let mut config = Config::default();
        for result in join_all(resolvers).await {
            config = config.merge_right(result?)
        }

        let config = ConfigModule::from(config).resolve_ambiguous_types(|v| Resolution {
            input: format!("{}Input", v),
            output: v.to_owned(),
        });

        let output_source = config::Source::detect(&gen_config.output.file)?;

        let config = match output_source {
            config::Source::Json => config.to_json(true)?,
            config::Source::Yml => config.to_yaml()?,
            config::Source::GraphQL => config.to_sdl(),
        };

        self.runtime
            .file
            .write(&gen_config.output.file, config.as_bytes())
            .await?;

        Ok(())
    }

    pub async fn read_all<T: AsRef<str>>(
        &self,
        input_source: ImportSource,
        files: &[T],
        query: &str,
    ) -> Result<ConfigModule> {
        let mut links = vec![];
        let proto_metadata = self.proto_reader.read_all(files).await?;

        let mut config = Config::default();
        for metadata in proto_metadata {
            match input_source {
                ImportSource::Proto => {
                    links.push(Link { id: None, src: metadata.path, type_of: LinkType::Protobuf });
                    let descriptor_set = resolve_file_descriptor_set(metadata.descriptor_set)?;
                    config = config.merge_right(from_proto(&[descriptor_set], query)?);
                }
            }
        }

        config.links = links;
        Ok(
            ConfigModule::from(config).resolve_ambiguous_types(|v| Resolution {
                input: format!("{}Input", v),
                output: v.to_owned(),
            }),
        )
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
    async fn test_read_all() {
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

        let generator = Generator::new(runtime);
        let news = format!("http://localhost:{}/news.proto", server.port());
        let greetings_a = format!("http://localhost:{}/greetings_a.proto", server.port());
        let greetings_b = test_dir
            .join("greetings_b.proto")
            .to_str()
            .unwrap()
            .to_string();

        let config = generator
            .read_all(ImportSource::Proto, &[news, greetings_a, greetings_b], "Query")
            .await
            .unwrap();

        assert_eq!(config.links.len(), 3);
        assert_eq!(config.types.get("Query").unwrap().fields.len(), 8);
    }
}
