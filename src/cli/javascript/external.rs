use anyhow::bail;

use crate::WorkerIO;

use super::channel::Message;

pub struct ExternalRuntime {
    url: String,
}

impl ExternalRuntime {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[async_trait::async_trait]
impl WorkerIO<Message, Message> for ExternalRuntime {
    async fn dispatch(&self, event: Message) -> anyhow::Result<Message> {
        log::debug!("event: {:?}", event);

        // Create a client
        let client = reqwest::Client::new();

        // Send the POST request
        let res = client.post(&self.url).json(&event).send().await?;

        // Check if the response status is success
        if res.status().is_success() {
            // Parse the response JSON
            let command: Message = res.json().await?;

            // Print the updated payload
            // println!("Updated Payload: {:?}", command);
            Ok(command)
        } else {
            // Handle error response
            bail!("Failed to get a success response: {:?}", res.status());
        }
    }
}
