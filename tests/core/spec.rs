extern crate core;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::{fs, panic};

use anyhow::Context;
use colored::Colorize;
use futures_util::future::join_all;
use hyper::{Body, Request};
use serde::{Deserialize, Serialize};
use tailcall::core::app_context::AppContext;
use tailcall::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::reader::ConfigReader;
use tailcall::core::config::transformer::Required;
use tailcall::core::config::{Config, ConfigModule, Source};
use tailcall::core::http::handle_request;
use tailcall::core::merge_right::MergeRight;
use tailcall::core::print_schema::print_schema;
use tailcall::core::valid::{Cause, ValidationError, Validator};
use tailcall_prettier::Parser;

use super::file::File;
use super::http::Http;
use super::model::*;
use super::runtime::ExecutionSpec;
use crate::core::runtime;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
struct SDLError {
    message: String,
    trace: Vec<String>,
    description: Option<String>,
}

impl<'a> From<Cause<&'a str>> for SDLError {
    fn from(value: Cause<&'a str>) -> Self {
        SDLError {
            message: value.message.to_string(),
            trace: value.trace.iter().map(|e| e.to_string()).collect(),
            description: None,
        }
    }
}

impl From<Cause<String>> for SDLError {
    fn from(value: Cause<String>) -> Self {
        SDLError {
            message: value.message.to_string(),
            trace: value.trace.iter().map(|e| e.to_string()).collect(),
            description: value.description,
        }
    }
}

async fn is_sdl_error(spec: ExecutionSpec, mock_http_client: Arc<Http>) -> bool {
    if spec.sdl_error {
        // errors: errors are expected, make sure they match
        let (source, content) = &spec.server[0];

        let config = source.decode(content);

        let config = match config {
            Ok(config) => {
                let mut runtime = runtime::create_runtime(mock_http_client, spec.env.clone(), None);
                runtime.file = Arc::new(File::new(spec.clone()));
                let reader = ConfigReader::init(runtime);
                match reader.resolve(config, spec.path.parent()).await {
                    Ok(config) => Blueprint::try_from(&config),
                    Err(e) => Err(ValidationError::new(e.to_string())),
                }
            }
            Err(e) => Err(e),
        };

        match config {
            Ok(_) => {
                tracing::error!("\terror FAIL");
                panic!(
                    "Spec {} {:?} with \"sdl error\" directive did not have a validation error.",
                    spec.name, spec.path
                );
            }
            Err(cause) => {
                let errors: Vec<SDLError> =
                    cause.as_vec().iter().map(|e| e.to_owned().into()).collect();

                let snapshot_name = format!("{}_error", spec.safe_name);

                insta::assert_json_snapshot!(snapshot_name, errors);
            }
        };

        return true;
    }
    false
}

async fn check_server_config(spec: ExecutionSpec) -> Vec<Config> {
    let mut server: Vec<Config> = Vec::with_capacity(spec.server.len());

    for (i, (source, content)) in spec.server.iter().enumerate() {
        let config = Config::from_source(source.to_owned(), content).unwrap_or_else(|e| {
            panic!(
                "Couldn't parse GraphQL in server definition #{} of {:#?}: {}",
                i + 1,
                spec.path,
                e
            )
        });

        let config = Config::default().merge_right(config);

        // TODO: we should probably figure out a way to do this for every test
        // but GraphQL identity checking is very hard, since a lot depends on the code
        // style the re-serializing check gives us some of the advantages of the
        // identity check too, but we are missing out on some by having it only
        // enabled for either new tests that request it or old graphql_spec
        // tests that were explicitly written with it in mind
        if spec.check_identity {
            if matches!(source, Source::GraphQL) {
                let actual = config.to_sdl();

                // \r is added automatically in windows, it's safe to replace it with \n
                let content = content.replace("\r\n", "\n");

                let path_str = spec.path.display().to_string();
                let context = format!("path: {}", path_str);

                let actual = tailcall_prettier::format(actual, &tailcall_prettier::Parser::Gql)
                    .await
                    .context(context.clone())
                    .unwrap();

                let expected = tailcall_prettier::format(content, &tailcall_prettier::Parser::Gql)
                    .await
                    .context(context)
                    .unwrap();

                pretty_assertions::assert_eq!(
                    actual,
                    expected,
                    "Identity check failed for {:#?}",
                    spec.path,
                );
            } else {
                panic!(
                    "Spec {:#?} has \"check identity\" enabled, but its config isn't in GraphQL.",
                    spec.path
                );
            }
        }

        server.push(config);
    }
    server
}

