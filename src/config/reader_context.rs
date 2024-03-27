use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::path::PathString;
use crate::EnvIO;

pub struct ConfigReaderContext<'a> {
    pub env: Arc<dyn EnvIO>,
    pub vars: &'a BTreeMap<String, String>,
}

impl<'a> PathString for ConfigReaderContext<'a> {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
        if path.is_empty() {
            return None;
        }

        path.split_first()
            .and_then(|(head, tail)| match head.as_ref() {
                "vars" => self.vars.get(tail[0].as_ref()).map(|v| v.into()),
                "env" => self.env.get(tail[0].as_ref()).map(|v| v.into()),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestEnvIO;

    #[test]
    fn path_string() {
        let reader_context = ConfigReaderContext {
            env: Arc::new(TestEnvIO::from_iter([(
                "ENV_1".to_owned(),
                "ENV_VAL".to_owned(),
            )])),
            vars: &BTreeMap::from_iter([("VAR_1".to_owned(), "VAR_VAL".to_owned())]),
        };

        assert_eq!(
            reader_context.path_string(&["env", "ENV_1"]),
            Some("ENV_VAL".into())
        );
        assert_eq!(reader_context.path_string(&["env", "ENV_5"]), None);
        assert_eq!(
            reader_context.path_string(&["vars", "VAR_1"]),
            Some("VAR_VAL".into())
        );
        assert_eq!(reader_context.path_string(&["vars", "VAR_6"]), None);
        assert_eq!(reader_context.path_string(&["unknown", "unknown"]), None);
    }
}
