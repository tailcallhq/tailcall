//! # Apollo Extensions for async_graphql
//!  <div align="center">
//!  <!-- CI -->
//!  <img src="https://github.com/Miaxos/async_graphql_apollo_studio_extension/actions/workflows/ci.yml/badge.svg" />
//!  <!-- Crates version -->
//!  <a href="https://crates.io/crates/async-graphql-extension-apollo-tracing">
//!    <img src="https://img.shields.io/crates/v/async-graphql-extension-apollo-tracing.svg?style=flat-square"
//!    alt="Crates.io version" />
//!  </a>
//!  <!-- Downloads -->
//!  <a href="https://crates.io/crates/async-graphql-extension-apollo-tracing">
//!    <img src="https://img.shields.io/crates/d/async-graphql-extension-apollo-tracing.svg?style=flat-square"
//!      alt="Download" />
//!  </a>
//! </div>
//!
//! ## Features
//!
//! * Fully support traces & errors
//! * Batched traces transfer
//! * Client segmentation
//! * Tracing
//! * Schema register protocol implemented
//!
//! ## Crate Features
//!
//! * `compression` - To enable GZIP Compression when sending traces to Apollo Studio.
mod compression;
mod proto;
pub mod register;
mod report_aggregator;

mod runtime;
mod packages;

use futures::SinkExt;
use protobuf::{well_known_types::timestamp::Timestamp, EnumOrUnknown, MessageField};
use report_aggregator::ReportAggregator;
use runtime::spawn;
use packages::serde_json;

#[macro_use]
extern crate tracing;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use async_graphql::QueryPathSegment;
use chrono::{DateTime, Utc};
use futures::lock::Mutex;
use std::convert::TryFrom;

use async_graphql::extensions::{
    Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextResolve,
    ResolveInfo,
};
use async_graphql::parser::types::{ExecutableDocument, OperationType, Selection};
use async_graphql::{Response, ServerResult, Value, Variables};
use proto::reports::{
    trace::{self, node, Node},
    Trace,
};
use std::convert::TryInto;

pub use proto::reports::trace::http::Method;

/// Apollo Tracing Extension to send traces to Apollo Studio
/// The extension to include to your `async_graphql` instance to connect with Apollo Studio.
///
/// <https://www.apollographql.com/docs/studio/setup-analytics/#adding-support-to-a-third-party-server-advanced>
///
/// Apollo Tracing works by creating traces from GraphQL calls, which contains extra data about the
/// request being processed. These traces are then batched sent to Apollo Studio.
///
/// The extension will start a separate function on a separate thread which will aggregate traces
/// and batch send them.
///
/// To add additional data to your metrics, you should add a ApolloTracingDataExt to your
/// query_data when you process a query with async_graphql.
pub struct ApolloTracing {
    report: Arc<ReportAggregator>,
}

/// The structure where you can add additional context for Apollo Studio.
/// This structure must be added to your query data.
///
/// It'll allow you to [segment your
/// users](https://www.apollographql.com/docs/studio/client-awareness/)
///
/// * `client_name` - You can segment your users by the client they are using to access your
/// GraphQL API, it's really usefull when you have mobile and web users for instance. Usually we
/// add a header `apollographql-client-name` to store this data. Apollo Studio will allow you to
/// aggregate your metrics by Client Name.
/// * `client_version` - You can segment your users by the client but it's usefull to also have the
/// version your clients are using, especially when you are serving your API for mobile users,
/// it'll allow you to follow metrics depending on which version your users are. Usually we add a
/// header `apollographql-client-version` to store this data.
/// * `method` - The HTTP Method.
/// * `status_code` - The status code return by your GraphQL API. It's a little weird to have to put it
/// before executing the graphql function, it'll be changed later but usually it's just a 200.
#[derive(Debug, Clone, Default, derive_builder::Builder)]
#[builder(pattern = "owned", setter(into, strip_option))]
pub struct ApolloTracingDataExt {
    #[builder(default)]
    pub client_name: Option<String>,
    #[builder(default)]
    pub client_version: Option<String>,
    #[builder(default)]
    pub method: Option<Method>,
    #[builder(default)]
    pub status_code: Option<u32>,
}

