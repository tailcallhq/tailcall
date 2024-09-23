use std::sync::Arc;

use async_graphql::dynamic::{self, DynamicRequest};
use async_graphql_value::ConstValue;
use hyper::body::Bytes;

use super::ir::model::HttpClientId;
use super::HttpIO;
use crate::cli::runtime::NativeHttp;
use crate::core::async_graphql_hyper::OperationId;
use crate::core::auth::context::GlobalAuthContext;
use crate::core::blueprint::{Blueprint, Definition, SchemaModifiers};
use crate::core::data_loader::{DataLoader, DedupeResult};
use crate::core::graphql::GraphqlDataLoader;
use crate::core::grpc;
use crate::core::grpc::data_loader::GrpcDataLoader;
use crate::core::http::{DataLoaderRequest, HttpDataLoader, Response};
use crate::core::ir::model::{DataLoaderId, IoId, IO, IR};
use crate::core::ir::Error;
use crate::core::rest::{Checked, EndpointSet};
use crate::core::runtime::TargetRuntime;

pub struct AppContext {
    pub schema: dynamic::Schema,
    pub runtime: TargetRuntime,
    pub blueprint: Blueprint,
    pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
    pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
    pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
    pub http_clients: Arc<Vec<Arc<dyn HttpIO>>>,
    pub endpoints: EndpointSet<Checked>,
    pub auth_ctx: Arc<GlobalAuthContext>,
    pub dedupe_handler: Arc<DedupeResult<IoId, ConstValue, Error>>,
    pub dedupe_operation_handler: DedupeResult<OperationId, Response<Bytes>, Error>,
}

impl AppContext {
    pub fn new(
        mut blueprint: Blueprint,
        runtime: TargetRuntime,
        endpoints: EndpointSet<Checked>,
    ) -> Self {
        let mut http_data_loaders = vec![];
        let mut gql_data_loaders = vec![];
        let mut grpc_data_loaders = vec![];
        let mut http_clients: Vec<Arc<dyn HttpIO>> = vec![];

        for def in blueprint.definitions.iter_mut() {
            if let Definition::Object(def) = def {
                for field in &mut def.fields {
                    let of_type = field.of_type.clone();
                    let upstream_batch = &blueprint.upstream.batch;
                    field.map_expr(|expr| {
                        expr.modify(|expr| match expr {
                            IR::IO(io) => match io {
                                IO::Http { req_template, group_by, http_filter, proxy, .. } => {
                                    let http_client: Arc<dyn HttpIO> = if let Some(proxy) = proxy {
                                        Arc::new(NativeHttp::init(
                                            &blueprint.upstream,
                                            &blueprint.telemetry,
                                            &Some(proxy.clone()),
                                        ))
                                    } else {
                                        runtime.http.clone()
                                    };

                                    let data_loader = HttpDataLoader::new(
                                        group_by.clone(),
                                        of_type.is_list(),
                                        http_client.clone(),
                                    )
                                    .to_data_loader(upstream_batch.clone().unwrap_or_default());

                                    let result = Some(IR::IO(IO::Http {
                                        req_template: req_template.clone(),
                                        group_by: group_by.clone(),
                                        http_filter: http_filter.clone(),
                                        dl_id: Some(DataLoaderId::new(http_data_loaders.len())),
                                        http_client_id: Some(HttpClientId::new(http_clients.len())),
                                        proxy: proxy.clone(),
                                    }));

                                    http_data_loaders.push(data_loader);
                                    http_clients.push(http_client);

                                    result
                                }

                                IO::GraphQL { req_template, field_name, batch, .. } => {
                                    let http_client = runtime.http.clone();

                                    let graphql_data_loader =
                                        GraphqlDataLoader::new(*batch, http_client.clone())
                                            .into_data_loader(
                                                upstream_batch.clone().unwrap_or_default(),
                                            );

                                    let result = Some(IR::IO(IO::GraphQL {
                                        req_template: req_template.clone(),
                                        field_name: field_name.clone(),
                                        batch: *batch,
                                        dl_id: Some(DataLoaderId::new(gql_data_loaders.len())),
                                        http_client_id: Some(HttpClientId::new(http_clients.len())),
                                    }));

                                    gql_data_loaders.push(graphql_data_loader);
                                    http_clients.push(http_client);

                                    result
                                }

                                IO::Grpc { req_template, group_by, proxy, .. } => {
                                    let http_client: Arc<dyn HttpIO> = if let Some(proxy) = proxy {
                                        Arc::new(NativeHttp::init(
                                            &blueprint.upstream,
                                            &blueprint.telemetry,
                                            &Some(proxy.clone()),
                                        ))
                                    } else {
                                        runtime.http.clone()
                                    };

                                    let data_loader = GrpcDataLoader {
                                        operation: req_template.operation.clone(),
                                        group_by: group_by.clone(),
                                        http_client: http_client.clone(),
                                    };
                                    let data_loader = data_loader.into_data_loader(
                                        upstream_batch.clone().unwrap_or_default(),
                                    );

                                    let result = Some(IR::IO(IO::Grpc {
                                        req_template: req_template.clone(),
                                        group_by: group_by.clone(),
                                        dl_id: Some(DataLoaderId::new(grpc_data_loaders.len())),
                                        http_client_id: Some(HttpClientId::new(http_clients.len())),
                                        proxy: proxy.clone(),
                                    }));

                                    grpc_data_loaders.push(data_loader);
                                    http_clients.push(http_client);

                                    result
                                }
                                IO::Js { name: method } => {
                                    Some(IR::IO(IO::Js { name: method.clone() }))
                                }
                            },
                            _ => None,
                        })
                    });
                }
            }
        }

        let schema = blueprint
            .to_schema_with(SchemaModifiers::default().extensions(runtime.extensions.clone()));
        let auth = blueprint.server.auth.clone();
        let auth_ctx = GlobalAuthContext::new(auth);

        AppContext {
            schema,
            runtime,
            blueprint,
            http_data_loaders: Arc::new(http_data_loaders),
            gql_data_loaders: Arc::new(gql_data_loaders),
            grpc_data_loaders: Arc::new(grpc_data_loaders),
            http_clients: Arc::new(http_clients),
            endpoints,
            auth_ctx: Arc::new(auth_ctx),
            dedupe_handler: Arc::new(DedupeResult::new(false)),
            dedupe_operation_handler: DedupeResult::new(false),
        }
    }

    pub async fn execute(&self, request: impl Into<DynamicRequest>) -> async_graphql::Response {
        self.schema.execute(request).await
    }
}
