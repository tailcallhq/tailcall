use std::collections::BTreeMap;

use super::TryFoldConfig;
use crate::config::{Config, Upstream, Upstreams};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};

pub fn to_upstreams<'a>() -> TryFold<'a, Config, Upstreams, String> {
  TryFoldConfig::<Upstreams>::new(|config, up| {
    let upstreams = up.merge_right(&config.upstreams);
    let mut upstream_map: BTreeMap<String, Upstream> = BTreeMap::new();

    Valid::from_iter(upstreams.0, |(_name, upstream)| {
      if let Some(ref base_url) = upstream.base_url {
        Valid::from(reqwest::Url::parse(base_url).map_err(|e| ValidationError::new(e.to_string())))
          .map_to(upstream.clone())
      } else {
        Valid::succeed(upstream.clone())
      }
    })
    .map(|mut upstreams| {
      upstreams.iter_mut().for_each(|upstream| {
        // if let None = upstream.name {
        // 	upstream.name = Some("default".to_string());
        // }
        upstream_map.insert(upstream.name.clone().unwrap_or("default".to_string()), upstream.clone());
      });
      if upstream_map.is_empty() {
        upstream_map.insert(
          "default".to_string(),
          Upstream { name: Some("default".to_string()), ..Default::default() },
        );
      }
      Upstreams(upstream_map)
    })
  })
}
