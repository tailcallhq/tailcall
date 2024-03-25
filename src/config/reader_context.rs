use std::collections::BTreeMap;
use std::sync::Arc;

use crate::path_value::PathValue;
use crate::EnvIO;

pub struct ConfigReaderContext<'a> {
    pub env: Arc<dyn EnvIO>,
    pub vars: &'a BTreeMap<String, String>,
}

impl<'a> PathValue for ConfigReaderContext<'a> {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        path.split_first()
            .and_then(|(head, tail)| match head.as_ref() {
                "vars" => self.vars.get_path_value(tail),
                "env" => self.env.get_path_value(tail),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestEnvIO;

    #[test]
    fn path_value() {
        let reader_context = ConfigReaderContext {
            env: Arc::new(TestEnvIO::from_iter([(
                "ENV_1".to_owned(),
                "ENV_VAL".to_owned(),
            )])),
            vars: &BTreeMap::from_iter([("VAR_1".to_owned(), "VAR_VAL".to_owned())]),
        };

        assert_eq!(
            reader_context.get_path_value(&["env", "ENV_1"]),
            Some("ENV_VAL".into())
        );
        assert_eq!(reader_context.get_path_value(&["env", "ENV_5"]), None);
        assert_eq!(
            reader_context.get_path_value(&["vars", "VAR_1"]),
            Some("VAR_VAL".into())
        );
        assert_eq!(reader_context.get_path_value(&["vars", "VAR_6"]), None);
        assert_eq!(reader_context.get_path_value(&["unknown", "unknown"]), None);
    }
}
