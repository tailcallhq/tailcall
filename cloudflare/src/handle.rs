use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::config::ConfigSet;
use tailcall::http::{graphiql, handle_request, AppContext};
use tailcall::EnvIO;

use crate::http::{to_request, to_response};
use crate::{init_cache, init_env, init_file, init_http, init_http2, init_proto_resolver};

lazy_static! {
    static ref APP_CTX: RwLock<Option<(String, Arc<AppContext>)>> = RwLock::new(None);
}
///
/// The handler which handles requests on cloudflare
///
pub async fn fetch(req: worker::Request, env: worker::Env) -> anyhow::Result<worker::Response> {
    log::info!(
        "{} {:?}",
        req.method().to_string(),
        req.url().map(|u| u.to_string())
    );
    let env = Rc::new(env);
    let hyper_req = to_request(req).await?;
    if hyper_req.method() == hyper::Method::GET {
        let response = graphiql(&hyper_req)?;
        return to_response(response).await;
    }
    let query = hyper_req
        .uri()
        .query()
        .ok_or(anyhow!("Unable parse extract query"))?;
    let query = serde_qs::from_str::<HashMap<String, String>>(query)?;
    let config_path = query
        .get("config")
        .ok_or(anyhow!("The key 'config' not found in the query"))?
        .clone();

    log::info!("config-url: {}", config_path);
    let app_ctx = get_app_ctx(env, config_path.as_str()).await?;
    let resp = handle_request::<GraphQLRequest>(hyper_req, app_ctx).await?;

    Ok(to_response(resp).await?)
}

///
/// Reads the configuration from the CONFIG environment variable.
///
async fn get_config(
    env_io: Arc<dyn EnvIO>,
    env: Rc<worker::Env>,
    file_path: &str,
) -> anyhow::Result<ConfigSet> {
    let bucket_id = env_io
        .get("BUCKET")
        .ok_or(anyhow!("BUCKET var is not set"))?;
    log::debug!("R2 Bucket ID: {}", bucket_id);
    let file_io = init_file(env.clone(), bucket_id)?;
    let http_io = init_http();
    let proto_resolver = init_proto_resolver();

    let reader = ConfigReader::init(file_io, http_io, proto_resolver);
    let config = reader.read(&file_path).await?;
    Ok(config)
}

///
/// Initializes the worker once and caches the app context
/// for future requests.
///
async fn get_app_ctx(env: Rc<worker::Env>, file_path: &str) -> anyhow::Result<Arc<AppContext>> {
    // Read context from cache
    if let Some(app_ctx) = read_app_ctx() {
        if app_ctx.0 == file_path {
            log::info!("Using cached application context");
            return Ok(app_ctx.clone().1);
        }
    }
    // Create new context
    let env_io = init_env(env.clone());
    let cfg = get_config(env_io.clone(), env.clone(), file_path).await?;
    log::info!("Configuration read ... ok");
    log::debug!("\n{}", cfg.to_sdl());
    let blueprint = Blueprint::try_from(&cfg)?;
    log::info!("Blueprint generated ... ok");
    let h_client = init_http();
    let h2_client = init_http2();
    let cache = init_cache(env);
    let app_ctx = Arc::new(AppContext::new(
        blueprint,
        h_client.clone(),
        h_client,
        env_io,
        cache,
    ));
    *APP_CTX.write().unwrap() = Some((file_path.to_string(), app_ctx.clone()));
    log::info!("Initialized new application context");
    Ok(app_ctx)
}

fn read_app_ctx() -> Option<(String, Arc<AppContext>)> {
    APP_CTX.read().unwrap().clone()
}
