use std::{
    convert::Infallible,
    task::{Context, Poll},
    time::Duration,
};

use async_graphql::{
    http::{create_multipart_mixed_stream, is_accept_multipart_mixed},
    Executor,
};
use async_graphql_extension_apollo_tracing::{ApolloTracingDataExtBuilder, Method};

use axum::{
    body::{Body, HttpBody},
    extract::FromRequest,
    http::{Request as HttpRequest, Response as HttpResponse},
    response::IntoResponse,
    BoxError,
};
use bytes::Bytes;
use futures_util::{future::BoxFuture, StreamExt};
use tower_service::Service;

use async_graphql_axum::{GraphQLBatchRequest, GraphQLRequest, GraphQLResponse};

/// A GraphQL service.
#[derive(Clone)]
pub struct GraphQL<E> {
    executor: E,
}

impl<E> GraphQL<E> {
    /// Create a GraphQL handler.
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

impl<B, E> Service<HttpRequest<B>> for GraphQL<E>
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Data: Into<Bytes>,
    B::Error: Into<BoxError>,
    E: Executor,
{
    type Response = HttpResponse<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HttpRequest<B>) -> Self::Future {
        let executor = self.executor.clone();
        let req = req.map(Body::new);
        Box::pin(async move {
            let is_accept_multipart_mixed = req
                .headers()
                .get("accept")
                .and_then(|value| value.to_str().ok())
                .map(is_accept_multipart_mixed)
                .unwrap_or_default();

            if is_accept_multipart_mixed {
                let req =
                    match GraphQLRequest::<rejection::GraphQLRejection>::from_request(req, &())
                        .await
                    {
                        Ok(req) => req,
                        Err(err) => return Ok(err.into_response()),
                    };

                // ------------------------------------------------------------
                // Fetch & add details about the client here.
                // ------------------------------------------------------------
                let req = req.0.data(
                    ApolloTracingDataExtBuilder::default()
                        .client_name("Sample_Client")
                        .client_version("v4")
                        .method(Method::Post)
                        .status_code(200u32)
                        .build()
                        .unwrap(),
                );
                // ------------------------------------------------------------

                let stream = executor.execute_stream(req, None);
                let body = Body::from_stream(
                    create_multipart_mixed_stream(
                        stream,
                        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
                            Duration::from_secs(30),
                        ))
                        .map(|_| ()),
                    )
                    .map(Ok::<_, std::io::Error>),
                );
                Ok(HttpResponse::builder()
                    .header("content-type", "multipart/mixed; boundary=graphql")
                    .body(body)
                    .expect("BUG: invalid response"))
            } else {
                let req = match GraphQLBatchRequest::<rejection::GraphQLRejection>::from_request(
                    req,
                    &(),
                )
                .await
                {
                    Ok(req) => req,
                    Err(err) => return Ok(err.into_response()),
                };
                // ------------------------------------------------------------
                // Fetch & add details about the client here.
                // ------------------------------------------------------------
                let req = req.0.data(
                    ApolloTracingDataExtBuilder::default()
                        .client_name("Sample_Client")
                        .client_version("v2")
                        .build()
                        .unwrap(),
                );
                // ------------------------------------------------------------
                Ok(GraphQLResponse(executor.execute_batch(req).await).into_response())
            }
        })
    }
}

pub mod rejection {
    use async_graphql::ParseRequestError;
    use axum::{
        body::Body,
        http,
        http::StatusCode,
        response::{IntoResponse, Response},
    };

    /// Rejection used for [`GraphQLRequest`](GraphQLRequest).
    pub struct GraphQLRejection(pub ParseRequestError);

    impl IntoResponse for GraphQLRejection {
        fn into_response(self) -> Response {
            match self.0 {
                ParseRequestError::PayloadTooLarge => http::Response::builder()
                    .status(StatusCode::PAYLOAD_TOO_LARGE)
                    .body(Body::empty())
                    .unwrap(),
                bad_request => http::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(format!("{:?}", bad_request)))
                    .unwrap(),
            }
        }
    }

    impl From<ParseRequestError> for GraphQLRejection {
        fn from(err: ParseRequestError) -> Self {
            GraphQLRejection(err)
        }
    }
}
