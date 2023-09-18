use crate::async_graphql_hyper::GraphQLResponse;
use async_graphql::BatchResponse;

/// Returns the minimum value from a slice of `Option<u64>`.
///
/// If any of the values is `None`, the function will return `None`.
///
/// # Arguments
///
/// * `values` - A slice of `Option<u64>`.
///
pub fn min(values: &[Option<u64>]) -> Option<u64> {
  let mut min = None;
  for value in values {
    if value.is_none() {
      return None;
    }
    if min.is_none() || min.unwrap() > value.unwrap() {
      min = *value;
    }
  }
  min
}

/// Sets the `cache_control` max_age for a given `GraphQLResponse`.
///
/// The function modifies the `GraphQLResponse` to set the `cache_control` `max_age`
/// to the specified `min_cache` value.
///
/// # Arguments
///
/// * `res` - The GraphQL response whose `cache_control` is to be set.
/// * `min_cache` - The `max_age` value to be set for `cache_control`.
///
/// # Returns
///
/// * A modified `GraphQLResponse` with updated `cache_control` `max_age`.
pub fn set_cache_control(mut res: GraphQLResponse, min_cache: i32) -> GraphQLResponse {
  match res.0 {
    BatchResponse::Single(ref mut res) => {
      res.cache_control.max_age = min_cache;
    }
    BatchResponse::Batch(ref mut list) => {
      for res in list {
        res.cache_control.max_age = min_cache;
      }
    }
  };
  res
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_min_some() {
    let values = vec![Some(3), Some(1), Some(4)];
    assert_eq!(min(&values), Some(1));
  }

  #[test]
  fn test_min_none() {
    let values = vec![Some(3), None, Some(4)];
    assert_eq!(min(&values), None);
  }

  #[test]
  fn test_set_cache_control_single_response() {
    let response = GraphQLResponse(BatchResponse::Single(async_graphql::Response::default()));

    let updated_response = set_cache_control(response, 10);

    match updated_response.0 {
      BatchResponse::Single(ref res) => {
        assert_eq!(res.cache_control.max_age, 10);
      }
      _ => panic!("Unexpected response type!"),
    }
  }

  #[test]
  fn test_set_cache_control_batch_response() {
    let response = GraphQLResponse(BatchResponse::Batch(vec![async_graphql::Response::default()]));

    let updated_response = set_cache_control(response, 20);

    match updated_response.0 {
      BatchResponse::Batch(ref list) => {
        for res in list {
          assert_eq!(res.cache_control.max_age, 20);
        }
      }
      _ => panic!("Unexpected response type!"),
    }
  }
}
