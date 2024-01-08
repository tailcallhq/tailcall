pub mod data_loader;
pub mod data_loader_request;
pub mod protobuf;
pub mod request;
pub mod request_template;

pub use data_loader_request::DataLoaderRequest;
pub use request::execute_grpc_request;
pub use request_template::RequestTemplate;


// TODO: request version isn't support in WASM builds
#[cfg(feature = "default")]
fn set_req_version(req: &mut reqwest::Request) {
  *req.version_mut() = reqwest::Version::HTTP_2;
}
