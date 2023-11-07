use hyper::http::uri::InvalidUri;
use hyper_tls::HttpsConnector;
use tokio_stream::StreamExt;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};
use tonic_reflection::pb::{
  server_reflection_client::ServerReflectionClient, server_reflection_request, server_reflection_response,
  ListServiceResponse, ServerReflectionRequest,
};

/// Fetch service name and method name from a given grpc server dynamically
pub async fn fetch_service_schema(
  address: &str,
  service_name: &str,
  /*tls_config: Option<(String, Identity)>*/
) -> Result<Vec<String>, tonic::Status> {
  let https = HttpsConnector::new();
  let channel = match Channel::builder(
    address
      .parse()
      .map_err(|e: InvalidUri| tonic::Status::internal(e.to_string()))?,
  )
  .connect_with_connector(https)
  .await
  {
    Ok(it) => it,
    Err(err) => return Err(tonic::Status::internal(format!("Transport error: {}", err))),
  };
  // let channel = setup_channel(address, tls_config);

  let mut client = ServerReflectionClient::new(channel);

  let request = ServerReflectionRequest {
    host: "".into(),
    message_request: Some(server_reflection_request::MessageRequest::ListServices("".into())),
    ..Default::default()
  };

  let response = client
    .server_reflection_info(tonic::Request::new(tokio_stream::once(request)))
    .await?;

  let mut response_stream = response.into_inner();

  let mut services = Vec::new();

  while let Some(result) = response_stream.next().await {
    match result {
      Ok(resp) => {
        if let Some(msg_resp) = resp.message_response {
          match msg_resp {
            server_reflection_response::MessageResponse::ListServicesResponse(ListServiceResponse {
              service: inner_services,
            }) => {
              for service in inner_services {
                services.push(service.name);
              }
            }
            _ => {}
          }
        }
      }
      Err(e) => {
        eprintln!("Error fetching service schema: {}", e);
      }
    }
  }

  // Filter the services based on the provided service_name
  services = services.into_iter().filter(|name| name == service_name).collect();

  Ok(services)
}

//TODO: move this to utils, may change function signature! - util fns
pub async fn setup_channel(address: &str, tls_config: Option<(String, Identity)>) -> Result<Channel, tonic::Status> {
  let mut endpoint = Endpoint::from_shared(address.to_string()).map_err(|e| tonic::Status::internal(e.to_string()))?;

  if let Some((cert, identity)) = tls_config {
    let cert = Certificate::from_pem(cert);
    endpoint = endpoint
      .tls_config(ClientTlsConfig::new().identity(identity).ca_certificate(cert))
      .map_err(|e| tonic::Status::internal(e.to_string()))?;
  }

  let channel = endpoint
    .connect()
    .await
    .map_err(|err| tonic::Status::internal(format!("Transport error: {}", err)))?;

  Ok(channel)
}
