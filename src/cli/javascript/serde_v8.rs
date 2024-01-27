#[cfg(feature = "js")]
use mini_v8::{MiniV8, Value};
#[cfg(feature = "js")]
use serde::de::DeserializeOwned;
#[cfg(feature = "js")]
use serde::Serialize;
#[cfg(feature = "js")]
use serde_json::Number;
#[cfg(feature = "js")]
pub trait SerdeV8: Sized {
  fn to_v8(self, mv8: &MiniV8) -> anyhow::Result<Value>;
  fn from_v8(value: &Value) -> anyhow::Result<Self>;
}
#[cfg(feature = "js")]
fn v8_serde(value: mini_v8::Value) -> anyhow::Result<serde_json::Value> {
  let serde_value: serde_json::Value = match value {
    Value::Undefined => serde_json::Value::Null,
    Value::Null => serde_json::Value::Null,
    Value::Boolean(v) => serde_json::Value::Bool(v),
    Value::Number(n) => {
      serde_json::Value::Number(Number::from_f64(n).ok_or(anyhow::anyhow!("error converting number"))?)
    }
    Value::String(s) => serde_json::Value::String(s.to_string()),
    Value::Array(v) => {
      let mut arr = Vec::new();
      for v in v.elements::<Value>() {
        arr.push(v8_serde(v.map_err(|e| anyhow::anyhow!(e.to_string()))?)?);
      }
      serde_json::Value::Array(arr)
    }
    Value::Function(_) => serde_json::Value::Null,
    Value::Object(v) => {
      let mut obj = serde_json::Map::new();
      let props = v.properties(false).map_err(|e| anyhow::anyhow!(e.to_string()))?;
      for kv in props {
        let (k, v) = kv.map_err(|e| anyhow::anyhow!(e.to_string()))?;
        obj.insert(k, v8_serde(v)?);
      }
      serde_json::Value::Object(obj)
    }
    Value::Date(d) => serde_json::Value::Number(Number::from_f64(d).ok_or(anyhow::anyhow!("error converting date"))?),
  };

  Ok(serde_value)
}
#[cfg(feature = "js")]
fn serde_v8(value: serde_json::Value, v8: &mini_v8::MiniV8) -> anyhow::Result<mini_v8::Value> {
  let value: mini_v8::Value = match value {
    serde_json::Value::Null => Value::Null,
    serde_json::Value::Bool(b) => Value::Boolean(b),
    serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or_default()),
    serde_json::Value::String(s) => Value::String(v8.create_string(s.as_str())),
    serde_json::Value::Array(a) => {
      let arr = v8.create_array();
      for v in a {
        arr.push(serde_v8(v, v8)?).map_err(|e| anyhow::anyhow!(e.to_string()))?;
      }
      Value::Array(arr)
    }
    serde_json::Value::Object(obj) => {
      let out = v8.create_object();
      for (k, v) in obj {
        out
          .set(k, serde_v8(v, v8)?)
          .map_err(|e| anyhow::anyhow!(e.to_string()))?;
      }
      Value::Object(out)
    }
  };
  Ok(value)
}
#[cfg(feature = "js")]
impl<A: Serialize + DeserializeOwned> SerdeV8 for A {
  fn to_v8(self, mv8: &MiniV8) -> anyhow::Result<Value> {
    let json = serde_json::to_value(&self)?;
    log::debug!("json: {}", json);
    serde_v8(json, mv8)
  }

  fn from_v8(value: &Value) -> anyhow::Result<A> {
    let serde_value = v8_serde(value.clone())?;
    let value: A = serde_json::from_value(serde_value)?;
    Ok(value)
  }
}
