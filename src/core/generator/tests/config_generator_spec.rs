use std::path::Path;

use tailcall::core::{blueprint::Blueprint, generator::FromGeneralizedConfig};
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

pub async fn run_test(path: &str) -> anyhow::Result<()> {
    let runtime = tailcall::cli::runtime::init(&Blueprint::default());
    let config_writer = FromGeneralizedConfig::new(runtime)
        .read(&path)
        .await?
        .generate()
        .await?;

    insta::assert_snapshot!(config_writer.config.to_sdl());

    Ok(())
}
