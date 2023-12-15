use std::sync::RwLock;
use lazy_static::lazy_static;
use worker::*;
use worker::Fetch::Url;
use worker::wasm_bindgen::JsCast;
use worker::wasm_bindgen_futures::JsFuture;
// use worker_sys::web_sys::{Request, Response};

/*use std::sync::Arc;
use hyper::Body;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::cli::CLIError;
use tailcall::config::Config;
use tailcall::http::{DefaultHttpClient, handle_request, ServerContext};
*/
lazy_static! {
    static ref SDL: RwLock<String> = RwLock::new(String::new());
}
// #[event(f)]

#[event(start)]
fn start() {
/*    let u = "https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql";
    let window = web_sys::window().unwrap();
    async_std::task::block_on(async move {
        let mut resp = Url(
            "https://httpbin.org/anything"
                .parse().unwrap(),
        )
            .send().await.unwrap();
        let txt = resp.text().await.unwrap();
        *SDL.write().unwrap() = format!("{:#?}", txt);
    });
*/



    // *SDL.write().unwrap() = format!("{:?}",x);
    /*
  async_std::task::block_on(async {
        // let client = reqwest::Client::new();
        // let response = client.get(url).send().await.unwrap();
        let body = reqwest::get("https://www.rust-lang.org").await;

    });*/
}


#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let mut sdl = SDL.read().unwrap().clone();
    if sdl.is_empty() {
        let x = make_req().await.unwrap();
        sdl = x.clone();
        *SDL.write().unwrap() = x;
    }
    // handle_request::<GraphQLRequest>(hyper::Request::new(Body::from(req.text()?)), SERVER_CONTEXT.clone());
    Response::ok(sdl)
}

async fn make_req() -> Result<String>{
    let mut resp = Url("https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql".parse().unwrap()).send().await?;
    let txt = resp.text().await?;
    Ok(txt)
}
