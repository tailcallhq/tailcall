use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};

use async_graphql_value::ConstValue;

use super::{FoldRateLimitResults, NumRequestsRemaining, RateLimitError, RateLimiter};
use crate::blueprint::{hash_const_value, RateLimit};
use crate::http::NumRequestsFetched;
use crate::json::JsonLike;

#[derive(Clone)]
pub struct LocalRateLimiter {
  type_rate_limits: Arc<HashMap<String, RateLimit>>,
  field_rate_limits: Arc<HashMap<String, HashMap<String, RateLimit>>>,
  num_requests_fetched: Arc<Mutex<HashMap<String, HashMap<String, NumRequestsFetched>>>>,
}

impl LocalRateLimiter {
  pub fn new(
    type_rate_limits: HashMap<String, RateLimit>,
    field_rate_limits: HashMap<String, HashMap<String, RateLimit>>,
  ) -> Self {
    Self {
      type_rate_limits: Arc::new(type_rate_limits),
      field_rate_limits: Arc::new(field_rate_limits),
      num_requests_fetched: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn allow_field(&self, field: &str, sub_field: &str) -> Result<NumRequestsRemaining, RateLimitError> {
    let lowercased_field = field.to_lowercase();
    let lowercased_sub_field = sub_field.to_lowercase();
    if let Some(rate_limit) = self
      .field_rate_limits
      .get(&lowercased_field)
      .and_then(|map| map.get(&lowercased_sub_field))
    {
      self.allow([lowercased_field, lowercased_sub_field], rate_limit)
    } else {
      Ok(NumRequestsRemaining::Unlimited)
    }
  }

  pub fn allow_obj(&self, type_name: &str, const_value: &ConstValue) -> Result<NumRequestsRemaining, RateLimitError> {
    let type_name = type_name.to_lowercase();
    if let Some(rate_limit @ RateLimit { group_by, .. }) = self.type_rate_limits.get(&type_name) {
      let group_by = group_by.as_ref().map(String::as_str).unwrap_or("");
      let mut hasher = DefaultHasher::new();
      if let Some(val) = const_value.get_key(group_by) {
        hash_const_value(val, &mut hasher);
      }
      let hash = hasher.finish();
      self.allow([type_name.to_lowercase(), hash.to_string()], rate_limit)
    } else {
      self.allow_obj_fields(&type_name, const_value)
    }
  }

  pub fn allow_obj_fields(
    &self,
    type_name: &str,
    const_value: &ConstValue,
  ) -> Result<NumRequestsRemaining, RateLimitError> {
    match const_value {
      ConstValue::Object(obj) => obj
        .keys()
        .map(|key| self.allow_field(type_name, key))
        .fold_rate_limit_results(),
      _ => Ok(NumRequestsRemaining::Unlimited),
    }
  }
}

impl RateLimiter<2> for LocalRateLimiter {
  fn with_nrf<F: Fn(&mut NumRequestsFetched) -> Result<NumRequestsRemaining, RateLimitError>>(
    &self,
    [key1, key2]: [String; Self::NUM_KEYS],
    f: F,
  ) -> Result<NumRequestsRemaining, RateLimitError> {
    let mut mtx_guard = self.num_requests_fetched.lock().unwrap();
    let nrf = mtx_guard.entry(key1).or_default().entry(key2).or_default();

    f(nrf)
  }
}
