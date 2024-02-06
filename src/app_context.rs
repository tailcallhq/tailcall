use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
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
use crate::rate_limiter::rate_limiter::RateLimiter;
use crate::target_runtime::TargetRuntime;

pub struct AppContext {
    pub schema: dynamic::Schema,
    pub runtime: TargetRuntime,
    pub blueprint: Blueprint,
    pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
    pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
    pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
    pub rate_limiter: RateLimiter,
}

impl AppContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(mut blueprint: Blueprint, runtime: TargetRuntime) -> Self {
        let http_data_loaders = Rc::new(RefCell::new(vec![]));
        let gql_data_loaders = Rc::new(RefCell::new(vec![]));
        let grpc_data_loaders = Rc::new(RefCell::new(vec![]));
        let mut field_rate_limits = HashMap::new();
        let mut type_rate_limits = HashMap::new();

        for def in blueprint.definitions.iter_mut() {
            if let Definition::ObjectTypeDefinition(def) = def {
                let fld = def.name.to_lowercase();
                if let Some(ref rate_limit) = def.rate_limit {
                    type_rate_limits.insert(fld.clone(), rate_limit.clone());
                }

                for field in &mut def.fields {
                    if let Some(ref rate_limit) = field.rate_limit {
                        let sb_fld = field.name.to_lowercase();

                        field_rate_limits
                            .entry(fld.clone())
                            .or_insert_with(HashMap::new)
                            .entry(sb_fld)
                            .or_insert(rate_limit.clone());
                    }

                    let of_type = field.of_type.clone();
                    let runtime = runtime.clone();
                    let upstream_batch = blueprint.upstream.batch.clone();
                    field.map_expr(|expr| {
                        let http_data_loaders = http_data_loaders.clone();
                        let gql_data_loaders = gql_data_loaders.clone();
                        let grpc_data_loaders = grpc_data_loaders.clone();
                        let runtime = runtime.clone();
                        let of_type = of_type.clone();
                        let upstream_batch = upstream_batch.clone();
                        expr.modify(move |expr| {
                            let http_data_loaders = http_data_loaders.clone();
                            let gql_data_loaders = gql_data_loaders.clone();
                            let grpc_data_loaders = grpc_data_loaders.clone();
                            match expr {
                                Expression::IO(io) => match io {
                                    IO::Http { req_template, group_by, .. } => {
                                        let data_loader = HttpDataLoader::new(
                                            runtime.http.clone(),
                                            group_by.clone(),
                                            matches!(of_type, ListType { .. }),
                                        )
                                        .to_data_loader(upstream_batch.clone().unwrap_or_default());

                                        let result = Some(Expression::IO(IO::Http {
                                            req_template: req_template.clone(),
                                            group_by: group_by.clone(),
                                            dl_id: Some(DataLoaderId(
                                                http_data_loaders.borrow().len(),
                                            )),
                                        }));

                                        http_data_loaders.borrow_mut().push(data_loader);

                                        result
                                    }

                                    IO::GraphQL { req_template, field_name, batch, .. } => {
                                        let graphql_data_loader =
                                            GraphqlDataLoader::new(runtime.http.clone(), *batch)
                                                .to_data_loader(
                                                    upstream_batch.clone().unwrap_or_default(),
                                                );

                                        let result = Some(Expression::IO(IO::GraphQL {
                                            req_template: req_template.clone(),
                                            field_name: field_name.clone(),
                                            batch: *batch,
                                            dl_id: Some(DataLoaderId(
                                                gql_data_loaders.borrow().len(),
                                            )),
                                        }));

                                        gql_data_loaders.borrow_mut().push(graphql_data_loader);

                                        result
                                    }

                                    IO::Grpc { req_template, group_by, .. } => {
                                        let data_loader = GrpcDataLoader {
                                            client: runtime.http2_only.clone(),
                                            operation: req_template.operation.clone(),
                                            group_by: group_by.clone(),
                                        };
                                        let data_loader = data_loader.to_data_loader(
                                            upstream_batch.clone().unwrap_or_default(),
                                        );

                                        let result = Some(Expression::IO(IO::Grpc {
                                            req_template: req_template.clone(),
                                            group_by: group_by.clone(),
                                            dl_id: Some(DataLoaderId(
                                                grpc_data_loaders.borrow().len(),
                                            )),
                                        }));

                                        grpc_data_loaders.borrow_mut().push(data_loader);

                                        result
                                    }
                                },
                                _ => None,
                            }
                        })
                    });
                }
            }
        }

        let schema = blueprint.to_schema();
        let rate_limiter = RateLimiter::new(type_rate_limits, field_rate_limits);

        AppContext {
            schema,
            runtime,
            blueprint,
            http_data_loaders: Arc::new(http_data_loaders.take()),
            gql_data_loaders: Arc::new(gql_data_loaders.take()),
            grpc_data_loaders: Arc::new(grpc_data_loaders.take()),
            rate_limiter,
        }
    }

    pub async fn execute(&self, request: impl Into<DynamicRequest>) -> Response {
        self.schema.execute(request).await
    }
}
