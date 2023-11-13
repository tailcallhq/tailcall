use std::ops::Add;
use std::sync::atomic::AtomicPtr;
use std::sync::Arc;
use std::time::Duration;

use imara_diff::intern::InternedInput;
use imara_diff::{diff, Algorithm};
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
  pub fn new(
    file_path: Vec<String>,
    refresh_interval: u64,
    state: Arc<AtomicPtr<ServerContext>>,
  ) -> Result<Self, CLIError> {
    Ok(Self { file_path, refresh_interval, state })
  }

  pub async fn start_polling(&self) {
    let client = Client::new();
    let state = Arc::clone(&self.state);
    let fp = self.file_path.clone();
    let duration = Duration::from_secs(self.refresh_interval);
    let mut interval = time::interval(duration);
    tokio::spawn(async move {
      loop {
        if make_request(&state, &client, &fp).await {
          interval = time::interval(interval.period().add(duration));
          log::debug!("The refresh interval is doubled.");
        } else {
          interval = time::interval(duration);
          log::debug!(
            "The refresh was successful. The polling interval has been reset, otherwise it remains constant."
          );
        }
        if interval.period().as_secs() > 99 {
          interval = time::interval(duration);
        }
        interval.reset();
        interval.tick().await;
      }
    });
  }
}

async fn make_request(state: &Arc<AtomicPtr<ServerContext>>, client: &Client, file_paths: &Vec<String>) -> bool {
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
        return true;
      }
    };

    if !resp.status().is_success() {
      log::error!("Unknown error. Exited with status code: {}", resp.status().as_u16());
      return true;
    }
    let updated_txt = match resp.text().await {
      Ok(updated_sdl) => updated_sdl,
      Err(_) => return true,
    };
    update_txt(state, &updated_txt).await;
  }
  false
}

async fn update_txt(state: &Arc<AtomicPtr<ServerContext>>, updated_txt: &str) {
  if let Ok(t) = Source::try_parse_and_detect(updated_txt) {
    match t {
      Source::Json => {
        if let Ok(config) = Config::from_json(updated_txt) {
          let state = unsafe { state.load(std::sync::atomic::Ordering::Relaxed).as_mut().unwrap() };
          let _ = update_schema(state, config);
        }
      }
      Source::Yml => {
        if let Ok(config) = Config::from_yaml(updated_txt) {
          let state = unsafe { state.load(std::sync::atomic::Ordering::Relaxed).as_mut().unwrap() };
          let _ = update_schema(state, config);
        }
      }
      Source::GraphQL => match Config::from_sdl(updated_txt) {
        Valid(updated_config) => {
          let state = unsafe { state.load(std::sync::atomic::Ordering::Relaxed).as_mut().unwrap() };
          if let Ok(config) = updated_config {
            let _ = update_schema(state, config);
          }
        }
      },
    }
  };
}

fn update_schema(state: &mut ServerContext, config: Config) -> anyhow::Result<()> {
  match Blueprint::try_from(&config) {
    Ok(blueprint) => {
      let new_schema = SchemaLoader::new_schema(blueprint.to_schema());
      let old_schema_str = state.schema.get_schema()?.sdl();
      let new_schema_str = new_schema.get_schema()?.sdl();
      state.schema = new_schema;
      log::debug!("{}", compare_schemas(new_schema_str, old_schema_str));
      Ok(())
    }
    Err(e) => {
      log::error!("Failed to create blueprint: {}", e);
      Ok(())
    }
  }
}

fn compare_schemas(new_schema_str: String, old_schema_str: String) -> String {
  let changed = new_schema_str.eq(&old_schema_str);
  return if changed {
    "Schema is the same".to_string()
  } else {
    let input = InternedInput::new(old_schema_str.as_str(), new_schema_str.as_str());
    let diff = diff(Algorithm::Myers, &input, imara_diff::UnifiedDiffBuilder::new(&input));
    log::debug!("{diff}");
    "Successfully updated schema".to_string()
  };
}
