mod native;
mod wasm;

#[async_trait::async_trait]
pub trait ConsoleOperation {
  async fn say(message: &str) -> anyhow::Result<()>;
  async fn ask(message: &str) -> anyhow::Result<String>;
}

pub struct Console {}

impl Console {
  pub fn init() -> Self {
    Console {}
  }
}
