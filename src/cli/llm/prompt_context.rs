use std::borrow::Cow;

use indexmap::IndexMap;

use crate::core::path::PathString;


pub struct PromptContext<'a> {
    vars: IndexMap<&'a str, String>,
}

impl<'a> PromptContext<'a> {
    pub fn new(vars: IndexMap<&'a str, String>) -> Self {
        PromptContext { vars }
    }
}

impl<'a> PathString for PromptContext<'a> {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
        if path.is_empty() {
            return None;
        }

        path.split_first().and_then(|(head, _)| {
            if let Some(value) = self.vars.get(head.as_ref()) {
                Some(value.into())
            } else {
                None
            }
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;

    #[test]
    fn test_path_string_with_existing_key() {
        let vars = indexmap! {
            "key1" => "value1".to_string(),
            "key2" => "value2".to_string(),
        };
        let context = PromptContext::new(vars);

        let path = vec!["key1"];
        let result = context.path_string(&path);

        assert_eq!(result, Some(Cow::Borrowed("value1")));
    }

    #[test]
    fn test_path_string_with_non_existing_key() {
        let vars = indexmap! {
            "key1" => "value1".to_string(),
            "key2" => "value2".to_string(),
        };
        let context = PromptContext::new(vars);

        let path = vec!["key3"];
        let result = context.path_string(&path);

        assert_eq!(result, None);
    }

    #[test]
    fn test_path_string_with_empty_path() {
        let vars = indexmap! {
            "key1" => "value1".to_string(),
        };
        let context = PromptContext::new(vars);

        let path: Vec<&str> = vec![];
        let result = context.path_string(&path);

        assert_eq!(result, None);
    }
}

