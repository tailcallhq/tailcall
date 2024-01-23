use tailcall::{EventHandler, ScriptIO};

pub struct CloudflareScript {}

impl<Event, Command> ScriptIO<Event, Command> for CloudflareScript {
  fn event_handler(&self) -> anyhow::Result<impl EventHandler<Event, Command>> {
    Ok(CloudflareEventHandler {})
  }
}

pub struct CloudflareEventHandler {}

impl<Event, Command> EventHandler<Event, Command> for CloudflareEventHandler {
  fn on_event(&self, _: Event) -> anyhow::Result<Command> {
    unimplemented!("on event should not be called in cloudflare env")
  }
}
