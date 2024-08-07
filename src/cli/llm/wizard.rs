use derive_setters::Setters;
use genai::chat::{ChatRequest, ChatResponse};
use genai::client::Client;

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

    pub async fn ask(&self, q: Q) -> Result<A>
    where
        Q: TryInto<ChatRequest, Error = super::Error>,
        A: TryFrom<ChatResponse, Error = super::Error>,
    {
        let response = self
            .client
            .exec_chat(self.model.as_str(), q.try_into()?, None)
            .await?;
        A::try_from(response)
    }
}
