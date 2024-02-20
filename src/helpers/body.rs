use crate::mustache::Mustache;
use crate::valid::Valid;

pub fn to_body(body: Option<&str>) -> Valid<Option<Mustache>, String> {
    let Some(body) = body else {
        return Valid::succeed(None);
    };

    Valid::succeed(Some(Mustache::parse(body)))
}

#[cfg(test)]
mod tests {
    use super::to_body;
    use crate::mustache::Mustache;
    use crate::valid::Valid;

    #[test]
    fn no_body() {
        let result = to_body(None);

        assert_eq!(result, Valid::succeed(None));
    }

    #[test]
    fn body_parse_success() {
        let result = to_body(Some("content"));

        assert_eq!(result, Valid::succeed(Some(Mustache::parse("content"))));
    }
}
