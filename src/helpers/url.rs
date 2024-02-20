use crate::mustache::Mustache;
use crate::valid::Valid;

pub fn to_url(url: &str) -> Valid<Mustache, String> {
    Valid::succeed(Mustache::parse(url))
}

#[cfg(test)]
mod tests {
    use super::to_url;

    #[test]
    fn parse_url() {
        use crate::mustache::Mustache;
        use crate::valid::Valid;

        let url = to_url("http://localhost:3000");

        assert_eq!(
            url,
            Valid::succeed(Mustache::parse("http://localhost:3000"))
        );
    }
}
