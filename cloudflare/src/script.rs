use tailcall::channel::{Command, Event};
use tailcall::ScriptIO;

pub struct CloudflareScript {}

#[async_trait::async_trait]
impl ScriptIO<Event, Command> for CloudflareScript {
  async fn on_event(&self, _: Event) -> anyhow::Result<Command> {
    unimplemented!("evaluate should not be called in cloudflare env")
  }
}
