use derive_setters::Setters;
use genai::adapter::{AdapterConfig, AdapterKind};
use genai::chat::{ChatOptions, ChatRequest, ChatResponse};
use genai::resolver::{AuthData, AuthResolver};
use genai::{Client, ConfigSet};

use super::adapter::Adapter;
use super::Result;

#[derive(Setters, Clone)]
pub struct Wizard<Q, A> {
    client: Client,
    // TODO: change model to enum
    model: Adapter,
    _q: std::marker::PhantomData<Q>,
    _a: std::marker::PhantomData<A>,
}

impl<Q, A> Wizard<Q, A> {
    pub fn new(model: Adapter, key: String) -> Self {
        // let auth_resolver = AuthResolver::from_sync_resolver(
        //     |kind: AdapterKind,
        //      _config_set: &ConfigSet<'_>|
        //      -> anyhow::Result<Option<AuthData>, genai::resolver::Error> {
        //         Ok(Some(AuthData::from_single(key.to_string())))
        //     },
        // );

        let auth_res = AuthResolver::from_key_value(key);
        let adapter_config = AdapterConfig::default().with_auth_resolver(auth_res);

        Self {
            client: Client::builder()
                .insert_adapter_config(AdapterKind::Groq, adapter_config)
                .with_chat_options(ChatOptions::default().with_json_mode(true))
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
            .exec_chat(&self.model.to_string(), q.try_into()?, None)
            .await?;
        A::try_from(response)
    }
}
