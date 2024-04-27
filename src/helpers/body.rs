use crate::mustache::Mustache;
use crate::valid::{Valid, ValidationError};

pub fn to_body(body: Option<&str>) -> Valid<Option<Mustache>, String> {
    let Some(body) = body else {
        return Valid::succeed(None);
    };

    Valid::from(
        Mustache::parse(body)
            .map(Some)
            .map_err(|e| ValidationError::new(e.to_string())),
    )
}

#[cfg(test)]
mod tests {
    use super::to_body;
    use crate::mustache::Mustache;
    use crate::valid::{Valid, Validator};

    #[test]
    fn no_body() {
        let result = to_body(None).map(|v| v.map(|v| v.to_string()));

        assert_eq!(result, Valid::succeed(None));
    }

    #[test]
    fn body_parse_success() {
        let result = to_body(Some("content"));

        assert_eq!(
            result.map(|v| v.map(|v| v.to_string())),
            Valid::succeed(Some(Mustache::parse("content").unwrap().to_string()))
        );
    }
}
