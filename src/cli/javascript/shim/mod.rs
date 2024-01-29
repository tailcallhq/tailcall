use super::sync_v8::SyncV8;

mod console;
pub mod fetch;
pub fn init(v8: &SyncV8) -> anyhow::Result<()> {
    console::init(v8)?;
    fetch::init(v8)?;
    Ok(())
}
