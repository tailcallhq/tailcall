use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tokio_retry::Retry;
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatResponse};
use genai::resolver::AuthResolver;
use genai::Client;
use super::Error;
use super::Result;
use crate::cli::llm::model::Model;
use derive_setters::Setters;

#[derive(Setters, Clone)]
pub struct Wizard<Q, A> {
    client: Client,
    model: Model,
    _q: std::marker::PhantomData<Q>,
    _a: std::marker::PhantomData<A>,
}

impl<Q, A> Wizard<Q, A> {
    pub fn new(model: Model, secret: Option<String>) -> Self {
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

    pub async fn ask_with_retry(&self, q: Q) -> Result<A>
    where
        Q: TryInto<ChatRequest, Error = Error>,
        A: TryFrom<ChatResponse, Error = Error>,
    {
        let strategy = ExponentialBackoff::from_millis(100).map(jitter).take(5);

        let retry_future = Retry::spawn(strategy, || async {
            let response = self
                .client
                .exec_chat(self.model.as_str(), q.clone().try_into()?, None)
                .await;
            
            match response {
                Ok(res) => {
                    if res.status_code() == 429 {
                        Err(Error::GenAI("API rate limit exceeded".into()))
                    } else {
                        A::try_from(res).map_err(|e| Error::GenAI(e.to_string()))
                    }
                },
                Err(e) => Err(Error::GenAI(e.to_string())),
            }
        });

        retry_future.await
    }
}
