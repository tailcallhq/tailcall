use std::path::Path;

use serde::{Deserialize, Serialize};
use tailcall::core::{
    blueprint::Blueprint,
    config,
    generator::{
        source::ImportSource, GeneratorConfig, GeneratorInput, GeneratorReImpl, InputSource,
        Resolved, UnResolved,
    },
    proto_reader::ProtoReader,
    resource_reader::ResourceReader,
};
use tokio::runtime::Runtime;

datatest_stable::harness!(
    run_config_generator_spec,
    "tailcall-fixtures/fixtures/generator/",
    r"^.*\.json"
);

pub fn run_config_generator_spec(path: &Path) -> datatest_stable::Result<()> {
    let path = path.to_path_buf();
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        run_test(&path.to_string_lossy()).await?;
        Ok(())
    })
}
#[derive(Serialize, Deserialize)]
struct GeneratorTest {
    config: GeneratorConfig<UnResolved>,
    #[serde(default)]
    resolvers: serde_json::Value,
}

/// performs all the i/o's required in the config file and generates
/// concrete vec containing data for generator.
async fn resolve_io(
    config: GeneratorConfig<Resolved>,
    resolvers: serde_json::Value,
) -> anyhow::Result<Vec<GeneratorInput>> {
    let mut generator_type_inputs = vec![];
    let runtime = tailcall::cli::runtime::init(&Blueprint::default());

    let reader = ResourceReader::cached(runtime.clone());
    let proto_reader = ProtoReader::init(reader.clone(), runtime.clone());

    for input in config.input.iter() {
        match &input.source {
            InputSource::Import { src, _marker } => {
                let source = ImportSource::detect(&src)?;
                match source {
                    ImportSource::Url => {
                        let content = resolvers
                            .get(src)
                            .unwrap_or(&serde_json::Value::Null)
                            .to_owned();
                        generator_type_inputs
                            .push(GeneratorInput::Json { url: src.parse()?, data: content })
                    }
                    ImportSource::Proto => {
                        let metadata = proto_reader.read(&src).await?;
                        generator_type_inputs.push(GeneratorInput::Proto { metadata });
                    }
                }
            }
            InputSource::Config { src, _marker } => {
                let source = config::Source::detect(&src)?;
                let schema = reader.read_file(&src).await?.content;
                generator_type_inputs.push(GeneratorInput::Config { schema, source });
            }
        }
    }

    Ok(generator_type_inputs)
}

pub async fn run_test(path: &str) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let config: GeneratorTest = serde_json::from_str(&contents)?;

    let resolved_config = config.config.resolve_paths(path)?;
    let generator_inp = resolve_io(resolved_config, config.resolvers).await?;
    let gen = GeneratorReImpl::new("f", "T");
    let cfg_module = gen.run("Query", &generator_inp)?;

    insta::assert_snapshot!(path, cfg_module.config.to_sdl());

    Ok(())
}
