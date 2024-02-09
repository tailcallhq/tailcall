use std::collections::HashMap;

use crate::EnvIO;

#[derive(Clone)]
pub struct TestEnvIO {
    vars: HashMap<String, String>,
}

impl EnvIO for TestEnvIO {
    fn get(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}

impl TestEnvIO {
    pub fn init() -> Self {
        Self { vars: std::env::vars().collect() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_with_env_vars() {
        let test_env = TestEnvIO::init();
        assert!(!test_env.vars.is_empty());
    }

    #[test]
    fn test_get_existing_var() {
        let mut vars = HashMap::new();
        vars.insert("EXISTING_VAR".to_string(), "value".to_string());
        let test_env = TestEnvIO { vars };
        let result = test_env.get("EXISTING_VAR");
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_get_non_existing_var() {
        let vars = HashMap::new();
        let test_env = TestEnvIO { vars };
        let result = test_env.get("NON_EXISTING_VAR");
        assert_eq!(result, None);
    }
}
