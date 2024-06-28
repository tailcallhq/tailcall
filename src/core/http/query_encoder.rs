pub enum QueryEncoder {
    /// it encodes the query value in the form of key=value1&key=value2&key=value3
    List,
    /// it encodes the query in the form of key=value
    Single,
}

impl Default for QueryEncoder {
    fn default() -> Self {
        QueryEncoder::Single
    }
}

impl QueryEncoder {
    pub fn detect(list_type: bool) -> Self {
        if list_type {
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
    fn test_detect_list_variant() {
        let encoder = QueryEncoder::detect(true);
        assert!(matches!(encoder, QueryEncoder::List));
    }

    #[test]
    fn test_detect_single_variant() {
        let encoder = QueryEncoder::detect(false);
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
