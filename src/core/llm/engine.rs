use derive_setters::Setters;
use genai::chat::{ChatMessage, ChatRequest};
use genai::client::Client;
use serde::de::DeserializeOwned;

use super::Error;

#[derive(Setters)]
pub struct Engine<A> {
    system_prompt: Option<String>,
    client: Client,
    model: Model,
    start_marker: String,
    end_marker: String,
    phantom: std::marker::PhantomData<A>,
}

#[derive(Default)]
pub enum Model {
    #[default]
    Gemini,
    Babbage,
    Davinci,
}

impl Model {
    fn as_str(&self) -> &'static str {
        todo!()
    }
}

impl<A: DeserializeOwned> Engine<A> {
    pub fn new(start_marker: String, end_marker: String) -> Self {
        Self {
            system_prompt: None,
            client: Default::default(),
            model: Model::default(),
            phantom: std::marker::PhantomData,
            start_marker,
            end_marker,
        }
    }

    pub async fn prompt(&self, prompt: &str) -> Result<A, super::Error> {
        let mut messages = vec![];
        match &self.system_prompt {
            Some(prompt) => messages.push(ChatMessage::system(prompt)),
            None => (),
        };

        messages.push(ChatMessage::user(prompt));
        let chat_req = ChatRequest::new(messages);
        let response = self
            .client
            .exec_chat(self.model.as_str(), chat_req, None)
            .await?;
        let content = response.content.ok_or(Error::EmptyResponse)?;

        // Slice the text between the start and end markers
        let start = content
            .find(&self.start_marker)
            .ok_or(Error::MissingMarker(self.start_marker.clone()))?;
        let end = content
            .rfind(&self.end_marker)
            .ok_or(Error::MissingMarker(self.end_marker.clone()))?;
        let json_str = &content[start..=end];

        Ok(serde_json::from_str(json_str)?)
    }
}