async fn run_query_tests_on_spec(
    spec: ExecutionSpec,
    server: Vec<ConfigModule>,
    mock_http_client: Arc<Http>,
) {
    if let Some(tests) = spec.test.as_ref() {
        let app_ctx = spec
            .app_context(
                server.first().unwrap(),
                spec.env.clone().unwrap_or_default(),
                mock_http_client.clone(),
            )
            .await;

        // test: Run test specs

        for (i, test) in tests.iter().enumerate() {
            let response = run_test(app_ctx.clone(), test)
                .await
                .context(spec.path.to_str().unwrap().to_string())
                .unwrap();

            let mut headers: BTreeMap<String, String> = BTreeMap::new();

            for (key, value) in response.headers() {
                headers.insert(key.to_string(), value.to_str().unwrap().to_string());
            }

            let response: APIResponse = APIResponse {
                status: response.status().clone().as_u16(),
                headers,
                body: Some(APIBody::Value(
                    serde_json::from_slice(
                        &hyper::body::to_bytes(response.into_body()).await.unwrap(),
                    )
                    .unwrap_or_default(),
                )),
            };

            let snapshot_name = format!("{}_{}", spec.safe_name, i);

            insta::assert_json_snapshot!(snapshot_name, response);
        }

        mock_http_client.test_hits(&spec.path);
    }
}

async fn test_spec(spec: ExecutionSpec) {
    // insta settings
    let mut insta_settings = insta::Settings::clone_current();
    insta_settings.set_prepend_module_to_snapshot(false);
    let _guard = insta::Settings::bind_to_scope(&insta_settings);

    let mock_http_client = Arc::new(Http::new(&spec));

    // check sdl error if any
    if is_sdl_error(spec.clone(), mock_http_client.clone()).await {
        return;
    }

    // Parse and validate all server configs + check for identity
    let server = check_server_config(spec.clone()).await;

    // Resolve all configs
    let mut runtime = runtime::create_runtime(mock_http_client.clone(), spec.env.clone(), None);
    runtime.file = Arc::new(File::new(spec.clone()));
    let reader = ConfigReader::init(runtime);

    let server: Vec<ConfigModule> = join_all(
        server
            .into_iter()
            .map(|config| reader.resolve(config, spec.path.parent())),
    )
    .await
    .into_iter()
    .enumerate()
    .map(|(i, result)| {
        result.unwrap_or_else(|e| {
            panic!(
                "Couldn't resolve GraphQL in server definition #{} of {:#?}: {}",
                i + 1,
                spec.path,
                e
            )
        })
    })
    .collect();

    // merged: Run merged specs
    let merged = server
        .iter()
        .fold(ConfigModule::default(), |acc, c| acc.merge_right(c.clone()))
        // Apply required transformers to the configuration
        .transform(Required)
        .to_result()
        .unwrap()
        .to_sdl();

    let formatter = tailcall_prettier::format(merged, &Parser::Gql)
        .await
        .unwrap();

    let snapshot_name = format!("{}_merged", spec.safe_name);

    insta::assert_snapshot!(snapshot_name, formatter);

    // client: Check if client spec matches snapshot
    if server.len() == 1 {
        let config = &server[0];

        let client = print_schema(
            (Blueprint::try_from(config)
                .context(format!("file: {}", spec.path.to_str().unwrap()))
                .unwrap())
            .to_schema(),
        );

        let formatted = tailcall_prettier::format(client, &Parser::Gql)
            .await
            .unwrap();
        let snapshot_name = format!("{}_client", spec.safe_name);

        insta::assert_snapshot!(snapshot_name, formatted);
    }

    // run query tests
    run_query_tests_on_spec(spec, server, mock_http_client).await;
}

pub async fn load_and_test_execution_spec(path: &Path) -> anyhow::Result<()> {
    let contents = fs::read_to_string(path)?;
    let spec: ExecutionSpec = ExecutionSpec::from_source(path, contents)
        .await
        .with_context(|| path.display().to_string())?;

    match spec.runner {
        Some(Annotation::Skip) => {
            println!("{} ... {}", spec.path.display(), "skipped".blue());
        }
        None => test_spec(spec).await,
    }

    Ok(())
}

async fn run_test(
    app_ctx: Arc<AppContext>,
    request: &APIRequest,
) -> anyhow::Result<hyper::Response<Body>> {
    let body = request
        .body
        .as_ref()
        .map(|body| Body::from(body.to_bytes()))
        .unwrap_or_default();

    let method = request.method.clone();
    let headers = request.headers.clone();
    let url = request.url.clone();
    let req = headers
        .into_iter()
        .fold(
            Request::builder()
                .method(method.to_hyper())
                .uri(url.as_str()),
            |acc, (key, value)| acc.header(key, value),
        )
        .body(body)?;

    // TODO: reuse logic from server.rs to select the correct handler
    if app_ctx.blueprint.server.enable_batch_requests {
        handle_request::<GraphQLBatchRequest>(req, app_ctx).await
    } else {
        handle_request::<GraphQLRequest>(req, app_ctx).await
    }
}
