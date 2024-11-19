use tailcall_jq::mustache::Mustache;
use tailcall_valid::Valid;

pub fn to_url(url: &str) -> Valid<Mustache, String> {
    Valid::succeed(Mustache::parse(url))
}

#[cfg(test)]
mod tests {
    use tailcall_jq::mustache::Mustache;

    use super::to_url;

    #[test]
    fn parse_url() {
        use tailcall_valid::Valid;

        let url = to_url("http://localhost:3000");

        assert_eq!(
            url,
            Valid::succeed(Mustache::parse("http://localhost:3000"))
        );
    }
}
