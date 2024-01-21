use async_graphql_value::indexmap::IndexMap;
use async_graphql_value::{ConstValue, Name as GQName, Number};
use mini_v8::{Error, FromValue, MiniV8, Result, ToValue, Value as JSValue};

#[derive(Debug)]
pub struct ValueWrapper(ConstValue);

impl From<ConstValue> for ValueWrapper {
  fn from(value: ConstValue) -> Self {
    Self(value)
  }
}

impl From<ValueWrapper> for ConstValue {
  fn from(value: ValueWrapper) -> Self {
    value.0
  }
}

impl FromValue for ValueWrapper {
  fn from_value(value: JSValue, _: &MiniV8) -> mini_v8::Result<Self> {
    let ag_value = match value {
      JSValue::Undefined | JSValue::Null => ConstValue::Null,
      JSValue::Boolean(v) => ConstValue::Boolean(v),
      JSValue::Number(v) => ConstValue::Number(Number::from_f64(v).ok_or(Error::FromJsConversionError {
        from: "number",
        to: "graphql number as it is out of supported range",
      })?),
      JSValue::Date(v) => ConstValue::Number(
        Number::from_f64(v)
          .ok_or(Error::FromJsConversionError { from: "Date", to: "graphql number as it is out of supported range" })?,
      ),
      JSValue::String(v) => ConstValue::String(v.to_string()),
      JSValue::Array(v) => {
        let list: Result<Vec<ConstValue>> = v.elements::<ValueWrapper>().map(|e| e.map(|v| v.into())).collect();

        ConstValue::List(list?)
      }
      JSValue::Function(_) => {
        log::warn!("Got a function from the js execution that couldn't be converted to GraphQL value");
        ConstValue::Null
      }
      JSValue::Object(v) => {
        let props: Result<Vec<(GQName, ConstValue)>> = v
          .properties::<String, ValueWrapper>(false)?
          .map(|e| e.map(|(k, v)| (GQName::new(k), v.into())))
          .collect();

        ConstValue::Object(IndexMap::from_iter(props?))
      }
    };

    Ok(ag_value.into())
  }
}

impl ToValue for ValueWrapper {
  fn to_value(self, mv8: &MiniV8) -> Result<JSValue> {
    let value = match self.0 {
      ConstValue::Null => JSValue::Null,
      ConstValue::Number(v) => JSValue::Number(v.as_f64().unwrap_or_default()),
      ConstValue::String(v) => JSValue::String(mv8.create_string(v.as_str())),
      ConstValue::Boolean(v) => JSValue::Boolean(v),
      ConstValue::Binary(_) => {
        return Err(Error::ToJsConversionError { from: "binary", to: "as it is not supported by js" })
      }
      ConstValue::Enum(_) => {
        return Err(Error::ToJsConversionError { from: "enum", to: "as it is not supported by js" })
      }
      ConstValue::List(v) => {
        let list = mv8.create_array();

        for e in v {
          list.push(ValueWrapper(e))?;
        }

        JSValue::Array(list)
      }
      ConstValue::Object(v) => {
        let object = mv8.create_object();
        for (k, v) in v {
          object.set::<&str, ValueWrapper>(k.as_str(), ValueWrapper(v))?;
        }

        JSValue::Object(object)
      }
    };

    Ok(value)
  }
}

#[cfg(test)]
mod tests {
  use async_graphql_value::indexmap::indexmap;
  use async_graphql_value::{ConstValue, Name, Number};
  use mini_v8::{FromValue, MiniV8, ToValue, Value as JSValue};
  use once_cell::sync::Lazy;

  use super::ValueWrapper;

  // TODO: const is actually wrong example to use in real app. But for tests that seems ok
  const V8: Lazy<MiniV8> = Lazy::new(|| MiniV8::new());

