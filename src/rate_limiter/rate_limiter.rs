use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use async_graphql_value::ConstValue;
use serde::Serialize;

use crate::blueprint::RateLimit;
use crate::helpers;
use crate::http::NumRequestsFetched;
use crate::json::JsonLike;

#[derive(Clone)]
pub struct RateLimiter {
    type_rate_limits: Arc<HashMap<String, RateLimit>>,
    field_rate_limits: Arc<HashMap<String, HashMap<String, RateLimit>>>,
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

impl RateLimiter {
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

    #[allow(clippy::too_many_arguments)]
    pub fn allow(
        &self,
        key1: String,
        key2: String,
        rate_limit: &RateLimit,
    ) -> Result<NumRequestsRemaining, RateLimitError> {
        let mut mtx_guard = self.num_requests_fetched.lock().unwrap();
        let nrf = mtx_guard
            .entry(key1)
            .or_default()
            .entry(key2)
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
    }

    pub fn allow_field(
        &self,
        field: &str,
        sub_field: &str,
    ) -> Result<NumRequestsRemaining, RateLimitError> {
        let lowercased_field = field.to_lowercase();
        let lowercased_sub_field = sub_field.to_lowercase();
        if let Some(rate_limit) = self
            .field_rate_limits
            .get(&lowercased_field)
            .and_then(|map| map.get(&lowercased_sub_field))
        {
            self.allow(lowercased_field, lowercased_sub_field, rate_limit)
        } else {
            Ok(NumRequestsRemaining::Unlimited)
        }
    }

    pub fn allow_obj(
        &self,
        type_name: &str,
        const_value: &ConstValue,
    ) -> Result<NumRequestsRemaining, RateLimitError> {
        let type_name = type_name.to_lowercase();
        if let Some(rate_limit @ RateLimit { group_by, .. }) = self.type_rate_limits.get(&type_name)
        {
            let group_by = group_by.as_ref().map(String::as_str).unwrap_or("");
            let mut hasher = DefaultHasher::new();
            if let Some(val) = const_value.get_key(group_by) {
                helpers::value::hash(val, &mut hasher);
            }
            let hash = hasher.finish();
            self.allow(type_name.to_lowercase(), hash.to_string(), rate_limit)
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
