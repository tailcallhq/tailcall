use tailcall::EnvIO;

pub struct WasmEnv {}
impl WasmEnv {
  pub fn new() -> Self {
    Self {}
  }
}
impl EnvIO for WasmEnv {
  fn get(&self, key: &str) -> Option<String> {
    unimplemented!("Not needed for npm pkg")
  }
}
