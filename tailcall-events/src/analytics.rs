use lazy_static::lazy_static;
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use sysinfo::System;

lazy_static! {
    static ref API_SECRET: String = "GVaEzXFeRkCI9YBIylbEjQ".to_string();
    static ref MEASUREMENT_ID: String = "G-JEP3QDWT0G".to_string();
    static ref BASE_URL: String = "https://www.google-analytics.com".to_string();
    static ref PARAPHRASE: String = "tc_key".to_string();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Params {
    pub cpu_cores: String,
    pub os_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub params: Params,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostData {
    base_url: String,
    api_secret: String,
    measurement_id: String,
    #[serde(rename = "client_id")]
    pub client_id: String,
    pub events: Vec<Event>,
}

impl PostData {
    fn create_client_id() -> anyhow::Result<String> {
        let mut builder = IdBuilder::new(Encryption::SHA256);
        builder
            .add_component(HWIDComponent::SystemID)
            .add_component(HWIDComponent::CPUCores);

        Ok(builder.build(PARAPHRASE.as_str())?)
    }
    fn prepare_event(command_name: &str) -> anyhow::Result<PostData> {
        let sys = System::new_all();
        let cores = sys.physical_core_count().unwrap_or(2).to_string();
        let os_name = System::long_os_version().unwrap_or("Unknown".to_string());
        Ok(PostData {
            base_url: BASE_URL.to_string(),
            api_secret: API_SECRET.to_string(),
            measurement_id: MEASUREMENT_ID.to_string(),
            client_id: PostData::create_client_id()?,
            events: vec![Event {
                name: command_name.to_string(),
                params: Params { cpu_cores: cores, os_name },
            }],
        })
    }

    pub async fn send_event(command_name: &str) -> Result<(), anyhow::Error> {
        let post_data = PostData::prepare_event(command_name)?;
        tracing::debug!("Sending event: {:?}", post_data);
        let request = reqwest::Request::try_from(post_data)?;
        let client = reqwest::Client::new();
        let response = client.execute(request).await?;
        let text = response.text().await?;
        tracing::debug!("Validation Message: {:?}", text);
        Ok(())
    }

    pub async fn alive_event_poll(runtime: &tokio::runtime::Runtime) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        runtime.spawn(async move {
            loop {
                interval.tick().await;
                let _ = PostData::send_event("alive").await;
            }
        });
    }
}

impl TryFrom<PostData> for reqwest::Request {
    type Error = anyhow::Error;

    fn try_from(value: PostData) -> std::result::Result<Self, Self::Error> {
        let mut url = reqwest::Url::parse(&value.base_url)?;
        url.set_path("/mp/collect");
        url.query_pairs_mut()
            .append_pair("api_secret", &value.api_secret)
            .append_pair("measurement_id", &value.measurement_id);
        let mut request = reqwest::Request::new(reqwest::Method::POST, url);
        let header_name = HeaderName::from_static("content-type");
        let header_value = HeaderValue::from_str("application/json")?;
        request.headers_mut().insert(header_name, header_value);
        let event = serde_json::json!({
            "client_id": value.client_id,
            "events": value.events,
        });

        let _ = request
            .body_mut()
            .insert(reqwest::Body::from(serde_json::to_string(&event)?));
        Ok(request)
    }
}
