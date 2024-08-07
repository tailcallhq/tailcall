use std::collections::HashMap;

use genai::chat::{ChatMessage, ChatRequest};
use genai::client::Client;
use serde::Deserialize;
use tokio::runtime::Runtime;

use crate::core::config::{Config, Type};
use crate::core::valid::Valid;
use crate::core::Transform;

const MODEL: &str = "gemini-1.5-flash-latest";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LLMResponse {
    #[allow(dead_code)]
    original_type_name: String,
    suggested_type_names: Vec<String>,
}

pub struct LLMTypeName {
    client: Client,
    used_type_names: Vec<String>,
    retry_count: u8,
}

impl Default for LLMTypeName {
    fn default() -> Self {
        Self {
            client: Default::default(),
            used_type_names: Default::default(),
            retry_count: 5,
        }
    }
}

impl LLMTypeName {
    // Given a prompt, suggests 5 names for it.
    async fn generate_type_names_inner(
        &self,
        prompt: &str,
        used_type_names: &str,
    ) -> Result<LLMResponse, anyhow::Error> {
        let base_system_message = ChatMessage::system("Given the GraphQL type definition below, provide a response in the form of a JSONP callback. The function should be named \"callback\" and should return JSON suggesting at least five suitable alternative names for the type. Each suggested name should be concise, preferably a single word, and capture the essence of the data it represents based on the roles and relationships implied by the field names. \n\n```graphql\ntype T {\n  name: String,\n  age: Int,\n  website: String\n}\n```\n\n**Expected JSONP Format:**\n\n```javascript\ncallback({\n  \"originalTypeName\": \"T\",\n  \"suggestedTypeNames\": [\"Person\",\"Profile\",\"Member\",\"Individual\",\"Contact\"\n  ]\n});\n```");
        let already_used_types = ChatMessage::system(format!(
            "We've already used following type names: {}",
            used_type_names
        ));

        let chat_req = ChatRequest::new(vec![
            base_system_message,
            already_used_types,
            ChatMessage::user(prompt),
        ]);

        let chat_res = self.client.exec_chat(MODEL, chat_req, None).await?;

        let response_text = chat_res.content.unwrap_or("NO ANSWER".to_string());

        // Extract the JSON from the JavaScript callback
        let start = response_text
            .find('{')
            .ok_or_else(|| anyhow::anyhow!("No JSON callback found."))?;
        let end = response_text
            .rfind('}')
            .ok_or_else(|| anyhow::anyhow!("No JSON callback found."))?;
        let json_str = &response_text[start..=end];

        let response: LLMResponse = serde_json::from_str(json_str)?;

        Ok(response)
    }

    // given type name and type, generated the 5 type names.
    async fn generate_type_name(
        &self,
        config: &Config,
        type_name: &str,
        type_: &Type,
        new_name_mappings: &HashMap<String, String>,
    ) -> anyhow::Result<Option<String>> {
        let mut t_config = Config::default();
        t_config.types.insert(type_name.to_string(), type_.clone());
        let type_sdl = t_config.to_sdl();

        let used_types: String = self.used_type_names.join(", ");

        let llm_response = self
            .generate_type_names_inner(&type_sdl, &used_types)
            .await?;

        let unique_type_name = llm_response
            .suggested_type_names
            .iter()
            .find(|suggested_type_name| {
                !config.types.contains_key(*suggested_type_name)
                    && !new_name_mappings.contains_key(*suggested_type_name)
            })
            .map(|suggested_type_name| suggested_type_name.to_owned());

        Ok(unique_type_name)
    }

    pub async fn generate(&mut self, mut config: Config) -> anyhow::Result<Config> {
        let mut new_name_mappings: HashMap<String, String> = HashMap::new();
        for (type_name, type_) in config.types.iter() {
            if config.is_root_operation_type(type_name) {
                continue;
            }

            // retries, if we find the type name is aleady used.
            for _ in 0..=self.retry_count {
                if let Some(unique_ty_name) = self
                    .generate_type_name(&config, type_name, type_, &new_name_mappings)
                    .await?
                {
                    new_name_mappings.insert(unique_ty_name.to_owned(), type_name.to_owned());
                    self.used_type_names.push(unique_ty_name);
                    break;
                }
            }
        }

        for (new_name, old_name) in new_name_mappings {
            if let Some(actual_ty) = config.types.get(&old_name) {
                config.types.insert(new_name.clone(), actual_ty.clone());
                config.types.remove(&old_name);

                // Replace all the instances of old name in config.
                for actual_type in config.types.values_mut() {
                    for actual_field in actual_type.fields.values_mut() {
                        if actual_field.type_of == old_name {
                            actual_field.type_of.clone_from(&new_name);
                        }
                    }
                }
            }
        }

        Ok(config)
    }
}

impl Transform for LLMTypeName {
    type Value = Config;
    type Error = String;

    fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        let a = std::thread::spawn(|| {
            Runtime::new().unwrap().block_on(async {
                let mut llm = LLMTypeName::default();
                llm.generate(value).await
            })
        })
        .join()
        .expect("Thread panicked");

        match a {
            Ok(b) => Valid::succeed(b),
            Err(e) => Valid::fail(e.to_string()),
        }
    }
}
