pub enum QueryEncoder {
    /// it encodes the query value in the form of id=1&id=2&id=3
    List,
    /// it encodes the query in the form of q=value
    Single,
}

impl QueryEncoder {
    pub fn detect(query: &str) -> Self {
        if query.starts_with('[') && query.ends_with(']') {
            QueryEncoder::List
        } else {
            QueryEncoder::Single
        }
    }

    pub fn encode<K, V>(&self, key: K, values: V) -> String
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        match self {
            QueryEncoder::List => values
                .as_ref()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|v| format!("{}={}", key.as_ref(), v.trim()))
                .collect::<Vec<_>>()
                .join("&"),
            QueryEncoder::Single => format!("{}={}", key.as_ref(), values.as_ref()),
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
        assert!(matches!(encoder, QueryEncoder::List));
    }

    #[test]
    fn test_detect_simple() {
        let query = "1";
        let encoder = QueryEncoder::detect(query);
        assert!(matches!(encoder, QueryEncoder::Single));
    }

    #[test]
    fn test_encode_repeated() {
        let encoder = QueryEncoder::List;
        let key = "id";
        let values = "[1,2,3]";
        let encoded = encoder.encode(key, values);
        assert_eq!(encoded, "id=1&id=2&id=3");
    }

    #[test]
    fn test_encode_simple() {
        let encoder = QueryEncoder::Single;
        let key = "q";
        let values = "value";
        let encoded = encoder.encode(key, values);
        assert_eq!(encoded, "q=value");
    }

    #[test]
    fn test_encode_repeated_with_spaces() {
        let encoder = QueryEncoder::List;
        let key = "id";
        let values = "[ 1 , 2 , 3 ]";
        let encoded = encoder.encode(key, values);
        assert_eq!(encoded, "id=1&id=2&id=3");
    }
}
