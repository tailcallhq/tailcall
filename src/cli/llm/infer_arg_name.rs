use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Error, Result, Wizard};
use crate::core::config::transformer::{ArgumentInfo, RenameArgs};
use crate::core::config::{Config, Resolver};
use crate::core::valid::{Valid, Validator};
use crate::core::{AsyncTransform, Mustache, Transform};

pub struct InferArgName {
    wizard: Wizard<Question, Answer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetaData {
    name: String,
    #[serde(rename = "outputType")]
    output_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OperationDefinition {
    argument: MetaData,
    field: MetaData,
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
    definition: OperationDefinition,
}

impl TryInto<ChatRequest> for Question {
    type Error = Error;

    fn try_into(self) -> Result<ChatRequest> {
        let input2 = OperationDefinition {
            argument: {
                MetaData {
                    name: "input2".to_string(),
                    output_type: "Article".to_string(),
                }
            },
            field: {
                MetaData {
                    name: "createPost".to_string(),
                    output_type: "Post".to_string(),
                }
            },
        };

        let input = serde_json::to_string_pretty(&Question { definition: input2 })?;
        let output = serde_json::to_string_pretty(&Answer {
            suggestions: vec![
                "createPostInput".into(),
                "postInput".into(),
                "articleInput".into(),
                "noteInput".into(),
                "messageInput".into(),
            ],
        })?;

        let template_str = include_str!("prompts/infer_arg_name.md");
        let template = Mustache::parse(template_str);

        let context = json!({
            "input": input,
            "output": output,
            "count": 5,
        });

        let rendered_prompt = template.render(&context);
        let request = ChatRequest::new(vec![
            ChatMessage::system(rendered_prompt),
            ChatMessage::user(serde_json::to_string(&self.definition)?),
        ]);
        Ok(request)
    }
}

impl InferArgName {
    pub fn new(model: String, secret: Option<String>) -> InferArgName {
        Self { wizard: Wizard::new(model, secret) }
    }

    pub async fn generate(&self, config: &Config) -> Result<IndexMap<String, ArgumentInfo>> {
        let mut mapping: IndexMap<String, ArgumentInfo> = IndexMap::new();

        for (type_name, type_) in config.types.iter() {
            // collect all the args that's needs to be processed with LLM.
            for (field_name, field) in type_.fields.iter() {
                if field.args.is_empty() {
                    continue;
                }
                // filter out query params as we shouldn't change the names of query params.
                for (arg_name, arg) in field.args.iter().filter(|(k, _)| match &field.resolver {
                    Some(Resolver::Http(http)) => !http.query.iter().any(|q| &q.key == *k),
                    _ => true,
                }) {
                    let question = OperationDefinition {
                        argument: MetaData {
                            name: arg_name.to_string(),
                            output_type: arg.type_of.name().to_owned(),
                        },
                        field: MetaData {
                            name: field_name.to_string(),
                            output_type: field.type_of.name().to_owned(),
                        },
                    };

                    let question = Question { definition: question };

                    let mut delay = 3;
                    loop {
                        let answer = self.wizard.ask(question.clone()).await;
                        match answer {
                            Ok(answer) => {
                                tracing::info!(
                                    "Suggestions for Argument {}: [{:?}]",
                                    arg_name,
                                    answer.suggestions,
                                );
                                mapping.insert(
                                    arg_name.to_owned(),
                                    ArgumentInfo::new(
                                        answer.suggestions,
                                        field_name.to_owned(),
                                        type_name.to_owned(),
                                    ),
                                );
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

impl AsyncTransform for InferArgName {
    type Value = Config;
    type Error = Error;

    async fn transform(
        &self,
        value: Self::Value,
    ) -> crate::core::valid::Valid<Self::Value, Self::Error> {
        match self.generate(&value).await {
            Ok(suggested_names) => {
                match RenameArgs::new(suggested_names)
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
    use crate::cli::llm::infer_arg_name::{OperationDefinition, MetaData};

    #[test]
    fn test_to_chat_request_conversion() {
        let question = Question {
            definition: OperationDefinition {
                argument: MetaData {
                    name: "input2".to_string(),
                    output_type: "Article".to_string(),
                },
                field: MetaData {
                    name: "createPost".to_string(),
                    output_type: "Post".to_string(),
                },
            },
        };
        let request: ChatRequest = question.try_into().unwrap();
        insta::assert_debug_snapshot!(request);
    }

    #[test]
    fn test_chat_response_parse() {
        let resp = ChatResponse {
            content: Some(MessageContent::Text(
                "{\"suggestions\":[\"createPostInput\",\"postInput\",\"articleInput\",\"noteInput\",\"messageInput\"]}".to_owned(),
            )),
            ..Default::default()
        };
        let answer = Answer::try_from(resp).unwrap();
        insta::assert_debug_snapshot!(answer);
    }
}
