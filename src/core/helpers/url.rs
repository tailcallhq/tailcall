use crate::core::mustache::Mustache;
use crate::core::valid::Valid;

pub fn to_url(url: &str) -> Valid<Mustache, miette::MietteDiagnostic> {
    Valid::succeed(Mustache::parse(url))
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
            Valid::succeed(Mustache::parse("http://localhost:3000"))
        );
    }
}
