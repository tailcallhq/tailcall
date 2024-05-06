use lazy_static::lazy_static;
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref API_SECRET: String = "secret".to_string();
    static ref MEASUREMENT_ID: String = "G-NCQKBRNVDW".to_string();
    static ref BASE_URL: String = "https://www.google-analytics.com".to_string();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Params {
    #[serde(rename = "command_name")]
    pub command_name: String,
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

        Ok(builder.build("tc_key")?)
    }
    fn prepare_event(command_name: &str) -> anyhow::Result<PostData> {
        Ok(PostData {
            base_url: BASE_URL.to_string(),
            api_secret: API_SECRET.to_string(),
            measurement_id: MEASUREMENT_ID.to_string(),
            client_id: PostData::create_client_id()?,
            events: vec![Event {
                name: "track_cli_usage".to_string(),
                params: Params { command_name: command_name.to_string() },
            }],
        })
    }

    pub async fn send_event(command_name: &str) -> Result<(), anyhow::Error> {
        let post_data = PostData::prepare_event(command_name)?;
        tracing::info!("Sending event: {:?}", post_data);
        let request = reqwest::Request::try_from(post_data)?;
        let client = reqwest::Client::new();
        let response = client.execute(request).await?;
        let _ = response.text().await?;
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

        let _ = request
            .body_mut()
            .insert(reqwest::Body::from(serde_json::to_string(&value)?));
        Ok(request)
    }
}
