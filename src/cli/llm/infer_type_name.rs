use std::collections::HashMap;
use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use serde::{Deserialize, Serialize};
use super::model::groq;
use super::{Error, Result, Wizard};
use crate::core::config::Config;

#[derive(Default)]
pub struct InferTypeName {
    secret: Option<String>,
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
            ChatMessage::system(
                "Do not add any additional text before or after the json".to_string(),
            ),
            ChatMessage::user(content),
        ]))
    }
}

impl InferTypeName {
    pub fn new(secret: Option<String>) -> InferTypeName {
        Self { secret }
    }

    pub async fn generate(&mut self, config: &Config) -> Result<HashMap<String, String>> {
        let secret = self.secret.as_ref().map(|s| s.to_owned());

        let wizard: Wizard<Question, Answer> = Wizard::new(groq::LLAMA38192, secret);

        let mut new_name_mappings: HashMap<String, String> = HashMap::new();

        let types_to_be_processed = config
            .types
            .iter()
            .filter(|(type_name, _)| !config.is_root_operation_type(type_name))
            .collect::<Vec<_>>();

        for (type_name, type_) in types_to_be_processed.into_iter() {
            let question = Question {
                fields: type_
                    .fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.type_of.clone()))
                    .collect(),
            };

            match wizard.ask(question).await {
                Ok(answer) => {
                    for name in answer.suggestions {
                        if !config.types.contains_key(&name)
                            && !new_name_mappings.contains_key(&name) {
                            new_name_mappings.insert(name, type_name.to_owned());
                            break;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to retrieve a name for the type '{}': {:?}", type_name, e);
                }
            }
        }

        Ok(new_name_mappings.into_iter().map(|(k, v)| (v, k)).collect())
    }
}

#[cfg(test)]
mod test {
    use genai::chat::{ChatRequest, ChatResponse, MessageContent};

    use super::{Answer, Question};

    #[test]
    fn test_to_chat_request_conversion() {
        let question = Question {
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
}