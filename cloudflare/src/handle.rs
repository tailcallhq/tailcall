use std::borrow::Borrow;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use hyper::{Body, Method, Request, Response};
use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::http::{
    graphiql, handle_request, showcase_get_app_ctx, AppContext, ShowcaseResources,
};
use tailcall::EnvIO;

use crate::env::CloudflareEnv;
use crate::http::{to_request, to_response};
use crate::{init_cache, init_file, init_http};

lazy_static! {
    static ref APP_CTX: RwLock<Option<(String, Arc<AppContext>)>> = RwLock::new(None);
}
///
/// The handler which handles requests on cloudflare
///
pub async fn fetch(
    req: worker::Request,
    env: worker::Env,
    _: worker::Context,
) -> anyhow::Result<worker::Response> {
    log::info!(
        "{} {:?}",
        req.method().to_string(),
        req.url().map(|u| u.to_string())
    );
    let req = to_request(req).await?;

    // Quick exit to GraphiQL
    //
    // Has to be done here, since when using GraphiQL, a config query parameter is not specified,
    // and get_app_ctx will fail without it.
    if req.method() == Method::GET {
        return to_response(graphiql(&req)?).await;
    }

    let env = Rc::new(env);
    let app_ctx = match get_app_ctx(env, &req).await? {
        Ok(app_ctx) => app_ctx,
        Err(e) => return to_response(e).await,
    };
    let resp = handle_request::<GraphQLRequest>(req, app_ctx).await?;
    to_response(resp).await
}

///
/// Initializes the worker once and caches the app context
/// for future requests.
///
async fn get_app_ctx(
    env: Rc<worker::Env>,
    req: &Request<Body>,
) -> anyhow::Result<Result<Arc<AppContext>, Response<Body>>> {
    // Read context from cache
    let file_path = req
        .uri()
        .query()
        .and_then(|x| serde_qs::from_str::<HashMap<String, String>>(x).ok())
        .and_then(|x| x.get("config").cloned());

    if let Some(file_path) = &file_path {
        if let Some(app_ctx) = read_app_ctx() {
            if app_ctx.0 == file_path.borrow() {
                log::info!("Using cached application context");
                return Ok(Ok(app_ctx.clone().1));
            }
        }
    }

    // Create new context
    let env_io = Arc::new(CloudflareEnv::init(env.clone()));

    let bucket_id = env_io
        .get("BUCKET")
        .ok_or(anyhow!("CONFIG var is not set"))?;
    log::debug!("R2 Bucket ID: {}", bucket_id);

    let resources = ShowcaseResources {
        http: init_http(),
        file: Some(init_file(env.clone(), bucket_id)?),
        env: Some(env_io),
        cache: init_cache(env),
    };

    match showcase_get_app_ctx::<GraphQLRequest>(req, resources).await? {
        Ok(app_ctx) => {
            let app_ctx = Arc::new(app_ctx);
            if let Some(file_path) = file_path {
                *APP_CTX.write().unwrap() = Some((file_path, app_ctx.clone()));
            }
            log::info!("Initialized new application context");
            Ok(Ok(app_ctx))
        }
        Err(e) => Ok(Err(e)),
    }
}

fn read_app_ctx() -> Option<(String, Arc<AppContext>)> {
    APP_CTX.read().unwrap().clone()
}