  #[test]
  fn null_conversion() {
    let initial = ConstValue::Null;
    let js_value = ValueWrapper::from(initial.clone()).to_value(&V8).unwrap();
    assert!(js_value.is_null());
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, initial);
  }

  #[test]
  fn undefined_conversion() {
    let initial = JSValue::Undefined;
    let gq_value: ConstValue = ValueWrapper::from_value(initial, &V8).unwrap().into();
    assert_eq!(gq_value, ConstValue::Null);
  }

  #[test]
  fn number_conversion() {
    let initial = ConstValue::Number(Number::from(5));
    let js_value = ValueWrapper::from(initial.clone()).to_value(&V8).unwrap();
    assert!(js_value.is_number());
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, ConstValue::Number(Number::from_f64(5.0).unwrap()));

    let initial = ConstValue::Number(Number::from(0));
    let js_value = ValueWrapper::from(initial.clone()).to_value(&V8).unwrap();
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, ConstValue::Number(Number::from_f64(0.0).unwrap()));

    let initial = ConstValue::Number(Number::from(-10));
    let js_value = ValueWrapper::from(initial.clone()).to_value(&V8).unwrap();
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, ConstValue::Number(Number::from_f64(-10.0).unwrap()));

    let initial = ConstValue::Number(Number::from_f64(0.25).unwrap());
    let js_value = ValueWrapper::from(initial.clone()).to_value(&V8).unwrap();
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, initial);
  }

  #[test]
  fn number_out_of_range() {
    let js_value = JSValue::Number(f64::NAN);
    let error = ValueWrapper::from_value(js_value, &V8).unwrap_err();

    assert_eq!(
      error.to_string(),
      "error converting JavaScript number to graphql number as it is out of supported range"
    );

    let js_value = JSValue::Number(f64::INFINITY);
    let error = ValueWrapper::from_value(js_value, &V8).unwrap_err();

    assert_eq!(
      error.to_string(),
      "error converting JavaScript number to graphql number as it is out of supported range"
    );
  }

  #[test]
  fn bool_conversion() {
    let initial = ConstValue::Boolean(true);
    let js_value = ValueWrapper::from(initial.clone()).to_value(&V8).unwrap();
    assert!(js_value.is_boolean());
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, initial);
  }

  #[test]
  fn date_conversion() {
    let date = 156156.584;
    let js_value = JSValue::Date(date);
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, ConstValue::Number(Number::from_f64(date).unwrap()));
  }

  #[test]
  fn date_out_off_range() {
    let date = f64::NAN;
    let js_value = JSValue::Date(date);
    let error = ValueWrapper::from_value(js_value, &V8).unwrap_err();
    assert_eq!(
      error.to_string(),
      "error converting JavaScript Date to graphql number as it is out of supported range"
    );
  }

  #[test]
  fn string_conversion() {
    let initial = ConstValue::String("str value".to_owned());
    let js_value = ValueWrapper::from(initial.clone()).to_value(&V8).unwrap();
    assert!(js_value.is_string());
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, initial);
  }

  #[test]
  fn array_conversion() {
    let initial = ConstValue::List(vec![
      ConstValue::String("str".to_string()),
      ConstValue::Null,
      ConstValue::Number(Number::from_f64(5.6).unwrap()),
      ConstValue::List(vec![ConstValue::Boolean(false), ConstValue::Null]),
    ]);
    let js_value = ValueWrapper::from(initial.clone()).to_value(&V8).unwrap();
    assert!(js_value.is_array());
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, initial);
  }

  #[test]
  fn object_conversion() {
    let nested_map = indexmap! {
      Name::new("c") => ConstValue::Number(Number::from_f64(3.2).unwrap()),
      Name::new("d") => ConstValue::Boolean(false),
    };
    let map = indexmap! {
      Name::new("a") => ConstValue::String("a str".to_owned()),
      Name::new("b") => ConstValue::Null,
      Name::new("nested") => ConstValue::Object(nested_map),
    };
    let initial = ConstValue::Object(map);
    let js_value = ValueWrapper::from(initial.clone()).to_value(&V8).unwrap();
    assert!(js_value.is_object());
    let gq_value: ConstValue = ValueWrapper::from_value(js_value, &V8).unwrap().into();
    assert_eq!(gq_value, initial);
  }

  #[test]
  fn graphql_enum() {
    let gq_value = ConstValue::Enum(Name::new("test"));
    let error = ValueWrapper::from(gq_value).to_value(&V8).unwrap_err();
    assert_eq!(
      error.to_string(),
      "error converting enum to JavaScript as it is not supported by js"
    )
  }
}
