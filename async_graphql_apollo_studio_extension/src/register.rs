//! # Apollo Schema reporting
//!
//! Implementation of the apollo Schema Reporting Protocol
//! <https://www.apollographql.com/docs/studio/schema/schema-reporting/>
use async_graphql::dynamic::Schema;

use reqwest::Client;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const SCHEMA_URL: &str = "https://schema-reporting.api.apollographql.com/api/graphql";
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
#[instrument(err, skip(authorization_token, schema))]
#[allow(clippy::too_many_arguments)]
pub async fn register(
    authorization_token: &str,
    schema: &Schema,
    server_id: &str,
    variant: &str,
    user_version: &str,
    platform: &str,
) -> anyhow::Result<()> {
    info!(
        target: TARGET_LOG,
        message = "Apollo Studio - Register Schema"
    );
    let client = Client::new();
    let schema_sdl = schema.sdl();
    let sha_from_schema = sha(schema);
    let boot_id = Uuid::new_v4();

    let mutation = format!(
        r#"
        mutation($schema: String!) {{
            me {{
              ... on ServiceMutation {{
                reportServerInfo(
                  info: {{
                    bootId: "{:?}"
                    serverId: "{}"
                    executableSchemaId: "{}"
                    graphVariant: "{}"
                    platform: "{}"
                    libraryVersion: "async-studio-extension {}"
                    runtimeVersion: "{}"
                    userVersion: "{}"        
                  }}
                  executableSchema: $schema
                ) {{
                  __typename
                  ... on ReportServerInfoError {{
                    code
                    message
                  }}
                  inSeconds
                  withExecutableSchema
                }}
              }}
            }}
          }}
        "#,
        boot_id,
        server_id,
        sha_from_schema,
        variant,
        platform,
        VERSION,
        RUNTIME_VERSION,
        user_version
    );

    let body = serde_json::json!({
        "query": mutation,
        "variables": {
            "schema": schema_sdl,
        },
    });

    println!("Request body: {body:?}");

    let result = client
        .post(SCHEMA_URL)
        .json(&body)
        .header("content-type", "application/json")
        .header("X-Api-Key", authorization_token)
        .send()
        .await;

    match result {
        Ok(data) => {
            info!(
                target: TARGET_LOG,
                message = "Schema correctly registered",
                response = &tracing::field::debug(&data)
            );
            let text = data.text().await;
            println!("api key: {authorization_token}");
            println!("response: {text:?}");
            debug!(target: TARGET_LOG, data = ?text);
            Ok(())
        }
        Err(err) => {
            let status_code = err.status();
            error!(target: TARGET_LOG, status = ?status_code, error = ?err);
            Err(anyhow::anyhow!(err))
        }
    }
}
