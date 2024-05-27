use async_graphql_value::ConstValue;
use schemars::schema::Schema;
use schemars::{schema_for, JsonSchema};

/// Represents unsigned integer type 64 bit size as string
#[derive(JsonSchema, Default)]
pub struct Int64Str(pub i64);

impl super::Scalar for Int64Str {
    // TODO: for now Str scalar only validates that input is string
    // because the main case to solve right now is to provide support for grpc
    // and for protobuf 64bit integers are already resolved as strings by default
    // see `protobuf::tests::scalars_proto_file`
    // but for general case we may consider to automatically convert such integers
    // to string in case they are passed as numbers. That will help to prevent
    // errors on clients that couldn't parse these integers lossless
    // i.e. JS and its JSON.parse
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let ConstValue::String(n) = value {
                n.parse::<i64>().is_ok()
            } else {
                false
            }
        }
    }
    fn schema(&self) -> Schema {
        schema_for!(Self).schema.into()
    }
}

#[cfg(test)]
mod test {
    use async_graphql_value::ConstValue;
    use serde_json::Number;

    use crate::core::scalar::{Int64Str, Scalar};
    use crate::{test_scalar_invalid, test_scalar_valid};

    test_scalar_valid! {
        Int64Str,
        ConstValue::String("100".to_string()),
        ConstValue::String((2 * i32::MAX as u64).to_string()),
        ConstValue::String("-15".to_string())
    }

    test_scalar_invalid! {
        Int64Str,
        ConstValue::Null,
        ConstValue::Number(Number::from(45648846465i64)),
        ConstValue::Number(
            Number::from_f64(1.25).unwrap()
        )
    }
}
