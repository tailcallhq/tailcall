use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Error, Result, Wizard};
use crate::core::config::transformer::{FieldInfo, RenameFields, TypeName};
use crate::core::config::{Config, Resolver};
use crate::core::valid::{Valid, Validator};
use crate::core::{AsyncTransform, Mustache, Transform};

const BASE_TEMPLATE: &str = include_str!("prompts/infer_field_name.md");

pub struct InferFieldName {
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
    url: String,
    method: String,
}

impl TryInto<ChatRequest> for Question {
    type Error = Error;

    fn try_into(self) -> Result<ChatRequest> {
        let template = Mustache::parse(BASE_TEMPLATE);

        let context = json!({
            "count": 5,
        });

        let rendered_prompt = template.render(&context);

        Ok(ChatRequest::new(vec![
            ChatMessage::system(rendered_prompt),
            ChatMessage::user(serde_json::to_string(&self)?),
        ]))
    }
}

impl InferFieldName {
    pub fn new(model: String, secret: Option<String>) -> InferFieldName {
        Self { wizard: Wizard::new(model, secret) }
    }

    pub async fn generate(&self, config: &Config) -> Result<IndexMap<String, FieldInfo>> {
        let mut mapping: IndexMap<String, FieldInfo> = IndexMap::new();

        for (type_name, type_) in config.types.iter() {
            for (field_name, field) in type_.fields.iter() {
                if let Some(Resolver::Http(http)) = &field.resolver {
                    let question = Question {
                        url: http.base_url.as_ref().unwrap().clone(),
                        method: http.method.to_string(),
                    };

                    let mut delay = 3;
                    loop {
                        let answer = self.wizard.ask(question.clone()).await;
                        match answer {
                            Ok(answer) => {
                                tracing::info!(
                                    "Suggestions for Field {}: [{:?}]",
                                    field_name,
                                    answer.suggestions,
                                );
                                mapping.insert(
                                    field_name.to_owned(),
                                    FieldInfo::new(answer.suggestions, TypeName::new(type_name)),
                                );
                                break;
                            }
                            Err(e) => {
                                if let Error::GenAI(_) = e {
                                    tracing::warn!(
                                            "Unable to retrieve a name for the field '{}'. Retrying in {}s",
                                            field_name,
                                            delay
                                        );
                                    tokio::time::sleep(tokio::time::Duration::from_secs(delay))
                                        .await;
                                    delay *= std::cmp::min(delay * 2, 60);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(mapping)
    }
}

impl AsyncTransform for InferFieldName {
    type Value = Config;
    type Error = Error;

    async fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        match self.generate(&value).await {
            Ok(suggested_names) => {
                match RenameFields::new(suggested_names)
                    .transform(value)
                    .to_result()
                {
                    Ok(v) => Valid::succeed(v),
                    Err(e) => Valid::fail(Error::Err(e.to_string())),
                }
            }
            Err(err) => Valid::fail(err),
        }
    }
}

#[cfg(test)]
mod test {
    use genai::chat::{ChatRequest, ChatResponse, MessageContent};

    use super::{Answer, Question};

    #[test]
    fn test_to_chat_request_conversion() {
        let question = Question {
            url: "https://jsonplaceholder.typicode.com/posts".to_string(),
            method: "GET".to_string(),
        };
        let request: ChatRequest = question.try_into().unwrap();
        insta::assert_debug_snapshot!(request);
    }

    #[test]
    fn test_chat_response_parse() {
        let resp = ChatResponse {
            content: Some(MessageContent::Text(
                "{\"suggestions\":[\"posts\",\"postList\",\"articles\",\"articlesList\",\"entries\"]}".to_owned(),
            )),
            ..Default::default()
        };
        let answer = Answer::try_from(resp).unwrap();
        insta::assert_debug_snapshot!(answer);
    }
}
