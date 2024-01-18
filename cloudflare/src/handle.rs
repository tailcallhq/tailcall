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
use crate::{init_env, init_file, init_http};

type CloudFlareAppContext = AppContext<CloudflareHttp, CloudflareEnv>;
lazy_static! {
  static ref APP_CTX: RwLock<Option<Arc<CloudFlareAppContext>>> = RwLock::new(None);
}
///
/// The main fetch handler that handles requests on cloudflare
///
pub async fn fetch(req: worker::Request, env: worker::Env, _: worker::Context) -> anyhow::Result<worker::Response> {
  let env = Rc::new(env);
  log::debug!("Execution starting");
  let file_path = format!(
    "{}.graphql",
    req.path().strip_prefix('/').ok_or(anyhow!("invalid prefix"))?
  );
  let app_ctx = get_app_ctx(env, file_path).await?;
  let resp = handle_request::<GraphQLRequest, CloudflareHttp, CloudflareEnv>(to_request(req).await?, app_ctx).await?;
  Ok(to_response(resp).await?)
}

///
/// Reads the configuration from the CONFIG environment variable.
///
async fn get_config(env_io: &impl EnvIO, env: Rc<worker::Env>, file_path: String) -> anyhow::Result<Config> {
  let bucket_id = env_io.get("BUCKET").ok_or(anyhow!("CONFIG var is not set"))?;
  let file_io = init_file(env.clone(), bucket_id);
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
    Ok(app_ctx.clone())
  } else {
    // Create new context
    let env_io = init_env(env.clone());
    let cfg = get_config(&env_io, env.clone(), file_path).await?;
    let blueprint = Blueprint::try_from(&cfg)?;
    let h_client = Arc::new(init_http());

    let app_ctx = Arc::new(AppContext::new(blueprint, h_client.clone(), h_client, Arc::new(env_io)));
    *APP_CTX.write().unwrap() = Some(app_ctx.clone());
    log::info!("Initialized new application context");
    Ok(app_ctx)
  }
}

fn read_app_ctx() -> Option<Arc<CloudFlareAppContext>> {
  APP_CTX.read().unwrap().clone()
}
