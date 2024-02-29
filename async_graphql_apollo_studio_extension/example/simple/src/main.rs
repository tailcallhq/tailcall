use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_extension_apollo_tracing::{
    register::register, ApolloTracing, ApolloTracingDataExt,
};
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use starwars::{QueryRoot, StarWars};
use tokio::net::TcpListener;
use tracing_subscriber::Layer;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt,
    registry::{LookupSpan, Registry},
};

mod graphql_service;
use graphql_service::GraphQL;

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() {
    let subscriber = Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(StarWars::new())
        .extension(ApolloTracing::new(
            "AUTH_KEY".into(),
            "mac-local".into(),
            "testblbl".into(),
            "new".into(),
            "v1.0.0".into(),
        ))
        .finish();

    let err = register(
        "AUTH_KEY",
        &schema,
        "my-allocation-id",
        "new",
        "1.0.0",
        "prod",
    )
    .await;

    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    println!("GraphiQL IDE: http://localhost:8000");

    axum::serve(TcpListener::bind("127.0.0.1:8000").await.unwrap(), app)
        .await
        .unwrap();
}
