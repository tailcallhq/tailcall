use futures_util::future;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper_staticfile::ResponseBuilder;

async fn serve_static(req: Request<Body>) -> Result<Response<Body>, std::io::Error> {
  // remove extension from path in request "users.json" becomes "users"
  //   req.uri().path().trim_end_matches(".json");
  hyper_staticfile::resolve_path("examples/server/data/", format!("{}.json", req.uri().path()).as_str())
    .await
    .map(|result| {
      ResponseBuilder::new()
        .request(&req)
        .build(result)
        .expect("unable to build response")
    })
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let make_svc = make_service_fn(|_| future::ok::<_, hyper::Error>(service_fn(move |req| serve_static(req))));

  // make_service_fn(|_conn| async { Ok::<_, Infallible>(hyper::service::service_fn(serve_static, static_)) });

  let addr = ([127, 0, 0, 1], 3000).into();

  let server = Server::bind(&addr).serve(make_svc);

  println!("Listening on http://{}", addr);

  server.await?;

  Ok(())
}
