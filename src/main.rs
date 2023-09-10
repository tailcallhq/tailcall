use anyhow::Result;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    console_subscriber::init();
    tailcall::cli::run().await?;
    Ok(())
}
