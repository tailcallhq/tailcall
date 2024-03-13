
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use std::time::SystemTime;


use serde::Serialize;


use crate::rate_limiter::rate_limit::RateLimit;
use crate::rate_limiter::rate_limit_key::RateLimitKey;
use crate::rate_limiter::Key;

#[derive(Clone)]
pub struct RateLimiter {
    num_requests_fetched: Arc<Mutex<HashMap<Key, NumRequestsFetched>>>,
}

struct NumRequestsFetched {
    last_fetched: SystemTime,
    num_requests: usize,
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

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self { num_requests_fetched: Arc::new(Mutex::new(HashMap::new())) }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn allow<Ctx>(
        &self,
        rate_limit_key: &impl RateLimitKey<Ctx>,
        rate_limit: &impl RateLimit,
        ctx: Ctx,
    ) -> Result<NumRequestsRemaining, RateLimitError> {
        let mut mtx_guard = self.num_requests_fetched.lock().unwrap();
        let nrf = mtx_guard
            .entry(rate_limit_key.key(ctx))
            .or_insert(NumRequestsFetched { last_fetched: SystemTime::now(), num_requests: 0 });

        let duration = nrf.last_fetched.elapsed().unwrap();
        let requests_remaining = rate_limit.requests().get() as usize - nrf.num_requests;
        if duration < rate_limit.duration() && requests_remaining > 0 {
            nrf.num_requests += 1;
            nrf.last_fetched = SystemTime::now();
        } else if duration >= rate_limit.duration() {
            nrf.last_fetched = SystemTime::now();
            nrf.num_requests = 1;
        } else {
            Err(RateLimitError::RateLimitExceeded)?
        }

        Ok(NumRequestsRemaining::Limited(requests_remaining))
    }
}

pub trait FoldRateLimitResults
where
    Self: IntoIterator<Item = Result<NumRequestsRemaining, RateLimitError>> + Sized,
{
    fn fold_rate_limit_results(self) -> Result<NumRequestsRemaining, RateLimitError> {
        self.into_iter()
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

impl<T> FoldRateLimitResults for T where
    T: IntoIterator<Item = Result<NumRequestsRemaining, RateLimitError>> + Sized
{
}
