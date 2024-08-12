use std::sync::Arc;

use derive_setters::Setters;
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatResponse};
use genai::Client;
use tokio_retry::strategy::ExponentialBackoff;

use super::Result;
use crate::cli::llm::model::Model;

#[derive(Setters, Clone)]
pub struct Wizard<Q, A> {
    client: Arc<Client>,
    model: Model,
    _q: std::marker::PhantomData<Q>,
    _a: std::marker::PhantomData<A>,
}

impl<Q, A> Wizard<Q, A> {
    pub fn new(model: Model, secret: Option<String>) -> Self {
        let mut config = genai::adapter::AdapterConfig::default();
        if let Some(key) = secret {
            config = config.with_auth_env_name(key);
        }

        let adapter = AdapterKind::from_model(model.as_str()).unwrap_or(AdapterKind::Ollama);
        let client = Client::builder()
            .with_chat_options(
                ChatOptions::default()
                    .with_json_mode(true)
                    .with_temperature(0.0),
            )
            .insert_adapter_config(adapter, config)
            .build();
        Self {
            client: Arc::new(client),
            model,
            _q: Default::default(),
            _a: Default::default(),
        }
    }

    async fn ask_inner(&self, q: Q) -> Result<A>
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

    pub async fn ask(&self, q: Q) -> Result<A>
    where
        Q: TryInto<ChatRequest, Error = super::Error> + Clone,
        A: TryFrom<ChatResponse, Error = super::Error>,
    {
        let retry_strategy = ExponentialBackoff::from_millis(3)
            .map(tokio_retry::strategy::jitter)
            .take(3);

        

        tokio_retry::Retry::spawn(retry_strategy, || async { self.ask_inner(q.clone()).await })
                .await
    }
}
