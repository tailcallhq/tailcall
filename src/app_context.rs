use std::sync::Arc;

use async_graphql::dynamic::{self, DynamicRequest};
use async_graphql::Response;

use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::grpc;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::http::{DataLoaderRequest, HttpDataLoader};
use crate::lambda::{DataLoaderId, Expression, IO};
use crate::runtime::TargetRuntime;

pub struct AppContext {
    pub schema: dynamic::Schema,
    pub runtime: TargetRuntime,
    pub blueprint: Blueprint,
    pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
    pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
    pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
}

impl AppContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(mut blueprint: Blueprint, runtime: TargetRuntime) -> Self {
        let mut http_data_loaders = vec![];
        let mut gql_data_loaders = vec![];
        let mut grpc_data_loaders = vec![];

        for def in blueprint.definitions.iter_mut() {
            if let Definition::Object(def) = def {
                for field in &mut def.fields {
                    let of_type = field.of_type.clone();
                    let upstream_batch = &blueprint.upstream.batch;
                    field.map_expr(|expr| {
                        expr.modify(|expr| match expr {
                            Expression::IO(io) => match io {
                                IO::Http { req_template, group_by, .. } => {
                                    let data_loader = HttpDataLoader::new(
                                        runtime.clone(),
                                        group_by.clone(),
                                        matches!(of_type, ListType { .. }),
                                    )
                                    .into_data_loader(upstream_batch.clone().unwrap_or_default());

                                    let result = Some(Expression::IO(IO::Http {
                                        req_template: req_template.clone(),
                                        group_by: group_by.clone(),
                                        dl_id: Some(DataLoaderId(http_data_loaders.len())),
                                    }));

                                    http_data_loaders.push(data_loader);

                                    result
                                }

                                IO::GraphQL { req_template, field_name, batch, .. } => {
                                    let graphql_data_loader =
                                        GraphqlDataLoader::new(runtime.clone(), *batch)
                                            .into_data_loader(
                                                upstream_batch.clone().unwrap_or_default(),
                                            );

                                    let result = Some(Expression::IO(IO::GraphQL {
                                        req_template: req_template.clone(),
                                        field_name: field_name.clone(),
                                        batch: *batch,
                                        dl_id: Some(DataLoaderId(gql_data_loaders.len())),
                                    }));

                                    gql_data_loaders.push(graphql_data_loader);

                                    result
                                }

                                IO::Grpc { req_template, group_by, .. } => {
                                    let data_loader = GrpcDataLoader {
                                        runtime: runtime.clone(),
                                        operation: req_template.operation.clone(),
                                        group_by: group_by.clone(),
                                    };
                                    let data_loader = data_loader
                                        .data_loader(upstream_batch.clone().unwrap_or_default());

                                    let result = Some(Expression::IO(IO::Grpc {
                                        req_template: req_template.clone(),
                                        group_by: group_by.clone(),
                                        dl_id: Some(DataLoaderId(grpc_data_loaders.len())),
                                    }));

                                    grpc_data_loaders.push(data_loader);

                                    result
                                }
                            },
                            _ => None,
                        })
                    });
                }
            }
        }

        let schema = blueprint.to_schema();

        AppContext {
            schema,
            runtime,
            blueprint,
            http_data_loaders: Arc::new(http_data_loaders),
            gql_data_loaders: Arc::new(gql_data_loaders),
            grpc_data_loaders: Arc::new(grpc_data_loaders),
        }
    }

    pub async fn execute(&self, request: impl Into<DynamicRequest>) -> Response {
        self.schema.execute(request).await
    }
}
