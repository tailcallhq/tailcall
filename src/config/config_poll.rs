use async_graphql::dynamic;
use tokio::sync::RwLock;
use url::Url;

use crate::blueprint::Blueprint;
use crate::config::reader::ConfigReader;
use crate::config::Config;

pub async fn fetch_once(file_paths: &Vec<String>, schema: &RwLock<dynamic::Schema>) {
  let conf = handle_fetch(file_paths).await;

  if let Ok(bp) = Blueprint::try_from(&conf) {
    let old_sdl = schema.read().await.sdl();
    let new_schema = bp.to_schema();
    if old_sdl.ne(&new_schema.sdl()) {
      log::debug!("Schema updated");
      *schema.write().await = new_schema.clone();
    } else {
      log::debug!("Schema not updated");
    }
  }
}

async fn handle_fetch(file_paths: &Vec<String>) -> Config {
  let mut config = Config::default();
  let mut handles = vec![];
  for path in file_paths {
    let path = path.clone();
    let join_handle = tokio::spawn(async move {
      let conf = if let Ok(url) = Url::parse(&path) {
        ConfigReader::from_url(url).await
      } else {
        let path = path.trim_end_matches('/');
        ConfigReader::from_file_path(path).await
      };
      log::debug!("Poll reset");
      conf
    });
    handles.push(join_handle);
  }
  for handle in handles {
    let conf = handle.await.unwrap();
    if let Ok(conf) = conf {
      config = config.clone().merge_right(&conf);
    }
  }
  config
}
