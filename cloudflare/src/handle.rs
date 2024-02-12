use std::borrow::Borrow;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

use hyper::{Body, Method, Request, Response};
use lazy_static::lazy_static;
use tailcall::builder::TailcallExecutor;
use tailcall::http::graphiql;
use tailcall::http::showcase::create_tailcall_executor;

use crate::http::{to_request, to_response};
use crate::runtime;

lazy_static! {
    static ref APP_CTX: RwLock<Option<(String, TailcallExecutor)>> = RwLock::new(None);
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

    let tailcall_executor = match get_app_ctx(env, &req).await? {
        Ok(app_ctx) => app_ctx,
        Err(e) => return to_response(e).await,
    };
    let resp = tailcall_executor.execute(req).await?;
    to_response(resp).await
}

///
/// Initializes the worker once and caches the app context
/// for future requests.
///
async fn get_app_ctx(
    env: worker::Env,
    req: &Request<Body>,
) -> anyhow::Result<Result<TailcallExecutor, Response<Body>>> {
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
    let env = Rc::new(env);
    let runtime = runtime::init(env)?;
    match create_tailcall_executor(req, runtime, true).await? {
        Ok(tailcall) => {
            if let Some(file_path) = file_path {
                *APP_CTX.write().unwrap() = Some((file_path, tailcall.clone()));
            }
            log::info!("Initialized new tailcall executor");
            Ok(Ok(tailcall))
        }
        Err(e) => Ok(Err(e)),
    }
}

fn read_app_ctx() -> Option<(String, TailcallExecutor)> {
    APP_CTX.read().unwrap().clone()
}
