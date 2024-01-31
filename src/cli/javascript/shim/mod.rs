use mini_v8::MiniV8;
use tokio::sync::mpsc;

use super::async_wrapper::ChannelMessage;

mod console;
pub mod fetch;

pub fn init(v8: &MiniV8, http_sender: mpsc::UnboundedSender<ChannelMessage>) -> anyhow::Result<()> {
    console::init(v8)?;
    fetch::init(v8.clone(), http_sender)?;
    Ok(())
}
