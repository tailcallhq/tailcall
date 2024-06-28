use std::collections::HashMap;
use std::num::NonZeroU64;

use crate::core::blueprint::DynamicValue;
use crate::core::config::group_by::GroupBy;
use crate::core::http::HttpFilter;
use crate::core::{graphql, grpc, http};
use crate::core::ir::model::{CacheKey, IoId};
use crate::core::jit::Eval;

#[derive(Clone, Debug)]
pub enum IR {
    Dynamic(DynamicValue<serde_json::Value>),
    IO(IO),
    Cache(Cache),
    Path(Vec<String>),
    Protect,
    Map(Map),
    Pipe(Box<IR>, Box<IR>),
}

#[derive(Clone, Debug)]
pub struct Map {
    pub input: Box<IR>,
    pub map: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct Cache {
    pub max_age: NonZeroU64,
    pub io: IO,
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct IO {
    pub group_by: Option<GroupBy>,
    pub protocol: Protocol,
}

#[derive(Clone, Debug)]
pub enum Protocol {
    Http {
        template: http::RequestTemplate,
        http_filter: Option<HttpFilter>,
    },
    GraphQL {
        template: graphql::RequestTemplate,
        field_name: String,
        batch: bool,
    },
    Grpc {
        template: grpc::RequestTemplate,
    },
    Script {
        name: String,
    },
}

impl CacheKey<Eval> for IO {
    fn cache_key(&self, ctx: &Eval) -> Option<IoId> {
        let protocol = &self.protocol;
        match protocol {
            Protocol::Http { template, .. } => template.cache_key(ctx),
            Protocol::Grpc { template, .. } => template.cache_key(ctx),
            Protocol::GraphQL { template, .. } => template.cache_key(ctx),
            Protocol::Script { .. } => None,
        }
    }
}

impl Cache {
    pub fn new(max_age: NonZeroU64, io: IO) -> Self {
        Self { max_age, io }
    }
}
