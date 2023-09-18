mod client;
mod data_loader;
mod default_request_like;
mod memo_client;
mod method;
mod request_context;
mod response;
mod scheme;
mod server;
mod server_context;

pub use client::*;
pub use data_loader::*;
use default_request_like::DefaultRequestLike;
use http_cache_semantics::CachePolicy;
pub use method::Method;
pub use request_context::RequestContext;
pub use response::*;
pub use scheme::Scheme;
pub use server::start_server;
pub use server_context::ServerContext;

lazy_static::lazy_static! {
    static ref DEFAULT_REQUEST_LIKE: DefaultRequestLike = DefaultRequestLike::default();
}

// TODO: add unit-tests
/// Returns the minimum TTL of the given responses.
pub fn min_ttl<'a>(res_vec: impl Iterator<Item = &'a Response>) -> i32 {
  let mut min = -1;
  for res in res_vec {
    let policy = CachePolicy::new(DEFAULT_REQUEST_LIKE.upcast(), res);
    let ttl = policy.max_age().as_secs() as i32;
    if min == -1 || ttl < min {
      min = ttl;
    }
  }
  min
}
