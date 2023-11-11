use std::sync::atomic::AtomicPtr;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::time;

use crate::blueprint::Blueprint;
use crate::cli::CLIError;
use crate::config::{Config, Source};
use crate::http::{SchemaLoader, ServerContext};
use crate::valid::Valid;
pub struct ConfigLoader {
  file_path: Vec<String>,
  refresh_interval: u64,
  state: Arc<AtomicPtr<ServerContext>>,
}

impl ConfigLoader {
  pub fn new(file_path: Vec<String>, refresh_interval: u64, state: Arc<AtomicPtr<ServerContext>>) -> Result<Self, CLIError> {
    Ok(Self { file_path, refresh_interval, state })
  }

  pub async fn start_polling(&self) {
    let client = Client::new();
    let refresh_interval = self.refresh_interval;
    let state = Arc::clone(&self.state);
    let fp = self.file_path.clone();

    let mut interval = time::interval(Duration::from_secs(refresh_interval));

    tokio::spawn(async move {
      loop {
        interval.tick().await;
        make_request(&state, &client, &fp).await;
      }
    });
  }
}

async fn make_request(state: &Arc<AtomicPtr<ServerContext>>, client: &Client, file_paths: &Vec<String>) {
  for file_path in file_paths {
    if !(file_path.starts_with("http://") || file_path.starts_with("https://")) {
      continue;
    }
    let request = client.get(file_path).build().unwrap();

    let resp = client.execute(request).await;

    let resp = match resp {
      Ok(resp) => resp,
      Err(e) => {
        log::error!("Failed to refresh configuration: {}", e);
        return;
      }
    };

    if !resp.status().is_success() {
      log::info!("Unknown error.");
      return;
    }
    let updated_txt = match resp.text().await {
      Ok(updated_sdl) => updated_sdl,
      Err(_) => return,
    };
    update_txt(state, &updated_txt).await;
  }
}

async fn update_txt(state: &Arc<AtomicPtr<ServerContext>>, updated_txt: &String) {
  match Source::try_parse_and_detect(&updated_txt) {
    Ok(t) => {
      match t {
        Source::Json => {
          match Config::from_json(&updated_txt) {
            Ok(config) => {
              let state = unsafe { state.load(std::sync::atomic::Ordering::Relaxed).as_mut().unwrap() };
              update_schema(state,config);
            }
            _ => {}
          }
        }
        Source::Yml => {
          match Config::from_yaml(&updated_txt) {
            Ok(config) => {
              let state = unsafe { state.load(std::sync::atomic::Ordering::Relaxed).as_mut().unwrap() };
              update_schema(state,config);
            }
            _ => {}
          }
        }
        Source::GraphQL => {
          match Config::from_sdl(&updated_txt) {
            Valid(updated_config) => {
              let state = unsafe { state.load(std::sync::atomic::Ordering::Relaxed).as_mut().unwrap() };
              if let Ok(config) = updated_config {
                update_schema(state, config);
              }
            }
          }
        }
      }
    }
    Err(_) => {}
  };
}

fn update_schema(state: &mut ServerContext, config: Config) {
  match Blueprint::try_from(&config) {
    Ok(blueprint) => {
      state.schema = SchemaLoader::new_schema(blueprint.to_schema());
      log::info!("Schema updated successfully");
    }
    Err(e) => {
      log::error!("Failed to create blueprint: {}", e);
      return;
    }
  }
}
