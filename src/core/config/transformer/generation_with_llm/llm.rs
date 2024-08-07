use genai::client::Client;

const MODEL: &str = "gemini-1.5-flash-latest";

pub struct LLMClient {
    client: Client,
    retry_count: u8,
    model: String,
}

impl Default for LLMClient {
    fn default() -> Self {
        Self {
            client: Default::default(),
            retry_count: Default::default(),
            model: "gemini-1.5-flash-latest".into(),
        }
    }
}