impl ApolloTracing {
    /// We initialize the ApolloTracing Extension by starting our aggregator async function which
    /// will receive every traces and send them to the Apollo Studio Ingress for processing
    ///
    /// * autorization_token - Token to send metrics to apollo studio.
    /// * hostname - Hostname like yourdomain-graphql-1.io
    /// * graph_ref - `ref@variant`  Graph reference with variant
    /// * release_name - Your release version or release name from Git for example
    pub fn new(
        authorization_token: String,
        hostname: String,
        graph_id: String,
        variant: String,
        service_version: String,
    ) -> ApolloTracing {
        let report = ReportAggregator::initialize(
            authorization_token,
            hostname,
            graph_id,
            variant,
            service_version,
        );

        ApolloTracing {
            report: Arc::new(report),
        }
    }
}

impl ExtensionFactory for ApolloTracing {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(ApolloTracingExtension {
            inner: Mutex::new(Inner {
                start_time: Utc::now(),
                end_time: Utc::now(),
            }),
            report: self.report.clone(),
            nodes: RwLock::new(HashMap::new()),
            root_node: Arc::new(RwLock::new(Node::default())),
            operation_name: RwLock::new("schema".to_string()),
        })
    }
}

struct Inner {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

struct ApolloTracingExtension {
    inner: Mutex<Inner>,
    report: Arc<ReportAggregator>,
    nodes: RwLock<HashMap<String, Arc<RwLock<Node>>>>,
    root_node: Arc<RwLock<Node>>,
    operation_name: RwLock<String>,
}

#[async_trait::async_trait]
impl Extension for ApolloTracingExtension {
    #[instrument(level = "debug", skip(self, ctx, next))]
    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let document = next.run(ctx, query, variables).await?;
        let is_schema = document
            .operations
            .iter()
            .filter(|(_, operation)| operation.node.ty == OperationType::Query)
            .any(|(_, operation)| operation.node.selection_set.node.items.iter().any(|selection| matches!(&selection.node, Selection::Field(field) if field.node.name.node == "__schema")));
        if !is_schema {
            let result: String =
                ctx.stringify_execute_doc(&document, &Variables::from_json(serde_json::from_str("{}").unwrap()));
            let name = document
                .operations
                .iter()
                .next()
                .and_then(|x| x.0)
                .map(|x| x.as_str())
                .unwrap_or("no_name");
            let query_type = format!("# {name}\n {query}", name = name, query = result);
            *self.operation_name.write().unwrap() = query_type;
        }
        Ok(document)
    }

    #[instrument(level = "debug", skip(self, ctx, next))]
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let start_time = Utc::now();
        self.inner.lock().await.start_time = start_time;

        let resp = next.run(ctx, operation_name).await;
        // Here every responses are executed
        // The next execute should aggregates a node a not a trace
        let mut inner = self.inner.lock().await;
        inner.end_time = Utc::now();

        let tracing_extension = ctx
            .data::<ApolloTracingDataExt>()
            .ok()
            .cloned()
            .unwrap_or_default();

        let client_name = tracing_extension
            .client_name
            .unwrap_or_else(|| "no client name".to_string());
        let client_version = tracing_extension
            .client_version
            .unwrap_or_else(|| "no client version".to_string());
        let method = tracing_extension
            .method
            .or(<Method as protobuf::Enum>::from_str("UNKNOWN"));
        let status_code = tracing_extension.status_code.unwrap_or(0);

        let mut trace: Trace = Trace {
            client_name,
            client_version,
            duration_ns: (inner.end_time - inner.start_time)
                .num_nanoseconds()
                .map(|x| x.try_into().unwrap())
                .unwrap_or(0),
            ..Default::default()
        };

        trace.details = Some(trace::Details {
            operation_name: operation_name
                .map(|x| x.to_string())
                .unwrap_or_else(|| "no operation".to_string()),
            ..Default::default()
        })
        .into();

        trace.http = Some(trace::HTTP {
            method: EnumOrUnknown::new(method.unwrap()),
            status_code,
            ..Default::default()
        })
        .into();

