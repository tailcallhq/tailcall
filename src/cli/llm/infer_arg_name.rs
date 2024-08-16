use std::collections::HashMap;

use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use serde::{Deserialize, Serialize};

use super::model::gemini;
use super::{Error, Result, Wizard};
use crate::core::config::Config;

#[derive(Default)]
pub struct InferArgName {
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
    fields: (String, (String, String)),
}

impl TryInto<ChatRequest> for Question {
    type Error = Error;

    fn try_into(self) -> Result<ChatRequest> {
        let content = serde_json::to_string(&self)?;
        let input = serde_json::to_string_pretty(&Question {
            fields: ("user".to_string(), ("p1".to_string(), "String".to_string())),
        })?;

        let output = serde_json::to_string_pretty(&Answer {
            suggestions: vec![
                "userId".into(),
                "userName".into(),
                "Id".into(),
                "email".into(),
                "userKey".into(),
            ],
        })?;

        Ok(ChatRequest::new(vec![
            ChatMessage::system(
                "Given the sample schema of a GraphQL type suggest 5 meaningful argumnent names for it accoding to the base parent field name and each argument's type.",
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

impl InferArgName {
    pub fn new(secret: Option<String>) -> InferArgName {
        Self { secret }
    }
    pub async fn generate(
        &mut self,
        config: &Config,
    ) -> Result<HashMap<String, Vec<(String, String)>>> {
        let secret = self.secret.as_ref().map(|s| s.to_owned());

        let wizard: Wizard<Question, Answer> = Wizard::new(gemini::GEMINI15_FLASH_LATEST, secret);

        let mut new_name_mappings: HashMap<String, Vec<(String, String)>> = HashMap::new();

        let query_type = config.types.get("Query");

        if let Some(type_) = query_type {
            let mut args_to_be_processed = HashMap::new();
            let fields = &type_.fields.keys().collect::<Vec<_>>();
            for key in fields {
                if let Some(field) = &type_.fields.get(key.as_str()) {
                    let args = field.args.iter().collect::<Vec<_>>();
                    if !args.is_empty() {
                        let args = args
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.type_of.clone()))
                            .collect::<Vec<_>>();
                        args_to_be_processed.insert(key.to_string(), args);
                    }
                }
            }
            let total = args_to_be_processed.len();
            for (i, arg) in args_to_be_processed.into_iter().enumerate() {
                for j in arg.1.iter() {
                    let arg = (arg.0.to_string(), j.to_owned());
                    let question = Question { fields: arg.clone() };

                    let mut delay = 3;
                    loop {
                        let answer = wizard.ask(question.clone()).await;
                        match answer {
                            Ok(answer) => {
                                let name = &answer.suggestions.join(", ");
                                for name in answer.suggestions {
                                    // check if the name is already used for current field
                                    if let Some(field) = &type_.fields.get(arg.0.as_str()) {
                                        let args = field.args.iter().collect::<Vec<_>>();
                                        if !args.is_empty() && args.iter().any(|(k, _)| *k == &name)
                                        {
                                            continue;
                                        }
                                        new_name_mappings
                                            .entry(arg.0.to_owned())
                                            .and_modify(|v| v.push((j.0.clone(), name.to_owned())))
                                            .or_insert_with(|| {
                                                vec![(j.0.clone(), name.to_owned())]
                                            });
                                        break;
                                    }
                                }
                                tracing::info!(
                                    "Suggestions for {}'s argument {}: [{}] - {}/{}",
                                    arg.0,
                                    arg.1 .0,
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
                                "Unable to retrieve a name for the argument '{}'. Retrying in {}s. Error: {}",
                                arg.0,
                                delay,
                                e
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
            tracing::info!("new_name_mappings {:?}", new_name_mappings);

            Ok(new_name_mappings)
        } else {
            Ok(HashMap::new())
        }
    }
}
