use wasm_bindgen::JsValue;

pub struct JsVal(JsValue);

impl From<JsVal> for JsValue {
    fn from(value: JsVal) -> Self {
        value.0
    }
}

impl From<JsValue> for JsVal {
    fn from(value: JsValue) -> Self {
        JsVal(value)
    }
}

impl From<String> for JsVal {
    fn from(value: String) -> Self {
        JsVal(JsValue::from_str(value.as_str()))
    }
}

impl From<anyhow::Error> for JsVal {
    fn from(value: anyhow::Error) -> Self {
        JsVal(JsValue::from_str(value.to_string().as_str()))
    }
}
