use regex::Regex;
use serde_json::Value;

pub fn detect_gql_data_type(value: &str) -> String {
    let trimmed_value = value.trim();

    if trimmed_value.parse::<i32>().is_ok() {
        "Int".to_string()
    } else if trimmed_value.parse::<f64>().is_ok() {
        "Float".to_string()
    } else if trimmed_value.parse::<bool>().is_ok() {
        "Boolean".to_string()
    } else if trimmed_value.contains(',') {
        let first_value = trimmed_value.split(',').next().unwrap_or("");
        detect_gql_data_type(first_value)
    } else {
        "String".to_string()
    }
}

pub fn is_valid_field_name(property_name: &str) -> bool {
    let gql_field_name_validator: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap();
    gql_field_name_validator.is_match(property_name)
}

pub fn to_gql_type(value: &Value) -> String {
    match value {
        Value::Null => "Empty",
        Value::Bool(_) => "Boolean",
        Value::Number(_) => "Int",
        Value::String(_) => "String",
        Value::Array(_) => "List",
        Value::Object(_) => "Object",
    }
    .to_string()
}

pub fn is_primitive(value: &Value) -> bool {
    let value_type = to_gql_type(value);
    value_type != "List" && value_type != "Object"
}

#[cfg(test)]
mod test {
    use serde_json::{json, Value};

    use super::{detect_gql_data_type, is_primitive, is_valid_field_name, to_gql_type};
    #[test]
    fn test_detect_gql_data_type() {
        assert_eq!(detect_gql_data_type("42"), "Int");
        assert_eq!(detect_gql_data_type("3.14"), "Float");
        assert_eq!(detect_gql_data_type("true"), "Boolean");
        assert_eq!(detect_gql_data_type("false"), "Boolean");
        assert_eq!(detect_gql_data_type("1,2,3"), "Int");
        assert_eq!(detect_gql_data_type("a,b,c"), "String");
        assert_eq!(detect_gql_data_type("hello"), "String");
    }

    #[test]
    fn test_is_valid_field_name() {
        assert!(!is_valid_field_name("first name"));
        assert!(!is_valid_field_name("10"));
        assert!(!is_valid_field_name("$10"));
        assert!(!is_valid_field_name("#10"));

        assert!(is_valid_field_name("firstName"));
        assert!(is_valid_field_name("lastname"));
        assert!(is_valid_field_name("lastname1"));
        assert!(is_valid_field_name("lastname2"));
        assert!(is_valid_field_name("last_name"));
    }

    #[test]
    fn test_to_gql_type() {
        assert_eq!(to_gql_type(&json!("Testing")), "String");
        assert_eq!(to_gql_type(&json!(12)), "Int");
        assert_eq!(to_gql_type(&json!(12.3)), "Int");
        assert_eq!(to_gql_type(&json!(-12)), "Int");
        assert_eq!(to_gql_type(&json!(-12.2)), "Int");
        assert_eq!(to_gql_type(&json!(true)), "Boolean");
        assert_eq!(to_gql_type(&json!(false)), "Boolean");
        assert_eq!(to_gql_type(&json!([1, 2, 3])), "List");
        assert_eq!(to_gql_type(&json!({"name":"test", "age": 12})), "Object");
        assert_eq!(to_gql_type(&Value::Null), "Empty");

        assert_eq!(to_gql_type(&json!([])), "List");
        assert_eq!(to_gql_type(&json!({})), "Object");
    }

    #[test]
    fn test_is_primitive() {
        assert!(is_primitive(&json!("Testing")));
        assert!(is_primitive(&json!(12)));
        assert!(is_primitive(&json!(12.3)));
        assert!(is_primitive(&json!(-12)));
        assert!(is_primitive(&json!(-12.2)));
        assert!(is_primitive(&json!(true)));
        assert!(is_primitive(&json!(false)));

        assert!(!is_primitive(&json!([1, 2, 3])));
        assert!(!is_primitive(&json!({"name":"test", "age": 12})));
    }
}
