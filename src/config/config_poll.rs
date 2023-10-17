use std::sync::atomic::AtomicPtr;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::time;

use crate::blueprint::Blueprint;
use crate::cli::CLIError;
use crate::config::Config;
use crate::http::ServerContext;

pub struct ConfigLoader {
  file_path: String,
  refresh_interval: u64,
  state: Arc<AtomicPtr<ServerContext>>,
}

impl ConfigLoader {
  pub fn new(file_path: String, refresh_interval: u64, state: Arc<AtomicPtr<ServerContext>>) -> Result<Self, CLIError> {
    Ok(Self { file_path, refresh_interval, state })
  }

  pub async fn start_polling(&self) {
    let client = Client::new();
    let refresh_interval = self.refresh_interval;
    let file_path = self.file_path.clone();
    let state = Arc::clone(&self.state);

    let mut interval = time::interval(Duration::from_secs(refresh_interval));

    tokio::spawn(async move {
      loop {
        interval.tick().await;

        let request = client.get(&file_path).build().unwrap();

        let resp = client.execute(request).await;

        let resp = match resp {
          Ok(resp) => resp,
          Err(e) => {
            log::error!("Failed to refresh configuration: {}", e);
            continue;
          }
        };

        if !resp.status().is_success() {
          log::info!("Unknown error.");
          continue;
        }

        let updated_sdl = match resp.text().await {
          Ok(updated_sdl) => updated_sdl,
          Err(_) => continue,
        };

        match Config::from_sdl(&updated_sdl) {
          Ok(updated_config) => {
            let state = unsafe { state.load(std::sync::atomic::Ordering::Relaxed).as_mut().unwrap() };
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
