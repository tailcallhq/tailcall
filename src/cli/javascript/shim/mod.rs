#[cfg(feature = "js")]
mod console;
#[cfg(feature = "js")]
pub fn init(v8: &mini_v8::MiniV8) -> anyhow::Result<()> {
  console::init(v8)?;
  Ok(())
}
