use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use async_graphql_value::ConstValue;
use serde::Serialize;

use crate::blueprint::RateLimit;
use crate::http::NumRequestsFetched;

#[derive(Clone)]
pub struct RateLimiter {
  rate_limit_configs: Arc<HashMap<String, HashMap<String, RateLimit>>>,
  num_requests_fetched: Arc<Mutex<HashMap<String, HashMap<String, NumRequestsFetched>>>>,
}

pub enum NumRequestsRemaining {
  Unlimited,
  Limited(usize),
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitError {
  RateLimitExceeded,
  InternalServerError,
}

impl RateLimitError {
  fn to_value(&self) -> serde_json::Value {
    serde_json::to_value(self.clone()).unwrap()
  }
}

impl Debug for RateLimitError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.to_value())
  }
}

impl Display for RateLimitError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_value())
  }
}

impl std::error::Error for RateLimitError {}

impl RateLimiter {
  pub fn new(rate_limit_configs: HashMap<String, HashMap<String, RateLimit>>) -> Self {
    Self {
      rate_limit_configs: Arc::new(rate_limit_configs),
      num_requests_fetched: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn allow(&self, field: String, sub_field: String) -> Result<NumRequestsRemaining, RateLimitError> {
    if let Some(rate_limit) = self.rate_limit_configs.get(&field).and_then(|map| map.get(&sub_field)) {
      let mut mtx_guard = self.num_requests_fetched.lock().unwrap();
      let nrf = mtx_guard
        .entry(field)
        .or_default()
        .entry(sub_field)
        .or_insert(NumRequestsFetched { last_fetched: SystemTime::now(), num_requests: 0 });

      let duration = nrf.last_fetched.elapsed().unwrap();
      let requests_remaining = rate_limit.requests.get() as usize - nrf.num_requests;
      if duration < rate_limit.duration && requests_remaining > 0 {
        nrf.num_requests += 1;
        nrf.last_fetched = SystemTime::now();
      } else if duration >= rate_limit.duration {
        nrf.last_fetched = SystemTime::now();
        nrf.num_requests = 1;
      } else {
        Err(RateLimitError::RateLimitExceeded)?
      }

      Ok(NumRequestsRemaining::Limited(requests_remaining))
    } else {
      Ok(NumRequestsRemaining::Unlimited)
    }
  }

  pub fn allow_obj(&self, type_name: String, const_value: &ConstValue) -> Result<NumRequestsRemaining, RateLimitError> {
    match const_value {
      ConstValue::Object(obj) => obj
        .keys()
        .map(|key| self.allow(type_name.clone(), key.to_string()))
        .try_fold(NumRequestsRemaining::Unlimited, |acc, rate_limit_result| {
          use NumRequestsRemaining::*;
          match (acc, rate_limit_result) {
            (Unlimited, Ok(Unlimited)) => Ok(Unlimited),
            (Limited(x), Ok(Limited(y))) => Ok(Limited(std::cmp::min(x, y))),
            (Unlimited, res @ Ok(Limited(_))) => res,
            (res @ Limited(_), Ok(Unlimited)) => Ok(res),
            (_, err @ Err(_)) => err,
          }
        }),
      _ => Ok(NumRequestsRemaining::Unlimited),
    }
  }
}
