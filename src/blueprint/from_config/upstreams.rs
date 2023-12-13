use super::TryFoldConfig;
use std::collections::BTreeMap;

use crate::config::{Config, Upstream, Upstreams};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};

pub fn to_upstreams<'a>() -> TryFold<'a, Config, Upstreams, String> {
  TryFoldConfig::<Upstreams>::new(|config, up| {
    let upstreams = up.merge_right(&config.upstreams); // TODO merge right
    let mut upstream_map: BTreeMap<String, Upstream> = BTreeMap::new();

		Valid::from_iter(upstreams.0, |(name, upstream)| {
			if let Some(ref base_url) = upstream.base_url {
				Valid::from(reqwest::Url::parse(base_url).map_err(|e| ValidationError::new(e.to_string())))
					.map_to(upstream.clone())
			} else {
				Valid::succeed(upstream.clone())
			}
		}).map(|upstreams| {
			upstreams.iter().for_each(|upstream| {
				upstream_map.insert(upstream.name.clone().unwrap_or("default".to_string()), upstream.clone());
			});
			Upstreams(upstream_map)
		})
  })
}
