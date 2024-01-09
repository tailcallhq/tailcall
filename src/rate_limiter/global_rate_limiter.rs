use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::{NumRequestsRemaining, RateLimitError, RateLimiter};
use crate::http::NumRequestsFetched;

#[derive(Clone)]
pub struct GlobalRateLimiter {
  num_requests_fetched: Arc<Mutex<HashMap<String, NumRequestsFetched>>>,
}

impl Default for GlobalRateLimiter {
  fn default() -> Self {
    Self::new()
  }
}

impl GlobalRateLimiter {
  pub fn new() -> Self {
    Self { num_requests_fetched: Arc::new(Mutex::new(HashMap::new())) }
  }
}

impl RateLimiter<1> for GlobalRateLimiter {
  fn with_nrf<F: Fn(&mut NumRequestsFetched) -> Result<NumRequestsRemaining, RateLimitError>>(
    &self,
    [key]: [String; Self::NUM_KEYS],
    f: F,
  ) -> Result<NumRequestsRemaining, RateLimitError> {
    let mut mtx_guard = self.num_requests_fetched.lock().unwrap();
    let nrf = mtx_guard.entry(key).or_default();

    f(nrf)
  }
}
