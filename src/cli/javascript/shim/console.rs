#[cfg(feature = "js")]
use mini_v8::MiniV8;

#[cfg(feature = "js")]
use crate::cli::javascript::serde_v8::SerdeV8;

#[cfg(feature = "js")]
pub fn init(v8: &MiniV8) -> anyhow::Result<()> {
  let console = v8.create_object();
  console
    .set("log", v8.create_function(console_log))
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

  v8.global()
    .set("console", console)
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
  Ok(())
}

#[cfg(feature = "js")]
fn console_log(invocation: mini_v8::Invocation) -> Result<mini_v8::Value, mini_v8::Error> {
  let args = invocation
    .args
    .iter()
    .flat_map(|v| {
      let p = serde_json::Value::from_v8(v).map_err(|e| {
        log::error!("JS: {}", e.to_string());
        e
      });
      Some(p.ok()?.to_string())
    })
    .collect::<Vec<_>>()
    .join(",");
  log::info!("JS: {}", args);
  Ok(mini_v8::Value::Undefined)
}
