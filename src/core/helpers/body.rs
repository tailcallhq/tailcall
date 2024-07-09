use serde_json::Value;

use crate::core::grpc::request_template::RequestBody;
use crate::core::mustache::Mustache;
use crate::core::valid::Valid;

pub fn to_body(body: Option<&Value>) -> Valid<Option<RequestBody>, String> {
    let Some(body) = body else {
        return Valid::succeed(None);
    };

    match body {
        Value::String(body) => {
            if let Ok(mustache) = Mustache::parse(body) {
                Valid::succeed(Some(RequestBody {
                    mustache: Some(mustache),
                    value: body.to_string(),
                }))
            } else {
                Valid::succeed(Some(RequestBody {
                    value: body.to_string(),
                    ..Default::default()
                }))
            }
        }
        value => Valid::succeed(Some(RequestBody {
            value: value.to_string(),
            ..Default::default()
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::to_body;
    use crate::core::grpc::request_template::RequestBody;
    use crate::core::mustache::Mustache;
    use crate::core::valid::Valid;

    #[test]
    fn no_body() {
        let result = to_body(None);

        assert_eq!(result, Valid::succeed(None));
    }

    #[test]
    fn body_parse_success() {
        let result = to_body(Some(&serde_json::Value::String("content".to_string())));

        assert_eq!(
            result,
            Valid::succeed(Some(RequestBody {
                mustache: Some(Mustache::parse("content").unwrap()),
                value: "content".to_string()
            }))
        );
    }
}
