use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use hyper::{Body, Request, Response};
use lazy_static::lazy_static;
use tailcall::core::app_context::AppContext;
use tailcall::core::async_graphql_hyper::GraphQLRequest;
use tailcall::core::http::{handle_request, showcase};

use crate::http::{to_request, to_response};
use crate::runtime;

lazy_static! {
    static ref APP_CTX: RwLock<Option<(String, Arc<AppContext>)>> = RwLock::new(None);
}
///
/// The handler which handles requests on cloudflare
pub async fn fetch(
    req: worker::Request,
    env: worker::Env,
    _: worker::Context,
) -> anyhow::Result<worker::Response> {
    tracing::info!(
        "{} {:?}",
        req.method().to_string(),
        req.url().map(|u| u.to_string())
    );
    let req = to_request(req).await?;

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
            if app_ctx.0.eq(file_path) {
                tracing::info!("Using cached application context");
                return Ok(Ok(app_ctx.clone().1));
            }
        }
    }

    let runtime = runtime::init(env)?;
    match showcase::create_app_ctx::<GraphQLRequest>(req, runtime, true).await? {
        Ok(app_ctx) => {
            let app_ctx: Arc<AppContext> = Arc::new(app_ctx);
            if let Some(file_path) = file_path {
                *APP_CTX.write().unwrap() = Some((file_path, app_ctx.clone()));
            }
            tracing::info!("Initialized new application context");
            Ok(Ok(app_ctx))
        }
        Err(e) => Ok(Err(e)),
    }
}

fn read_app_ctx() -> Option<(String, Arc<AppContext>)> {
    APP_CTX.read().unwrap().clone()
}
