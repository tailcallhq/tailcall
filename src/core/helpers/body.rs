use serde_json::Value;
use tailcall_valid::Valid;

use crate::core::grpc::request_template::RequestBody;
use crate::core::mustache::Mustache;

pub fn to_body(body: Option<&Value>) -> Valid<Option<RequestBody>, String> {
    let Some(body) = body else {
        return Valid::succeed(None);
    };

    let mut req_body = RequestBody::default();

    let value = body.to_string();
    let mustache = Mustache::parse(&value);
    // TODO: req_body.mustache is always set making req_body.value useless
    req_body = req_body.mustache(Some(mustache));

    Valid::succeed(Some(req_body.value(value)))
}

#[cfg(test)]
mod tests {
    use tailcall_valid::Valid;

    use super::to_body;
    use crate::core::grpc::request_template::RequestBody;
    use crate::core::mustache::Mustache;

    #[test]
    fn no_body() {
        let result = to_body(None);

        assert_eq!(result, Valid::succeed(None));
    }

    #[test]
    fn body_parse_success() {
        let value = serde_json::Value::String("content".to_string());
        let result = to_body(Some(&value));

        assert_eq!(
            result,
            Valid::succeed(Some(RequestBody {
                mustache: Some(Mustache::parse(value.to_string().as_str())),
                value: value.to_string()
            }))
        );
    }
}
