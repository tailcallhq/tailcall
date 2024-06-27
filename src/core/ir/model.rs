use std::collections::HashMap;
use std::fmt::Debug;
use std::num::NonZeroU64;

use async_graphql::Value;
use strum_macros::Display;

use super::discriminator::Discriminator;
use super::{EvalContext, ResolverContextLike};
use crate::core::blueprint::DynamicValue;
use crate::core::config::group_by::GroupBy;
use crate::core::graphql::{self};
use crate::core::http::HttpFilter;
use crate::core::{grpc, http};

#[derive(Clone, Debug, Display)]
pub enum IR {
    Dynamic(DynamicValue<Value>),
    #[strum(to_string = "{0}")]
    IO(IO),
    Cache(Cache),
    // TODO: Path can be implement using Pipe
    Path(Box<IR>, Vec<String>),
    ContextPath(Vec<String>),
    Protect(Box<IR>),
    Map(Map),
    Pipe(Box<IR>, Box<IR>),
    Discriminate(Discriminator, Box<IR>),
}

#[derive(Clone, Debug)]
pub struct Map {
    pub input: Box<IR>,
    // accept key return value instead of
    pub map: HashMap<String, String>,
}

#[derive(Clone, Debug, strum_macros::Display)]
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

#[derive(Clone, Copy, Debug)]
pub struct DataLoaderId(usize);

impl DataLoaderId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct IoId(u64);

impl IoId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

pub trait CacheKey<Ctx> {
    fn cache_key(&self, ctx: &Ctx) -> Option<IoId>;
}

#[derive(Clone, Debug)]
pub struct Cache {
    pub max_age: NonZeroU64,
    pub io: Box<IO>,
}

impl Cache {
    ///
    /// Wraps an expression with the cache primitive.
    /// Performance DFS on the cache on the expression and identifies all the IO
    /// nodes. Then wraps each IO node with the cache primitive.
    pub fn wrap(max_age: NonZeroU64, expr: IR) -> IR {
        expr.modify(move |expr| match expr {
            IR::IO(io) => Some(IR::Cache(Cache { max_age, io: Box::new(io.to_owned()) })),
            _ => None,
        })
    }
}

impl IR {
    pub fn pipe(self, next: Self) -> Self {
        IR::Pipe(Box::new(self), Box::new(next))
    }

    pub fn modify(self, mut f: impl FnMut(&IR) -> Option<IR>) -> IR {
        self.modify_inner(&mut f)
    }

    fn modify_box<F: FnMut(&IR) -> Option<IR>>(self, modifier: &mut F) -> Box<IR> {
        Box::new(self.modify_inner(modifier))
    }

    fn modify_inner<F: FnMut(&IR) -> Option<IR>>(self, modifier: &mut F) -> IR {
        let modified = modifier(&self);
        match modified {
            Some(expr) => expr,
            None => {
                let expr = self;
                match expr {
                    IR::Pipe(first, second) => {
                        IR::Pipe(first.modify_box(modifier), second.modify_box(modifier))
                    }
                    IR::ContextPath(path) => IR::ContextPath(path),
                    IR::Dynamic(_) => expr,
                    IR::IO(_) => expr,
                    IR::Cache(Cache { io, max_age }) => {
                        let expr = *IR::IO(*io).modify_box(modifier);
                        match expr {
                            IR::IO(io) => IR::Cache(Cache { io: Box::new(io), max_age }),
                            expr => expr,
                        }
                    }
                    IR::Path(expr, path) => IR::Path(expr.modify_box(modifier), path),
                    IR::Protect(expr) => IR::Protect(expr.modify_box(modifier)),
                    IR::Map(Map { input, map }) => {
                        IR::Map(Map { input: input.modify_box(modifier), map })
                    }
                    IR::Discriminate(discriminator, expr) => {
                        IR::Discriminate(discriminator, expr.modify_box(modifier))
                    }
                }
            }
        }
    }
}

impl<'a, Ctx: ResolverContextLike + Sync> CacheKey<EvalContext<'a, Ctx>> for IO {
    fn cache_key(&self, ctx: &EvalContext<'a, Ctx>) -> Option<IoId> {
        match self {
            IO::Http { req_template, .. } => req_template.cache_key(ctx),
            IO::Grpc { req_template, .. } => req_template.cache_key(ctx),
            IO::GraphQL { req_template, .. } => req_template.cache_key(ctx),
            IO::Js { .. } => None,
        }
    }
}
