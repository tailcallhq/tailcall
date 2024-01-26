use std::time::Duration;

use mini_v8::{Error, MiniV8, Script};

// TODO: Performance optimizations
// This function can be optimized quite heavily
pub fn execute_js(
    script: &str,
    input: async_graphql::Value,
    timeout: Option<Duration>,
) -> Result<async_graphql::Value, Error> {
    let mv8 = MiniV8::new();
    let source = create_source(script, input);
    let value: String = mv8.eval(Script { source, timeout, origin: None })?;
    let json = serde_json::from_str(value.as_str()).unwrap();
    Ok(json)
}

fn create_source(script: &str, input: async_graphql::Value) -> String {
    let template = "(function (ctx) {return JSON.stringify(--SCRIPT--)} )(--INPUT--);";

    template
        .replace("--SCRIPT--", script)
        .replace("--INPUT--", input.to_string().as_str())
}

#[cfg(test)]
#[test]
fn test_json() {
    let json = r#"
    {
        "name": "John Doe",
        "age": 43
    }
    "#;
    let json = serde_json::from_str(json).unwrap();
    let script = "ctx.name";
    let actual = execute_js(script, json, Some(Duration::from_secs(1))).unwrap();
    let expected = async_graphql::Value::from("John Doe");

    assert_eq!(actual, expected);
}

#[cfg(test)]
#[test]
fn test_timeout() {
    let script = "(function () {while(true) {};})()";
    let actual = execute_js(
        script,
        async_graphql::Value::Null,
        Some(Duration::from_millis(10)),
    );
    match actual {
        Err(Error::Timeout) => {} // Success case
        _ => panic!("Expected a Timeout error, but got {:?}", actual), // Failure case
    }
}
