use std::borrow::Cow;

use tailcall_hasher::TailcallHashMap;

use crate::core::EnvIO;

#[derive(Clone)]
pub struct EnvNative {
    vars: TailcallHashMap<String, String>,
}

impl EnvIO for EnvNative {
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        self.vars.get(key).map(Cow::from)
    }
}

impl EnvNative {
    pub fn init() -> Self {
        Self { vars: std::env::vars().collect() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_with_env_vars() {
        let test_env = EnvNative::init();
        assert!(!test_env.vars.is_empty());
    }

    #[test]
    fn test_get_existing_var() {
        let mut vars = TailcallHashMap::default();
        vars.insert("EXISTING_VAR".to_string(), "value".to_string());
        let test_env = EnvNative { vars };
        let result = test_env.get("EXISTING_VAR");
        assert_eq!(result, Some("value".into()));
    }

    #[test]
    fn test_get_non_existing_var() {
        let vars = TailcallHashMap::default();
        let test_env = EnvNative { vars };
        let result = test_env.get("NON_EXISTING_VAR");
        assert_eq!(result, None);
    }
}
