use std::env;
use std::path::PathBuf;
use anyhow::Result;
use env_logger::Env;

fn logger_init() {
    let env = Env::new();
    env_logger::Builder::from_env(env).init();
}

fn main() {
    logger_init();

    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();

    match update_json_schema(root_dir, "examples/jsonplaceholder.json") {
        Ok(_) => {
            log::info!("Json Schema updated successfully.")
        },
        Err(e) => {
            log::error!("Unable to update json schema due to: {}", e)
        }
    }

}

fn update_json_schema(mut root_dir: PathBuf, path: &str) -> Result<()> {
    root_dir.push(path);
    println!("{:?}", root_dir);
    Ok(())
}
