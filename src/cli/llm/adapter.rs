use crate::core::runtime::TargetRuntime;

pub struct Adapter {}

impl Adapter {
    pub fn config(rt: &TargetRuntime) -> genai::adapter::AdapterConfig {
        let env_key = rt.env.get("TAILCALL_SECRET");
        let mut config = genai::adapter::AdapterConfig::default();
        if let Some(key) = env_key {
            config = config.with_auth_env_name(key);
        }
        config
    }
}
