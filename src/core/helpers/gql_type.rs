use regex::Regex;
use serde_json::Value;

pub fn detect_gql_data_type(value: &str) -> String {
    if let Ok(_) = value.parse::<i32>() {
        return "Int".to_string();
    }
    if let Ok(_) = value.parse::<f64>() {
        return "Float".to_string();
    }
    if let Ok(_) = value.parse::<bool>() {
        return "Boolean".to_string();
    }
    if value.contains(',') {
        return "List".to_string();
    }
    "String".to_string()
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

pub fn is_list_type(value: &Value) -> bool {
    to_gql_type(value) == "List"
}

pub fn is_primitive(value: &Value) -> bool {
    let value_type = to_gql_type(value);
    value_type != "List" && value_type != "Object"
}


#[cfg(test)]
mod test {
    use serde_json::{json, Value};
    use super::{detect_gql_data_type, is_valid_field_name, to_gql_type, is_list_type, is_primitive};
    #[test]
    fn test_detect_gql_data_type(){
        assert_eq!(detect_gql_data_type("42"), "Int");
        assert_eq!(detect_gql_data_type("3.14"), "Float");
        assert_eq!(detect_gql_data_type("true"), "Boolean");
        assert_eq!(detect_gql_data_type("false"), "Boolean");
        assert_eq!(detect_gql_data_type("1,2,3"), "List");
        assert_eq!(detect_gql_data_type("hello"), "String");
    }

    #[test]
    fn test_is_valid_field_name(){
        assert_eq!(is_valid_field_name("first name"),false);
        assert_eq!(is_valid_field_name("10"),false);
        assert_eq!(is_valid_field_name("$10"),false);
        assert_eq!(is_valid_field_name("#10"),false);


        assert_eq!(is_valid_field_name("firstName"),true);
        assert_eq!(is_valid_field_name("lastname"),true);
        assert_eq!(is_valid_field_name("lastname1"),true);
        assert_eq!(is_valid_field_name("lastname2"),true);
        assert_eq!(is_valid_field_name("last_name"),true);
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

        assert_eq!(to_gql_type(&json!([])), "List");
        assert_eq!(to_gql_type(&json!({})), "Object");
    }


    #[test]
    fn test_is_list_type() {
        assert_eq!(is_list_type(&json!("Testing")), false);
        assert_eq!(is_list_type(&json!(12)), false);
        assert_eq!(is_list_type(&json!(12.3)), false);
        assert_eq!(is_list_type(&json!(-12)), false);
        assert_eq!(is_list_type(&json!(-12.2)), false);
        assert_eq!(is_list_type(&json!(true)), false);
        assert_eq!(is_list_type(&json!(false)), false);
        assert_eq!(is_list_type(&json!({"name":"test", "age": 12})), false);

        assert_eq!(is_list_type(&json!([1, 2, 3])), true);
    }

    #[test]
    fn test_is_primitive() {
        assert_eq!(is_primitive(&json!("Testing")), true);
        assert_eq!(is_primitive(&json!(12)), true);
        assert_eq!(is_primitive(&json!(12.3)), true);
        assert_eq!(is_primitive(&json!(-12)), true);
        assert_eq!(is_primitive(&json!(-12.2)), true);
        assert_eq!(is_primitive(&json!(true)), true);
        assert_eq!(is_primitive(&json!(false)), true);

        assert_eq!(is_primitive(&json!([1, 2, 3])), false);
        assert_eq!(is_primitive(&json!({"name":"test", "age": 12})), false);
    }

}