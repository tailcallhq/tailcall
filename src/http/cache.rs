use std::time::Duration;

use cache_control::{Cachability, CacheControl};

use super::Response;

pub fn cache_policy(res: &Response<async_graphql::Value>) -> Option<CacheControl> {
  let header = res.headers.get(hyper::header::CACHE_CONTROL)?;
  let value = header.to_str().ok()?;

  CacheControl::from_value(value)
}

pub fn max_age(res: &Response<async_graphql::Value>) -> Option<Duration> {
  match cache_policy(res) {
    Some(value) => value.max_age,
    None => None,
  }
}

pub fn cache_visibility(res: &Response<async_graphql::Value>) -> String {
  let cachability = cache_policy(res).and_then(|value| value.cachability);

  match cachability {
    Some(Cachability::Public) => "public".to_string(),
    Some(Cachability::Private) => "private".to_string(),
    Some(Cachability::NoCache) => "no-cache".to_string(),
    _ => "".to_string(),
  }
}

/// Returns the minimum TTL of the given responses.
pub fn min_ttl<'a>(res_vec: impl Iterator<Item = &'a Response<async_graphql::Value>>) -> i32 {
  let mut min = -1;

  for res in res_vec {
    if let Some(max_age) = max_age(res) {
      let ttl = max_age.as_secs() as i32;
      if min == -1 || ttl < min {
        min = ttl;
      }
    }
  }
  min
}

#[cfg(test)]
mod tests {

  use std::time::Duration;

  use hyper::HeaderMap;

  use crate::http::Response;

  fn cache_control_header(i: i32) -> HeaderMap {
    let mut headers = reqwest::header::HeaderMap::default();
    headers.append("Cache-Control", format!("max-age={}", i).parse().unwrap());
    headers
  }

  fn cache_control_header_visibility(i: i32, visibility: &str) -> HeaderMap {
    let mut headers = reqwest::header::HeaderMap::default();
    headers.append(
      "Cache-Control",
      format!("max-age={}, {}", i, visibility).parse().unwrap(),
    );
    headers
  }

  #[test]
  fn test_max_age_none() {
    let response = Response::default();
    assert_eq!(super::max_age(&response), None);
  }

  #[test]
  fn test_max_age_some() {
    let headers = cache_control_header(3600);
    let response = Response::default().headers(headers);

    assert_eq!(super::max_age(&response), Some(Duration::from_secs(3600)));
  }

  #[test]
  fn test_min_ttl() {
    let max_ages = [3600, 1800, 7200].map(|i| Response::default().headers(cache_control_header(i)));
    let min = super::min_ttl(max_ages.iter());
    assert_eq!(min, 1800);
  }

  #[test]
  fn test_cache_visibility_public() {
    let headers = cache_control_header_visibility(3600, "public");
    let response = Response::default().headers(headers);

    assert_eq!(super::max_age(&response), Some(Duration::from_secs(3600)));
    assert_eq!(super::cache_visibility(&response), "public");
  }

  #[test]
  fn test_cache_visibility_private() {
    let headers = cache_control_header_visibility(3600, "private");
    let response = Response::default().headers(headers);

    assert_eq!(super::max_age(&response), Some(Duration::from_secs(3600)));
    assert_eq!(super::cache_visibility(&response), "private");
  }
}
