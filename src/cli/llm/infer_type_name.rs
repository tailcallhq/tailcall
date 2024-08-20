use std::collections::HashMap;

use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::model::groq;
use super::{Error, Result, Wizard};
use crate::core::config::Config;
use crate::core::Mustache;

#[derive(Default)]
pub struct InferTypeName {
    secret: Option<String>,
    wizard: Option<Wizard<Question, Answer>>,
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

        let template_str = include_str!("prompts/infer_type_name.md");
        let template = Mustache::parse(template_str);

        let context = json!({
            "input": input,
            "output": output,
        });

        let rendered_prompt = template.render(&context);

        Ok(ChatRequest::new(vec![
            ChatMessage::system(rendered_prompt),
            ChatMessage::user(serde_json::to_string(&self)?),
        ]))
    }
}

impl InferTypeName {
    pub fn new(secret: Option<String>) -> InferTypeName {
        Self { secret, wizard: None }
    }

    fn create_system_messages() -> Vec<ChatMessage> {
        vec![
            ChatMessage::system(
                "Given the sample schema of a GraphQL type suggest 5 meaningful names for it.",
            ),
            ChatMessage::system("The name should be concise and preferably a single word"),
            ChatMessage::system("Example Input:"),
            ChatMessage::system(
                serde_json::to_string_pretty(&Question {
                    fields: vec![
                        ("id".to_string(), "String".to_string()),
                        ("name".to_string(), "String".to_string()),
                        ("age".to_string(), "Int".to_string()),
                    ],
                })
                .unwrap(),
            ),
            ChatMessage::system("Example Output:"),
            ChatMessage::system(
                serde_json::to_string_pretty(&Answer {
                    suggestions: vec![
                        "Person".into(),
                        "Profile".into(),
                        "Member".into(),
                        "Individual".into(),
                        "Contact".into(),
                    ],
                })
                .unwrap(),
            ),
            ChatMessage::system("Ensure the output is in valid JSON format"),
            ChatMessage::system("Do not add any additional text before or after the json"),
        ]
    }

    pub async fn generate(&mut self, config: &Config) -> Result<HashMap<String, String>> {
        let secret = self.secret.as_ref().map(|s| s.to_owned());
        self.wizard = Some(Wizard::new(groq::LLAMA38192, secret));

        let mut new_name_mappings: HashMap<String, String> = HashMap::new();

        let types_to_be_processed = config
            .types
            .iter()
            .filter(|(type_name, _)| !config.is_root_operation_type(type_name))
            .collect::<Vec<_>>();

        let total = types_to_be_processed.len();
        let system_messages = Self::create_system_messages();

        for (i, (type_name, type_)) in types_to_be_processed.into_iter().enumerate() {
            let question = Question {
                fields: type_
                    .fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.type_of.clone()))
                    .collect(),
            };

            let mut delay = 3;
            loop {
                let answer = self.ask_with_context(&system_messages, &question).await;
                match answer {
                    Ok(answer) => {
                        let name = &answer.suggestions.join(", ");
                        for name in answer.suggestions {
                            if config.types.contains_key(&name)
                                || new_name_mappings.contains_key(&name)
                            {
                                continue;
                            }
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

                        break;
                    }
                    Err(e) => {
                        if let Error::GenAI(_) = e {
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

    async fn ask_with_context(
        &self,
        system_messages: &[ChatMessage],
        question: &Question,
    ) -> Result<Answer> {
        // Remove the conversion to ChatRequest
        self.wizard.as_ref().unwrap().ask(question.clone()).await
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
