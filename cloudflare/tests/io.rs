#[cfg(test)]
mod test_cf_io {
  use std::rc::Rc;

  use lazy_static::lazy_static;
  use serde_json::Value;
  use tailcall::EnvIO;
  use wasm_bindgen_test::wasm_bindgen_test;
  use worker::wasm_bindgen::JsValue;
  use worker::Env;

  wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

  lazy_static! {
    static ref VENV: Value = {
      let mut venv = serde_json::Map::new();
      venv.insert(
        "CONFIG".to_string(),
        Value::String("/MY_R2/examples/jsonplaceholder.graphql".to_string()),
      );
      venv.insert("MY_R2".to_string(), Value::Object(serde_json::Map::new()));
      Value::Object(venv)
    };
  }

  #[wasm_bindgen_test]
  async fn env_io() {
    let env_io = cloudflare::init_env(get_venv());
    assert_eq!(env_io.get("CONFIG").unwrap(), "/MY_R2/examples/jsonplaceholder.graphql");
  }
  fn get_venv() -> Rc<Env> {
    let js_val = <JsValue as gloo_utils::format::JsValueSerdeExt>::from_serde(&VENV.clone())
      .expect("Failed to serialize map to JsValue");
    Rc::new(worker::Env::from(js_val))
  }
}
