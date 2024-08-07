use std::collections::HashMap;

use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use serde::{Deserialize, Serialize};

use super::Wizard;
use super::{Error, Result};
use crate::core::config::Config;

const MODEL: &str = "gemini-1.5-flash-latest";
const START_MARKER: &str = "$$$__START__$$$";
const END_MARKER: &str = "$$$__END__$$$";

#[derive(Default)]
pub struct InferTypeName {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Answer {
    suggestions: Vec<String>,
}

impl TryFrom<ChatResponse> for Answer {
    type Error = Error;

    fn try_from(response: ChatResponse) -> Result<Self> {
        let content = response.content.ok_or(Error::EmptyResponse)?;
        let start = content
            .find(START_MARKER)
            .ok_or(Error::MissingMarker(START_MARKER.to_string()))?
            + START_MARKER.len();
        let end = content
            .rfind(&END_MARKER)
            .ok_or(Error::MissingMarker(END_MARKER.to_string()))?;
        let json = &content[start..end];
        Ok(serde_json::from_str(json)?)
    }
}

#[derive(Clone, Serialize)]
struct Question {
    fields: Vec<(String, String)>,
}

impl TryInto<ChatRequest> for Question {
    type Error = Error;

    fn try_into(self) -> Result<ChatRequest> {
        let content = serde_json::to_string(&self)?;
        let input = serde_json::to_string_pretty(&Question {
            fields: vec![
                ("id".to_string(), "String".to_string()),
                ("name".to_string(), "String".to_string()),
                ("age".to_string(), "Int".to_string()),
            ],
        })?;

        let output = serde_json::to_string_pretty(&Answer {
            suggestions: vec![
                "Person".into(),
                "Profile".into(),
                "Member".into(),
                "Individual".into(),
                "Contact".into(),
            ],
        })?;

        Ok(ChatRequest::new(vec![
            ChatMessage::system(
                "Given the sample schema of a GraphQL type suggest 5 meaningful names for it.",
            ),
            ChatMessage::system("The name should be concise and preferably a single word"),
            ChatMessage::system("Example Input:"),
            ChatMessage::system(input),
            ChatMessage::system("Example Output:"),
            ChatMessage::system(output),
            ChatMessage::system("Ensure the output is in valid JSON format".to_string()),
            ChatMessage::system(format!(
                "Ensure output json starts with the marker {}",
                START_MARKER
            )),
            ChatMessage::system(format!(
                "Ensure output json ends with the marker {}",
                END_MARKER
            )),
            ChatMessage::user(content),
        ]))
    }
}

impl InferTypeName {
    pub async fn generate(&mut self, config: &Config) -> Result<HashMap<String, String>> {
        let engine: Wizard<Question, Answer> = Wizard::new(MODEL.to_string());

        let mut new_name_mappings: HashMap<String, String> = HashMap::new();
        for (type_name, type_) in config.types.iter() {
            if config.is_root_operation_type(type_name) {
                // Ignore the root types as their names are already given by the user.
                continue;
            }

            // convert type to sdl format.
            let question = Question {
                fields: type_
                    .fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.type_of.clone()))
                    .collect(),
            };

            let answer = engine.ask(question).await?;
            for name in answer.suggestions {
                if config.types.contains_key(&name) || new_name_mappings.contains_key(&name) {
                    continue;
                }
                new_name_mappings.insert(name, type_name.to_owned());
                break;
            }
        }

        Ok(new_name_mappings.into_iter().map(|(k, v)| (v, k)).collect())
    }
}
