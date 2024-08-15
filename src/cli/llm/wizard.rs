use derive_setters::Setters;
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatResponse};
use genai::resolver::AuthResolver;
use genai::Client;
use rand::seq::SliceRandom;
use rand::thread_rng;

use super::Result;
use crate::cli::llm::model::{Model, ModelKind};

#[derive(Setters, Clone)]
pub struct Wizard<Q, A> {
    client: Client,
    model: Model,
    _q: std::marker::PhantomData<Q>,
    _a: std::marker::PhantomData<A>,
}

impl<Q, A> Wizard<Q, A> {
    pub fn new(models: Vec<ModelKind>, secret: Option<String>) -> Self {
        let mut config = genai::adapter::AdapterConfig::default();
        if let Some(key) = secret {
            config = config.with_auth_resolver(AuthResolver::from_key_value(key));
        }

        let mut rng = thread_rng();
        let model_kind = models.choose(&mut rng).unwrap();
        let model = model_kind.to_model();

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
