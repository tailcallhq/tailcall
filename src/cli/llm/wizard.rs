use derive_setters::Setters;
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatResponse};
use genai::resolver::AuthResolver;
use genai::Client;

use super::Result;

#[derive(Setters, Clone)]
pub struct Wizard<Q, A> {
    client: Client,
    model: String,
    _q: std::marker::PhantomData<Q>,
    _a: std::marker::PhantomData<A>,
}

impl<Q, A> Wizard<Q, A> {
    pub fn new(model: String, secret: Option<String>) -> Self {
        let mut config = genai::adapter::AdapterConfig::default();
        if let Some(key) = secret {
            config = config.with_auth_resolver(AuthResolver::from_key_value(key));
        }

        let adapter = AdapterKind::from_model(model.as_str()).unwrap_or(AdapterKind::Ollama);

        let chat_options = ChatOptions::default()
            .with_json_mode(true)
            .with_temperature(0.0);

        Self {
            client: Client::builder()
                .with_chat_options(chat_options)
                .insert_adapter_config(adapter, config)
                .build(),
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
