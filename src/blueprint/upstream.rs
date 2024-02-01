use std::ops::Deref;

use super::TryFoldConfig;
use crate::config::{ConfigSet, Upstream};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};

pub fn to_upstream<'a>() -> TryFold<'a, ConfigSet, Upstream, String> {
    TryFoldConfig::<Upstream>::new(|config_set, up| {
        let upstream = up.merge_right(config_set.deref().upstream.clone());
        if let Some(ref base_url) = upstream.base_url {
            Valid::from(
                reqwest::Url::parse(base_url).map_err(|e| ValidationError::new(e.to_string())),
            )
            .map_to(upstream.clone())
        } else {
            Valid::succeed(upstream.clone())
        }
    })
}
