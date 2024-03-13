use std::num::NonZeroU64;

use crate::blueprint::LocalRateLimit;

pub trait RateLimit {
    fn requests(&self) -> NonZeroU64;
    fn duration(&self) -> std::time::Duration;
}

impl RateLimit for LocalRateLimit {
    fn requests(&self) -> NonZeroU64 {
        self.requests
    }
    fn duration(&self) -> std::time::Duration {
        self.duration
    }
}
