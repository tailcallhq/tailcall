use std::collections::HashMap;

use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::model::groq;
use super::{Error, Result, TypeUsageIndex, Wizard};
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
    type_refs: IndexMap<String, u32>,
    fields: Vec<(String, String)>,
}

impl TryInto<ChatRequest> for Question {
    type Error = Error;

    fn try_into(self) -> Result<ChatRequest> {
        let content = serde_json::to_string(&self)?;
        let mut type_refs = IndexMap::new();
        type_refs.insert("User".to_owned(), 13);
        type_refs.insert("Profile".to_owned(), 11);
        type_refs.insert("Person".to_owned(), 14);

        let input = serde_json::to_string_pretty(&Question {
            type_refs,
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
            ChatMessage::system("In this context, 'type_refs' refer to the number of times the type is used in different fields as output type. For example, if the following type is referenced 12 times in the 'user' field and 13 times in the 'profile' field."),
            ChatMessage::system("While suggesting the type name, do consider above type_refs information for understanding the type context."),
            ChatMessage::system("Example Input:"),
            ChatMessage::system(input),
            ChatMessage::system("Example Output:"),
            ChatMessage::system(output),
            ChatMessage::system("Ensure the output is in valid JSON format".to_string()),
            ChatMessage::system(
                "Do not add any additional text before or after the json".to_string(),
            ),
            ChatMessage::user("User Input:"),
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

        let wizard: Wizard<Question, Answer> = Wizard::new(groq::LLAMA70B_VERSATILE, secret);

        let mut new_name_mappings = HashMap::new();

        // removed root type from types.
        let types_to_be_processed = config
            .types
            .iter()
            .filter(|(type_name, _)| !config.is_root_operation_type(type_name))
            .collect::<Vec<_>>();

        let type_tango = TypeUsageIndex::new(&config);

        let total = types_to_be_processed.len();
        for (i, (type_name, type_)) in types_to_be_processed.into_iter().enumerate() {
            if let Some(type_refs) = type_tango.get(type_name) {
                // convert to prompt.
                let question = Question {
                    type_refs: type_refs.clone(),
                    fields: type_
                        .fields
                        .iter()
                        .map(|(k, v)| (k.clone(), v.type_of.clone()))
                        .collect(),
                };

                let mut delay = 3;
                loop {
                    let answer = wizard.ask(question.clone()).await;
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
        }

        Ok(new_name_mappings.into_iter().map(|(k, v)| (v, k)).collect())
    }
}

#[cfg(test)]
mod test {
    use genai::chat::{ChatRequest, ChatResponse, MessageContent};
    use indexmap::IndexMap;

    use super::{Answer, Question};

    #[test]
    fn test_to_chat_request_conversion() {
        let mut type_refs = IndexMap::new();
        type_refs.insert("User".to_owned(), 13);
        type_refs.insert("Profile".to_owned(), 11);
        type_refs.insert("Person".to_owned(), 14);

        let question = Question {
            type_refs,
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
