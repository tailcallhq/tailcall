use derive_setters::Setters;
use genai::chat::{ChatOptions, ChatRequest, ChatResponse};
use genai::Client;

use super::Result;

#[derive(Setters, Clone)]
pub struct Wizard<Q, A> {
    client: Client,
    // TODO: change model to enum
    model: String,
    _q: std::marker::PhantomData<Q>,
    _a: std::marker::PhantomData<A>,
}

impl<Q, A> Wizard<Q, A> {
    pub fn new(model: String) -> Self {
        Self {
            client: Default::default(),
            model,
            _q: Default::default(),
            _a: Default::default(),
        }
    }

    pub fn with_json_mode(self, json_mode: bool) -> Self {
        Self {
            client: Client::builder()
                .with_chat_options(ChatOptions::default().with_json_mode(json_mode))
                .build(),
            model: self.model,
            _q: self._q,
            _a: self._a,
        }
    }

    pub async fn ask(&self, q: Q) -> Result<A>
    where
        Q: TryInto<ChatRequest, Error = super::Error>,
        A: TryFrom<ChatResponse, Error = super::Error>,
    {
        let chat_opts = ChatOptions::default().with_json_mode(true);
        let response = self
            .client
            .exec_chat(self.model.as_str(), q.try_into()?, Some(&chat_opts))
            .await?;
        A::try_from(response)
    }
}
