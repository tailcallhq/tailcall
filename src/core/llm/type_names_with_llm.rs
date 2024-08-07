use std::collections::HashMap;

use serde::Deserialize;

use crate::core::config::Config;

use super::engine::Engine;

const MODEL: &str = "gemini-1.5-flash-latest";
const PROMPT: &str = include_str!("prompt.md");

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LLMResponse {
    #[allow(dead_code)]
    original_type_name: String,
    suggested_type_names: Vec<String>,
}

pub struct LLMTypeName {
    retry_count: u8,
}

impl Default for LLMTypeName {
    fn default() -> Self {
        Self { retry_count: 5 }
    }
}

impl LLMTypeName {
    pub async fn generate(&mut self, config: &Config) -> anyhow::Result<HashMap<String, String>> {
        let engine = Engine::<serde_json::Value>::new("$$START$$".into(), "$$END$$".into())
            .system_prompt(Some(PROMPT.into()));

        let mut new_name_mappings: HashMap<String, String> = HashMap::new();
        for (type_name, type_) in config.types.iter() {
            if config.is_root_operation_type(type_name) {
                // Ignore the root types as their names are already given by the user.
                continue;
            }

            // convert type to sdl format.
            let mut t_config = Config::default();
            t_config.types.insert(type_name.to_string(), type_.clone());
            let type_sdl = t_config.to_sdl();

            // Retry logic to handle network or other errors
            for _ in 0..=self.retry_count {
                match engine.prompt(&type_sdl).await {
                    Ok(response) => {
                        let llm_response: LLMResponse = serde_json::from_value(response)?;
                        for name in llm_response.suggested_type_names {
                            if config.types.contains_key(&name)
                                || new_name_mappings.contains_key(&name)
                            {
                                continue;
                            }
                            new_name_mappings.insert(name, type_name.to_owned());
                            break;
                        }
                    }
                    Err(e) => {
                        todo!()
                    }
                }
            }
        }

        Ok(new_name_mappings.into_iter().map(|(k, v)| (v, k)).collect())
    }
}
