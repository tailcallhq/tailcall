use std::collections::HashMap;
use std::num::NonZeroU64;

use strum_macros::Display;

use crate::core::blueprint::DynamicValue;
use crate::core::config::group_by::GroupBy;
use crate::core::http::HttpFilter;
use crate::core::ir::model::DataLoaderId;
use crate::core::{graphql, grpc, http};

#[derive(Clone, Debug, Display)]
pub enum IR {
    Dynamic(DynamicValue<serde_json::Value>),
    #[strum(to_string = "{0}")]
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
    // accept key return value instead of
    pub map: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct Cache {
    pub max_age: NonZeroU64,
    pub io: Box<IO>,
}

#[derive(Clone, Debug, Display)]
pub enum IO {
    Http {
        req_template: http::RequestTemplate,
        group_by: Option<GroupBy>,
        dl_id: Option<DataLoaderId>,
        http_filter: Option<HttpFilter>,
    },
    GraphQL {
        req_template: graphql::RequestTemplate,
        field_name: String,
        batch: bool,
        dl_id: Option<DataLoaderId>,
    },
    Grpc {
        req_template: grpc::RequestTemplate,
        group_by: Option<GroupBy>,
        dl_id: Option<DataLoaderId>,
    },
    Js {
        name: String,
    },
}

impl Cache {
    pub fn new(max_age: NonZeroU64, io: Box<IO>) -> Self {
        Self { max_age, io }
    }
}
