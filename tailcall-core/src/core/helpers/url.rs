use crate::core::mustache::Mustache;
use crate::core::valid::{Valid, ValidationError};

pub fn to_url(url: &str) -> Valid<Mustache, String> {
    Valid::from(Mustache::parse(url).map_err(|e| ValidationError::new(e.to_string())))
}

#[cfg(test)]
mod tests {
    use super::to_url;

    #[test]
    fn parse_url() {
        use crate::core::mustache::Mustache;
        use crate::core::valid::Valid;

        let url = to_url("http://localhost:3000");

        assert_eq!(
            url,
            Valid::succeed(Mustache::parse("http://localhost:3000").unwrap())
        );
    }
}
