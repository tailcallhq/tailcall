//! # Apollo Schema reporting
//!
//! Implementation of the apollo Schema Reporting Protocol
//! <https://www.apollographql.com/docs/studio/schema/schema-reporting/>
use async_graphql::dynamic::Schema;

use reqwest::Client;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const SCHEMA_URL: &str = "https://graphql.api.apollographql.com/api/graphql,";
const TARGET_LOG: &str = "apollo-studio-extension-register";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const RUNTIME_VERSION: &str = "Rust - No runtime version provided yet";

/**
 * Compute the SHA256 of a Schema
 * Usefull for Apollo Studio
 */
pub fn sha(schema: &Schema) -> String {
    let mut hasher = Sha256::new();
    let schema_sdl = schema.sdl();
    let schema_bytes = schema_sdl.as_bytes();
    hasher.update(schema_bytes);
    let sha_from_schema = Sha256::digest(schema_bytes);
    format!("{:x}", sha_from_schema)
}

/// Register your schema to Apollo Studio
///
/// * `authorization_token` - Token to send schema to apollo Studio.
/// * `schema` - async_graphql generated schema.
/// * `server_id` - An ID that's unique for each instance of your edge server. Unlike bootId, this value should persist across an instance's restarts. In a Kubernetes cluster, this might be the pod name, whereas the container can restart.
/// * `variant` - The name of the graph variant to register the schema to. The default value is current.
/// * `user_version` - An arbitrary string you can set to distinguish data sent by different versions of your edge server. For example, this can be the SHA of the Git commit for your deployed server code. We plan to make this value visible in Apollo Studio.
/// * `platform` - The infrastructure environment that your edge server is running in (localhost, kubernetes/deployment, aws lambda, google cloud run, google cloud function, AWS ECS, etc.)
// #[instrument(err, skip(authorization_token, schema))]
pub async fn register(
    authorization_token: &str,
    schema: &Schema,
    _server_id: &str,
    _variant: &str,
    _user_version: &str,
    _platform: &str,
) -> anyhow::Result<()> {
    // info!(
    //     target: TARGET_LOG,
    //     message = "Apollo Studio - Register Schema"
    // );
    let client = Client::new();
    let schema_sdl = schema.sdl();
    println!("Schema SDL {schema_sdl:?}");
    let _sha_from_schema = sha(schema);
    let _boot_id = Uuid::new_v4();

    let query = r#"
        mutation PublishSubgraphSchema($graphId: ID!, $variantName: String!, $subgraphName: String!, $schemaDocument: PartialSchemaInput!, $url: String, $revision: String!) {
          graph(id: $graphId) {
            publishSubgraph(graphVariant: $variantName, activePartialSchema: $schemaDocument, name: $subgraphName, url: $url, revision: $revision) {
              launchUrl
              updatedGateway
              wasCreated
            }
          }
        }
        "#.to_string();

    let body = serde_json::json!({
        "query": query,
        "variables": {
            "graphId": "tailcall",
            "variantName": "current",
            "subgraphName": "tailcall-test-3",
            "schemaDocument": schema_sdl,
            "url": "https://b5ab-2401-4900-71d2-4117-1997-5490-eb62-ca8b.ngrok-free.app",
            "revision": "1.3"
        },
    });
    println!("{body}");

    let result = client
        .post(SCHEMA_URL)
        .json(&body)
        .header("content-type", "application/json")
        .header("X-Api-Key", authorization_token)
        .send()
        .await;

    match result {
        Ok(data) => {
            // info!(
            //     target: TARGET_LOG,
            //     message = "Schema correctly registered",
            //     response = &tracing::field::debug(&data)
            // );
            let text = data.text().await;
            println!("{text:?}");
            // debug!(target: TARGET_LOG, data = ?text);
            Ok(())
        }
        Err(err) => {
            let _status_code = err.status();
            // error!(target: TARGET_LOG, status = ?status_code, error = ?err);
            Err(anyhow::anyhow!(err))
        }
    }
}
