use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use hyper::header::HeaderValue;
use reqwest::header::IF_NONE_MATCH;
use reqwest::Client;
use tokio::time;

use crate::blueprint::Blueprint;
use crate::cli::CLIError;
use crate::config::Config;
use crate::http::ServerContext;

pub struct ConfigLoader {
  file_path: String,
  refresh_interval: u64,
  state: Arc<RwLock<ServerContext>>,
}

impl ConfigLoader {
  pub fn new(file_path: String, refresh_interval: u64, state: Arc<RwLock<ServerContext>>) -> Result<Self, CLIError> {
    Ok(Self { file_path, refresh_interval, state })
  }

  pub async fn start_polling(&self) {
    let client = Client::new();
    let refresh_interval = self.refresh_interval;
    let file_path = self.file_path.clone();
    let state = Arc::clone(&self.state);

    let mut interval = time::interval(Duration::from_secs(refresh_interval));
    let mut etag: Option<String> = None;

    tokio::spawn(async move {
      loop {
        interval.tick().await;

        let mut headers = HashMap::new();

        if let Some(etag_value) = &etag {
          headers.insert(IF_NONE_MATCH, HeaderValue::from_str(etag_value).unwrap());
        }

        let mut resp = client.get(&file_path);

        for (k, v) in headers {
          resp = resp.header(k, v);
        }
        let resp = resp.send().await;

        let resp = match resp {
          Ok(resp) => resp,
          Err(e) => {
            log::error!("Failed to refresh configuration: {}", e);
            continue;
          }
        };

        if resp.status() == 304 {
          log::info!("The resource has not been modified.");
          continue;
        }

        if !resp.status().is_success() {
          log::info!("Unknown error.");
          continue;
        }

        if let Some(new_etag) = resp.headers().get("etag") {
          etag = Some(new_etag.to_str().unwrap().to_string());
        }

        let updated_sdl = match resp.text().await {
          Ok(updated_sdl) => updated_sdl,
          Err(_) => continue,
        };

        match Config::from_sdl(&updated_sdl) {
          Ok(updated_config) => {
            let mut state = state.write().unwrap();
            match Blueprint::try_from(&updated_config) {
              Ok(blueprint) => {
                state.schema = blueprint.to_schema();
                log::info!("Schema updated sucessfuly");
              }
              Err(e) => {
                log::error!("Failed to create blueprint: {}", e);
                continue;
              }
            }
          }
          Err(_) => continue,
        };
      }
    });
  }
}
