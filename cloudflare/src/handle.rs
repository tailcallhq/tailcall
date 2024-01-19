use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::config::Config;
use tailcall::http::{handle_request, AppContext};
use tailcall::EnvIO;

use crate::env::CloudflareEnv;
use crate::http::{to_request, to_response, CloudflareHttp};
use crate::{init_cache, init_env, init_file, init_http};

type CloudFlareAppContext = AppContext<CloudflareHttp, CloudflareEnv>;
lazy_static! {
  static ref APP_CTX: RwLock<Option<Arc<CloudFlareAppContext>>> = RwLock::new(None);
}
///
/// The main fetch handler that handles requests on cloudflare
///
pub async fn fetch(req: worker::Request, env: worker::Env, _: worker::Context) -> anyhow::Result<worker::Response> {
  let env = Rc::new(env);
  log::info!("{} {}", req.method().to_string(), req.path());
  let route = req
    .path()
    .strip_prefix('/')
    .ok_or(anyhow!("invalid prefix"))?
    .to_string();
  let file_path = match route.ends_with("/graphql") {
    true => route.replace("/graphql", ".graphql").to_string(),
    false => {
      format!("{}.graphql", route)
    }
  };
  let app_ctx = get_app_ctx(env, file_path).await?;
  let resp = handle_request::<GraphQLRequest, CloudflareHttp, CloudflareEnv>(to_request(req).await?, app_ctx).await?;
  Ok(to_response(resp).await?)
}

///
/// Reads the configuration from the CONFIG environment variable.
///
async fn get_config(env_io: &impl EnvIO, env: Rc<worker::Env>, file_path: String) -> anyhow::Result<Config> {
  let bucket_id = env_io.get("BUCKET").ok_or(anyhow!("CONFIG var is not set"))?;
  log::debug!("R2 Bucket ID: {}", bucket_id);
  let file_io = init_file(env.clone(), bucket_id)?;
  let http_io = init_http();
  let reader = ConfigReader::init(file_io, http_io);
  let config = reader.read(&[file_path]).await?;
  Ok(config)
}

///
/// Initializes the worker once and caches the app context
/// for future requests.
///
async fn get_app_ctx(env: Rc<worker::Env>, file_path: String) -> anyhow::Result<Arc<CloudFlareAppContext>> {
  // Read context from cache
  if let Some(app_ctx) = read_app_ctx() {
    log::info!("Using cached application context");
    Ok(app_ctx.clone())
  } else {
    // Create new context
    let env_io = init_env(env.clone());
    let cfg = get_config(&env_io, env.clone(), file_path).await?;
    log::info!("Configuration read ... ok");
    log::debug!("\n{}", cfg.to_sdl());
    let blueprint = Blueprint::try_from(&cfg).map_err(|e| {
      log::error!("Blueprint generation failed: {}", e);
      e
    })?;
    log::info!("Blueprint generated ... ok");
    let h_client = Arc::new(init_http());
    let cache = Arc::new(init_cache(env));

    let app_ctx = Arc::new(AppContext::new(
      blueprint,
      h_client.clone(),
      h_client,
      Arc::new(env_io),
      cache,
    ));
    *APP_CTX.write().unwrap() = Some(app_ctx.clone());
    log::info!("Initialized new application context");
    Ok(app_ctx)
  }
}

fn read_app_ctx() -> Option<Arc<CloudFlareAppContext>> {
  APP_CTX.read().unwrap().clone()
}
