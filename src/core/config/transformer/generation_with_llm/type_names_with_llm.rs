use std::collections::HashMap;

use genai::chat::{ChatMessage, ChatRequest};
use genai::client::Client;
use serde::Deserialize;

use crate::core::config::{Config, Type};

const MODEL: &str = "gemini-1.5-flash-latest";
const PROMPT: &str = "Given the GraphQL type definition below, provide a response in the form of a JSONP callback. The function should be named \"callback\" and should return JSON suggesting at least ten suitable alternative names for the type. Each suggested name should be concise, preferably a single word, and capture the essence of the data it represents based on the roles and relationships implied by the field names. \n\n```graphql\ntype T {\n  name: String,\n  age: Int,\n  website: String\n}\n```\n\n**Expected JSONP Format:**\n\n```javascript\ncallback({\n  \"originalTypeName\": \"T\",\n  \"suggestedTypeNames\": [\"Person\",\"Profile\",\"Member\",\"Individual\",\"Contact\"\n  ]\n});\n```";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LLMResponse {
    #[allow(dead_code)]
    original_type_name: String,
    suggested_type_names: Vec<String>,
}

pub struct LLMTypeName {
    client: Client,
    retry_count: u8,
}

impl Default for LLMTypeName {
    fn default() -> Self {
        Self { client: Default::default(), retry_count: 5 }
    }
}

impl LLMTypeName {
    // Given a prompt, suggests 5 names for it.
    async fn generate_type_names_inner(
        &self,
        prompt: &str,
        used_type_names: &str,
    ) -> Result<LLMResponse, anyhow::Error> {
        let base_system_message = ChatMessage::system(PROMPT);
        let already_used_types = ChatMessage::system(format!(
            "We've already used following type names: {}",
            used_type_names
        ));

        let chat_req = ChatRequest::new(vec![
            base_system_message,
            already_used_types,
            ChatMessage::user(prompt),
        ]);

        for attempt in 0..=self.retry_count {
            match self.client.exec_chat(MODEL, chat_req.clone(), None).await {
                Ok(chat_res) => {
                    let response_text = chat_res.content.unwrap_or_else(|| "NO ANSWER".to_string());

                    // Extract the JSON from the JavaScript callback
                    let start = response_text
                        .find('{')
                        .ok_or_else(|| anyhow::anyhow!("No JSON callback found."))?;
                    let end = response_text
                        .rfind('}')
                        .ok_or_else(|| anyhow::anyhow!("No JSON callback found."))?;
                    let json_str = &response_text[start..=end];

                    let response: LLMResponse = serde_json::from_str(json_str)?;

                    return Ok(response);
                }
                Err(e) if attempt < self.retry_count => {
                    println!(
                        "Request failed (attempt {} of {}): {:?}",
                        attempt + 1,
                        self.retry_count,
                        e
                    );
                }
                Err(e) => return Err(e.into()),
            }
        }

        Err(anyhow::anyhow!(
            "Failed to get a valid response after {} attempts.",
            self.retry_count
        ))
    }

    // Given type name and type, generate the 5 type names.
    #[allow(clippy::too_many_arguments)]
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

        let used_types: String = new_name_mappings
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");

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

    pub async fn generate(&mut self, config: &Config) -> anyhow::Result<HashMap<String, String>> {
        let mut new_name_mappings: HashMap<String, String> = HashMap::new();
        for (type_name, type_) in config.types.iter() {
            if config.is_root_operation_type(type_name) {
                // Ignore the root types as their names are already given by the user.
                continue;
            }

            // Retry logic to handle network or other errors
            for _ in 0..=self.retry_count {
                match self
                    .generate_type_name(config, type_name, type_, &new_name_mappings)
                    .await
                {
                    Ok(Some(unique_ty_name)) => {
                        new_name_mappings.insert(unique_ty_name.to_owned(), type_name.to_owned());
                        break;
                    }
                    Ok(None) => {
                        eprintln!("No unique type name found for type '{}'", type_name);
                        continue;
                    }
                    Err(e) => {
                        eprintln!(
                            "Error generating type name for type '{}': {:?}",
                            type_name, e
                        );
                    }
                }
            }
        }

        Ok(new_name_mappings.into_iter().map(|(k, v)| (v, k)).collect())
    }
}
