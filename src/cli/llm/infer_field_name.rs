use std::collections::HashMap;

use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use super::model::groq;
use super::{Error, Result, Wizard};
use crate::core::generator::Input;
use crate::core::Mustache;

#[derive(Default)]
pub struct InferFieldName {
    secret: Option<String>,
    use_llm: bool,
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
    field: (String, String),
}

impl TryInto<ChatRequest> for Question {
    type Error = Error;

    fn try_into(self) -> Result<ChatRequest> {
        let input1 = serde_json::to_string_pretty(&Question {
            field: (
                "jsonplaceholder.typicode.com/posts".to_string(),
                "GET".to_string(),
            ),
        })?;
        let output1 = serde_json::to_string_pretty(&Answer {
            suggestions: vec![
                "posts".into(),
                "article".into(),
                "stories".into(),
                "pictures".into(),
                "events".into(),
            ],
        })?;
        let input2 = serde_json::to_string_pretty(&Question {
            field: (
                "jsonplaceholder.typicode.com/posts/1".to_string(),
                "GET".to_string(),
            ),
        })?;

        let output2 = serde_json::to_string_pretty(&Answer {
            suggestions: vec![
                "post".into(),
                "article".into(),
                "story".into(),
                "picture".into(),
                "event".into(),
            ],
        })?;
        let count = 5;

        let template_str = include_str!("prompts/infer_field_name.md");
        let template = Mustache::parse(template_str);

        let context = json!({
            "input1": input1,
            "output1": output1,
            "input2": input2,
            "output2": output2,
            "count": count,
        });

        let rendered_prompt = template.render(&context);

        Ok(ChatRequest::new(vec![
            ChatMessage::system(rendered_prompt),
            ChatMessage::user(serde_json::to_string(&self)?),
        ]))
    }
}

impl InferFieldName {
    pub fn new(secret: Option<String>, use_llm: bool) -> InferFieldName {
        Self { secret, use_llm }
    }
    pub async fn generate(
        &mut self,
        mut input_samples: Vec<Input>,
    ) -> Result<HashMap<Url, Vec<String>>> {
        let secret = self.secret.as_ref().map(|s| s.to_owned());

        let wizard: Wizard<Question, Answer> = Wizard::new(groq::LLAMA38192, secret);

        let mut new_field_names: HashMap<Url, Vec<String>> = HashMap::new();
        let total = input_samples.len();
        for (i, input) in input_samples.iter_mut().enumerate() {
            if let Input::Json { url, method, field_name, .. } = input {
                if field_name.is_none() {
                    let mut suggested_field_names = vec![format!("field{}", i)];
                    if self.use_llm {
                        let domain = url.host().unwrap_or(url::Host::Domain("")).to_string();
                        let formatted_url = format!("{:?}{:?}", domain, url.path());
                        let question = Question { field: (formatted_url, method.to_string()) };

                        let mut delay = 3;
                        loop {
                            let answer = wizard.ask(question.clone()).await;
                            match answer {
                                Ok(answer) => {
                                    suggested_field_names = answer.suggestions;
                                    tracing::info!(
                                        "Suggestions for {}: {:?} - {}/{}",
                                        url.path(),
                                        suggested_field_names,
                                        i + 1,
                                        total
                                    );
                                    // TODO: case where suggested names are already used, then
                                    // extend the base question
                                    break;
                                }
                                Err(e) => {
                                    // TODO: log errors after certain number of retries.
                                    if let Error::GenAI(_) = e {
                                        // TODO: retry only when it's required.
                                        tracing::warn!(
                                            "Unable to retrieve a name for the field '{}'. Retrying in {}s. Error: {}",
                                            url.path(),
                                            delay,
                                            e
                                        );
                                        tokio::time::sleep(tokio::time::Duration::from_secs(delay))
                                            .await;
                                        delay = std::cmp::min(delay * 2, 60);
                                    }
                                }
                            }
                        }
                    }

                    new_field_names.insert(url.clone(), suggested_field_names);
                }
            }
        }
        Ok(new_field_names)
    }
}
