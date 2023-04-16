use axum::Server;
use server::{registry::SchemaRegistry, router::make_router};
use std::sync::{Arc, Mutex};

const PORT: u16 = 8080;

#[tokio::main]
async fn main() {
    let registry = Arc::new(Mutex::new(SchemaRegistry::new()));
    let router = make_router(registry);
    let address = format!("127.0.0.1:{}", PORT);

    println!("GraphiQL IDE: {}", address);

    Server::bind(&(address.as_str()).parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
