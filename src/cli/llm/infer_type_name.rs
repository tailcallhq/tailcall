use std::collections::HashMap;

use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Error, Result, Wizard};
use crate::core::config::{Config, LinkType};
use crate::core::Mustache;

const BASE_TEMPLATE: &str = include_str!("prompts/infer_type_name.md");

pub struct InferTypeName {
    wizard: Wizard<Question, Answer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Answer {
    suggestions: Vec<String>,
}

impl TryFrom<ChatResponse> for Answer {
    type Error = Error;

    fn try_from(response: ChatResponse) -> Result<Self> {
        let message_content = response.content.ok_or(Error::EmptyResponse)?;
        let text_content = message_content.text_as_str().ok_or(Error::EmptyResponse)?;
        Ok(serde_json::from_str(text_content)?)
    }
}

#[derive(Clone, Serialize)]
struct Question {
    #[serde(skip_serializing_if = "IndexSet::is_empty")]
    used_types: IndexSet<String>,
    fields: Vec<(String, String)>,
}

impl TryInto<ChatRequest> for Question {
    type Error = Error;

    fn try_into(self) -> Result<ChatRequest> {
        let input = serde_json::to_string_pretty(&Question {
            used_types: IndexSet::default(),
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

        let template = Mustache::parse(BASE_TEMPLATE);

        let context = json!({
            "used_types": self.used_types,
            "input": input,
            "output": output,
        });

        let rendered_prompt = template.render(&context);

        Ok(ChatRequest::new(vec![
            ChatMessage::system(rendered_prompt),
            ChatMessage::user(serde_json::to_string(&json!({
                "fields": &self.fields,
            }))?),
        ]))
    }
}

impl InferTypeName {
    pub fn new(model: String, secret: Option<String>) -> InferTypeName {
        Self { wizard: Wizard::new(model, secret) }
    }

    /// Determines if a type name is automatically generated.
    ///
    /// A type name is considered automatically generated if:
    /// - It contain "__"
    /// - It starts with 'T' or 'M' and All characters after the first one are
    ///   ASCII digits
    fn is_auto_generated(type_name: &str, is_grpc: bool) -> bool {
        if is_grpc && type_name.contains("__") {
            return true;
        }

        type_name.starts_with(['T', 'M'])
            && type_name.len() > 1
            && type_name[1..].chars().all(|c| c.is_ascii_digit())
    }

    pub async fn generate(&mut self, config: &Config) -> Result<HashMap<String, String>> {
        let mut new_name_mappings: HashMap<String, String> = HashMap::new();

        let is_grpc = config
            .links
            .iter()
            .any(|link| link.type_of == LinkType::Protobuf);

        // Filter out root operation types and types with non-auto-generated names
        let types_to_be_processed = config
            .types
            .iter()
            .filter(|(type_name, _)| {
                !config.is_root_operation_type(type_name)
                    && Self::is_auto_generated(type_name, is_grpc)
            })
            .collect::<Vec<_>>();

        let mut used_type_names = config
            .types
            .iter()
            .filter(|(ty_name, _)| !Self::is_auto_generated(ty_name, is_grpc))
            .map(|(ty_name, _)| ty_name.to_owned())
            .collect::<IndexSet<_>>();

        let total = types_to_be_processed.len();
        for (i, (type_name, type_)) in types_to_be_processed.into_iter().enumerate() {
            // convert type to sdl format.
            let question = Question {
                used_types: used_type_names.clone(),
                fields: type_
                    .fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.type_of.name().to_owned()))
                    .collect(),
            };

            let mut delay = 3;
            loop {
                let answer = self.wizard.ask(question.clone()).await;
                match answer {
                    Ok(answer) => {
                        let name = &answer.suggestions.join(", ");
                        for name in answer.suggestions {
                            if config.types.contains_key(&name)
                                || new_name_mappings.contains_key(&name)
                            {
                                continue;
                            }
                            used_type_names.insert(name.clone());
                            new_name_mappings.insert(name, type_name.to_owned());
                            break;
                        }
                        tracing::info!(
                            "Suggestions for {}: [{}] - {}/{}",
                            type_name,
                            name,
                            i + 1,
                            total
                        );

                        // TODO: case where suggested names are already used, then extend the base
                        // question with `suggest different names, we have already used following
                        // names: [names list]`
                        break;
                    }
                    Err(e) => {
                        // TODO: log errors after certain number of retries.
                        if let Error::GenAI(_) = e {
                            // TODO: retry only when it's required.
                            tracing::warn!(
                                "Unable to retrieve a name for the type '{}'. Retrying in {}s",
                                type_name,
                                delay
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                            delay *= std::cmp::min(delay * 2, 60);
                        }
                    }
                }
            }
        }

        Ok(new_name_mappings.into_iter().map(|(k, v)| (v, k)).collect())
    }
}

#[cfg(test)]
mod test {
    use genai::chat::{ChatRequest, ChatResponse, MessageContent};
    use indexmap::indexset;

    use super::{Answer, Question};
    use crate::cli::llm::InferTypeName;

    #[test]
    fn test_to_chat_request_conversion() {
        let question = Question {
            used_types: indexset! {"Profile".to_owned(), "Person".to_owned()},
            fields: vec![
                ("id".to_string(), "String".to_string()),
                ("name".to_string(), "String".to_string()),
                ("age".to_string(), "Int".to_string()),
            ],
        };
        let request: ChatRequest = question.try_into().unwrap();
        insta::assert_debug_snapshot!(request);
    }

    #[test]
    fn test_chat_response_parse() {
        let resp = ChatResponse {
            content: Some(MessageContent::Text(
                "{\"suggestions\":[\"Post\",\"Story\",\"Article\",\"Event\",\"Brief\"]}".to_owned(),
            )),
            ..Default::default()
        };
        let answer = Answer::try_from(resp).unwrap();
        insta::assert_debug_snapshot!(answer);
    }

    #[test]
    fn test_is_auto_generated() {
        assert!(InferTypeName::is_auto_generated("T1", false));
        assert!(InferTypeName::is_auto_generated("T1234", false));
        assert!(InferTypeName::is_auto_generated("M1", false));
        assert!(InferTypeName::is_auto_generated("M5678", false));
        assert!(InferTypeName::is_auto_generated("Some__Type", true));

        assert!(!InferTypeName::is_auto_generated("Some__Type", false));
        assert!(!InferTypeName::is_auto_generated("User", false));
        assert!(!InferTypeName::is_auto_generated("T123abc", false));
        assert!(!InferTypeName::is_auto_generated("M", false));
        assert!(!InferTypeName::is_auto_generated("", false));
        assert!(!InferTypeName::is_auto_generated("123T", false));
        assert!(!InferTypeName::is_auto_generated("A1234", false));
    }
}
