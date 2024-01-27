mod console;
pub mod fetch;
pub fn init(v8: &mini_v8::MiniV8) -> anyhow::Result<()> {
    console::init(v8)?;
    fetch::init(v8)?;
    Ok(())
}
