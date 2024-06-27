use std::path::Path;
use std::sync::Arc;

use http::NativeHttpTest;
use tailcall::cli::generator::Generator;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::{self, ConfigModule};
use tailcall::core::generator::Generator as ConfigGenerator;
use tokio::runtime::Runtime;

mod cacache_manager;
mod http;

datatest_stable::harness!(
    run_config_generator_spec,
    "src/core/generator/tests/fixtures/generator",
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

async fn run_test(path: &str) -> anyhow::Result<()> {
    let mut runtime = tailcall::cli::runtime::init(&Blueprint::default());
    runtime.http = Arc::new(NativeHttpTest::default());

    let generator = Generator::new(path, runtime);
    let config = generator.read().await?;
    let preset: config::transformer::Preset = config.preset.clone().unwrap_or_default().into();

    // resolve i/o's
    let input_samples = generator.resolve_io(config).await?;

    let cfg_module = ConfigGenerator::default()
        .inputs(input_samples)
        .transformers(vec![Box::new(preset)])
        .generate(true)?;

    // remove links since they break snapshot tests
    let mut base_config = cfg_module.config().clone();
    base_config.links = Default::default();

    let config = ConfigModule::from(base_config);

    insta::assert_snapshot!(path, config.to_sdl());
    Ok(())
}
