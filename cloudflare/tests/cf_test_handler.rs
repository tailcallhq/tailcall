#[cfg(test)]
mod cf_test_resp {
  use gloo_utils::format::JsValueSerdeExt;
  use serde_json::{json, Value};
  use wasm_bindgen_test::wasm_bindgen_test;
  use worker::wasm_bindgen::JsValue;
  use worker::{Env, Request, RequestInit};

  #[wasm_bindgen_test]
  async fn test_resp_get() {
    let req = Request::new("localhost:19194", worker::Method::Get).unwrap();
    let mut resp = cloudflare::handle::fetch(req, Env::from(JsValue::null()))
      .await
      .unwrap();
    assert!(resp
      .text()
      .await
      .unwrap()
      .contains("<title>Tailcall - GraphQL IDE</title>"));
  }

  #[wasm_bindgen_test]
  async fn test_resp_post() {
    let qry = JsValue::from_str(
      "{\"operationName\":null,\"variables\":{},\"query\":\"{\\n  user(id: 1) {\\n    id\\n  }\\n}\\n\"}",
    );
    let mut req_init = RequestInit::new();
    req_init.method = worker::Method::Post;
    req_init.body = Some(qry);

    let env = json!({
        "BUCKET": "MY_R2",
    });
    let env_val = <JsValue as JsValueSerdeExt>::from_serde(&env).unwrap();
    let env = Env::from(env_val.clone());

    let req = Request::new_with_init("http://localhost:19194/graphql?config=https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql", &req_init).unwrap();
    let mut resp = cloudflare::handle::fetch(req, env).await.unwrap();
    assert_eq!(
      json!({
          "data": {
              "user": {
                  "id": 1
              }
          }
      }),
      resp.json::<Value>().await.unwrap()
    );
  }
}
