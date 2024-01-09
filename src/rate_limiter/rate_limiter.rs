use std::time::SystemTime;

use serde::Serialize;

use crate::blueprint::RateLimit;
use crate::http::NumRequestsFetched;

pub trait RateLimiter<const NUM_KEYS: usize> {
  const NUM_KEYS: usize = NUM_KEYS;

  fn with_nrf<F: Fn(&mut NumRequestsFetched) -> Result<NumRequestsRemaining, RateLimitError>>(
    &self,
    keys: [String; NUM_KEYS],
    f: F,
  ) -> Result<NumRequestsRemaining, RateLimitError>;
  #[allow(clippy::too_many_arguments)]
  fn allow(&self, keys: [String; NUM_KEYS], rate_limit: &RateLimit) -> Result<NumRequestsRemaining, RateLimitError> {
    println!("{keys:?}");
    self.with_nrf(keys, |nrf| {
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
    })
  }
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

impl From<RateLimitError> for anyhow::Error {
  fn from(value: RateLimitError) -> Self {
    let message = serde_json::to_vec(&value).unwrap();
    let len = message.len();
    let message: String = message
      .into_iter()
      .skip(1)
      .take(len - 2)
      .map(|byte| byte as char)
      .collect();
    anyhow::anyhow!(message)
  }
}

impl From<RateLimitError> for async_graphql::Error {
  fn from(value: RateLimitError) -> Self {
    let message = serde_json::to_vec(&value).unwrap();
    let len = message.len();
    let message: String = message
      .into_iter()
      .skip(1)
      .take(len - 2)
      .map(|byte| byte as char)
      .collect();
    async_graphql::Error::new(message)
  }
}

pub trait FoldRateLimitResults
where
  Self: IntoIterator<Item = Result<NumRequestsRemaining, RateLimitError>> + Sized,
{
  fn fold_rate_limit_results(self) -> Result<NumRequestsRemaining, RateLimitError> {
    self
      .into_iter()
      .try_fold(NumRequestsRemaining::Unlimited, |acc, rate_limit_result| {
        use NumRequestsRemaining::*;
        match (acc, rate_limit_result) {
          (Unlimited, Ok(Unlimited)) => Ok(Unlimited),
          (Limited(x), Ok(Limited(y))) => Ok(Limited(std::cmp::min(x, y))),
          (Unlimited, res @ Ok(Limited(_))) => res,
          (res @ Limited(_), Ok(Unlimited)) => Ok(res),
          (_, err @ Err(_)) => err,
        }
      })
  }
}

impl<T> FoldRateLimitResults for T where T: IntoIterator<Item = Result<NumRequestsRemaining, RateLimitError>> + Sized {}
