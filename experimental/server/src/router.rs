use crate::{
    blueprint::{Blueprint, BlueprintDefinition},
    digest::Digest,
    registry::SchemaRegistry,
};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

type RouterState = Arc<Mutex<SchemaRegistry>>;

#[derive(Deserialize)]
struct PublishSchemaPayload {
    definitions: Vec<BlueprintDefinition>,
}

async fn publish_schema(
    State(registry): State<Arc<Mutex<SchemaRegistry>>>,
    payload: String,
) -> impl IntoResponse {
    let digest = Digest::from_bytes(payload.as_bytes());
    let payload = serde_json::from_str::<PublishSchemaPayload>(payload.as_str()).unwrap();
    let blueprint = Blueprint::new(digest.clone(), payload.definitions);
    let mut registry = registry.lock().unwrap();

    if let Some(_) = registry.add(digest, blueprint) {
        "blueprint updated"
    } else {
        "blueprint added"
    }
    // todo!("the cli client will send a PUT request here to publish a new schema")
}

async fn root() -> impl IntoResponse {
    "OK"
}

pub fn make_router(state: RouterState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/schemas", put(publish_schema))
        .with_state(state)
}
