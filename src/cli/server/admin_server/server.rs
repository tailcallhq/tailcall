use std::net::SocketAddr;

use anyhow::Result;
use async_graphql::{EmptyMutation, EmptySubscription, Request, Schema};
use http::{Method, Response, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use super::graphql::{Config, Query};
use crate::core::async_graphql_hyper::GraphQLResponse;
use crate::core::blueprint::Blueprint;
use crate::core::config::ConfigModule;
use crate::core::Errata;

#[derive(Debug)]
pub struct AdminServer {
    addr: SocketAddr,
    sdl: String,
}

impl AdminServer {
    pub fn new(config_module: &ConfigModule) -> Result<Option<Self>> {
        if let Some(admin) = config_module.server.admin.as_ref() {
            let blueprint = Blueprint::try_from(config_module).map_err(Errata::from)?;
            let sdl = crate::core::document::print(config_module.config().into());
            let addr = (blueprint.server.hostname, admin.port.get()).into();

            Ok(Some(Self { addr, sdl }))
        } else {
            Ok(None)
        }
    }

    pub async fn start(self) -> Result<()> {
        let server = Server::try_bind(&self.addr)?;
        let config = Config { sdl: self.sdl };
        let query = Query { config };
        let schema = Schema::new(query, EmptyMutation, EmptySubscription);

        server
            .serve(make_service_fn(|_| {
                let schema = schema.clone();
                async move {
                    Result::<_>::Ok(service_fn(move |req| {
                        let (parts, body) = req.into_parts();
                        let schema = schema.clone();

                        async move {
                            match parts.method {
                                Method::POST if parts.uri.path() == "/graphql" => {
                                    let body = hyper::body::to_bytes(body).await?;
                                    let request: Request = serde_json::from_slice(&body)?;

                                    let res = schema.execute(request).await;
                                    let res = GraphQLResponse::from(res);

                                    Result::<_>::Ok(res.into_response()?)
                                }
                                _ => {
                                    let mut response = Response::default();

                                    *response.status_mut() = StatusCode::NOT_FOUND;

                                    Result::<_>::Ok(response)
                                }
                            }
                        }
                    }))
                }
            }))
            .await?;

        Ok(())
    }
}
