pub enum QueryEncoder {
    /// it encodes the query value in the form of id=1&id=2&id=3
    Repeated,
    /// it encodes the query in the form of q=value
    Simple,
}

impl QueryEncoder {
    pub fn detect(query: &str) -> Self {
        if query.starts_with('[') && query.ends_with(']') {
            QueryEncoder::Repeated
        } else {
            QueryEncoder::Simple
        }
    }

    pub fn encode<K, V>(&self, key: K, values: V) -> String
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        match self {
            QueryEncoder::Repeated => values
                .as_ref()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|v| format!("{}={}", key.as_ref(), v.trim()))
                .collect::<Vec<_>>()
                .join("&"),
            QueryEncoder::Simple => format!("{}={}", key.as_ref(), values.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_repeated() {
        let query = "[1,2,3]";
        let encoder = QueryEncoder::detect(query);
        assert!(matches!(encoder, QueryEncoder::Repeated));
    }

    #[test]
    fn test_detect_simple() {
        let query = "1";
        let encoder = QueryEncoder::detect(query);
        assert!(matches!(encoder, QueryEncoder::Simple));
    }

    #[test]
    fn test_encode_repeated() {
        let encoder = QueryEncoder::Repeated;
        let key = "id";
        let values = "[1,2,3]";
        let encoded = encoder.encode(key, values);
        assert_eq!(encoded, "id=1&id=2&id=3");
    }

    #[test]
    fn test_encode_simple() {
        let encoder = QueryEncoder::Simple;
        let key = "q";
        let values = "value";
        let encoded = encoder.encode(key, values);
        assert_eq!(encoded, "q=value");
    }

    #[test]
    fn test_encode_repeated_with_spaces() {
        let encoder = QueryEncoder::Repeated;
        let key = "id";
        let values = "[ 1 , 2 , 3 ]";
        let encoded = encoder.encode(key, values);
        assert_eq!(encoded, "id=1&id=2&id=3");
    }
}
