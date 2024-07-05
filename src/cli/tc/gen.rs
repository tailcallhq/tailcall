use anyhow::Result;

use crate::cli::generator::Generator;
use crate::core::runtime::TargetRuntime;

pub(super) async fn gen_command(file_path: &str, runtime: TargetRuntime) -> Result<()> {
    Generator::new(file_path, runtime.clone())
        .generate()
        .await?;
    Ok(())
}
