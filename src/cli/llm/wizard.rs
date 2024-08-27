use super::error::{Error, Result, WebcError};
use derive_setters::Setters;
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatResponse};
use genai::resolver::AuthResolver;
use genai::Client;
use reqwest::StatusCode;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::RetryIf;

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
        Q: TryInto<ChatRequest, Error = Error> + Clone,
        A: TryFrom<ChatResponse, Error = Error>,
    {
        let retry_strategy = ExponentialBackoff::from_millis(1000).map(jitter).take(5);

        RetryIf::spawn(
            retry_strategy,
            || async {
                let request = q.clone().try_into()?;
                self.client
                    .exec_chat(self.model.as_str(), request, None)
                    .await
                    .map_err(Error::from)
                    .and_then(A::try_from)
            },
            |err: &Error| matches!(err, Error::Webc(WebcError::ResponseFailedStatus { status, .. }) if *status == StatusCode::TOO_MANY_REQUESTS)
        )
        .await
    }
}
