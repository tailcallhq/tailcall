use crate::cli::generator::Generator;
use crate::core::error::Error;
use crate::core::runtime::TargetRuntime;

pub(super) async fn gen_command(file_path: &str, runtime: TargetRuntime) -> Result<(), Error> {
    Generator::new(file_path, runtime.clone())
        .generate()
        .await?;
    Ok(())
}
