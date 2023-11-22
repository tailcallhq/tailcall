use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use url::Url;

use crate::blueprint::Blueprint;
use crate::config::reader::ConfigReader;
use crate::config::Config;
use crate::http::ServerContext;

pub struct ConfigLoader {
  file_paths: HashMap<String, u64>,
  poll_interval: u64,
}

impl ConfigLoader {
  pub fn init(file_paths: Vec<String>, poll_interval: u64) -> ConfigLoader {
    let mut path_hm = HashMap::new();
    for file_path in file_paths {
      path_hm.insert(file_path, poll_interval);
    }
    Self { file_paths: path_hm, poll_interval }
  }
  pub async fn start_polling(&mut self, batch_single: Arc<ServerContext>) {
    loop {
      let conf = handle_poll(&mut self.file_paths, self.poll_interval).await; // the function will make sure that there are no unwanted memory allocations
      if let Ok(bp) = Blueprint::try_from(&conf) {
        let old_sdl = batch_single.schema.read().await.sdl();
        let new_schema = bp.to_schema();
        if old_sdl.ne(&new_schema.sdl()) {
          log::debug!("Schema changed");
          *batch_single.schema.write().await = new_schema.clone();
        } else {
          log::debug!("Schema is not changed");
        }
      }
      drop(conf);
    }
  }
}

async fn handle_poll(file_paths: &mut HashMap<String, u64>, poll_interval: u64) -> Config {
  let mut cd = Config::default();
  let mut handles = vec![];
  for (path, dur) in &mut *file_paths {
    let path = path.clone();
    let mut dur = *dur;
    let join_handle = tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_secs(dur));
      interval.reset();
      interval.tick().await;
      let conf = if let Ok(url) = Url::parse(&path) {
        ConfigReader::from_url(url).await
      } else {
        let path = path.trim_end_matches('/');
        ConfigReader::from_file_path(path).await
      };
      if let Ok(conf) = conf {
        log::debug!("Poll reset");
        dur = poll_interval;
        (path, dur, Some(conf))
      } else {
        if dur > 99 {
          log::debug!("Poll reset");
          dur = poll_interval;
        } else {
          log::debug!("Poll duration doubled");
          dur <<= 1;
        }
        (path, dur, None)
      }
    });
    handles.push(join_handle);
  }
  for handle in handles {
    let (a, b, c) = handle.await.unwrap();
    file_paths.insert(a, b);
    if let Some(conf) = c {
      cd = cd.clone().merge_right(&conf);
    }
  }
  cd
}