        trace.end_time = MessageField::some(Timestamp {
            nanos: inner.end_time.timestamp_subsec_nanos().try_into().unwrap(),
            seconds: inner.end_time.timestamp(),
            special_fields: Default::default(),
        });

        trace.start_time =
            protobuf::MessageField::some(protobuf::well_known_types::timestamp::Timestamp {
                nanos: inner
                    .start_time
                    .timestamp_subsec_nanos()
                    .try_into()
                    .unwrap(),
                seconds: inner.start_time.timestamp(),
                special_fields: Default::default(),
            });

        let root_node = self.root_node.read().unwrap();
        trace.root = Some(root_node.clone()).into();

        let mut sender = self.report.sender();

        let operation_name = self.operation_name.read().unwrap().clone();

        let _handle = spawn(async move {
            if let Err(e) = sender.send((operation_name, trace)).await {
                error!(error = ?e);
            }
        });
        resp
    }

    #[instrument(level = "debug", skip(self, ctx, info, next))]
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        // We do create a node when it's invoked which we insert at the right place inside the
        // struct.

        let path = info.path_node.to_string_vec().join(".");
        let field_name = info.path_node.field_name().to_string();
        let parent_type = info.parent_type.to_string();
        let _return_type = info.return_type.to_string();
        let start_time = Utc::now() - self.inner.lock().await.start_time;
        let path_node = info.path_node;

        let node: Node = Node {
            end_time: 0,
            id: match path_node.segment {
                QueryPathSegment::Name(name) => Some(node::Id::ResponseName(name.to_string())),
                QueryPathSegment::Index(index) => {
                    Some(node::Id::Index(index.try_into().unwrap_or(0)))
                }
            },
            start_time: match start_time
                .num_nanoseconds()
                .and_then(|x| u64::try_from(x).ok())
            {
                Some(duration) => duration,
                None => Utc::now()
                    .timestamp_nanos_opt()
                    .unwrap_or_default()
                    .try_into()
                    .unwrap_or_default(),
            },
            parent_type: parent_type.to_string(),
            original_field_name: field_name,
            ..Default::default()
        };

        let node = Arc::new(RwLock::new(node));
        self.nodes.write().unwrap().insert(path, node.clone());
        let parent_node = path_node.parent.map(|x| x.to_string_vec().join("."));
        // Use the path to create a new node
        // https://github.com/apollographql/apollo-server/blob/291c17e255122d4733b23177500188d68fac55ce/packages/apollo-server-core/src/plugin/traceTreeBuilder.ts
        let res = match next.run(ctx, info).await {
            Ok(res) => Ok(res),
            Err(e) => {
                let json = match serde_json::to_string(&e) {
                    Ok(content) => content,
                    Err(e) => format!("{{ \"error\": \"{e:?}\" }}"),
                };
                let error = trace::Error {
                    message: e.message.clone(),
                    location: e
                        .locations
                        .clone()
                        .into_iter()
                        .map(|x| trace::Location {
                            line: x.line as u32,
                            column: x.column as u32,
                            special_fields: protobuf::SpecialFields::default(),
                        })
                        .collect(),
                    json,
                    ..Default::default()
                };

                node.write().unwrap().error = vec![error];
                Err(e)
            }
        };
        let end_time = Utc::now() - self.inner.lock().await.start_time;

        node.write().unwrap().end_time = match end_time
            .num_nanoseconds()
            .and_then(|x| u64::try_from(x).ok())
        {
            Some(duration) => duration,
            None => Utc::now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
                .try_into()
                .unwrap_or_default(),
        };

        match parent_node {
            None => {
                let mut root_node = self.root_node.write().unwrap();
                let child = &mut root_node.child;
                let node = node.read().unwrap();
                // Can't copy or pass a ref to Protobuf
                // So we clone
                child.push(node.clone());
            }
            Some(parent) => {
                let nodes = self.nodes.read().unwrap();
                let node_read = nodes.get(&parent).unwrap();
                let mut parent = node_read.write().unwrap();
                let child = &mut parent.child;
                let node = node.read().unwrap();
                // Can't copy or pass a ref to Protobuf
                // So we clone
                child.push(node.clone());
            }
        };

        res
    }
}
