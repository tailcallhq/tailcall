use std::hash::{Hash, Hasher};

use tailcall_hasher::TailcallHasher;

use crate::core::async_graphql_hyper::GraphQLRequestLike;

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct OperationId(u64);

impl OperationId {
    pub fn from<T: GraphQLRequestLike + Hash>(bytes: &T, headers: &hyper::HeaderMap) -> Self {
        let key = key(bytes, headers);
        OperationId(key)
    }
}

fn key<T: GraphQLRequestLike + Hash>(bytes: &T, headers: &hyper::HeaderMap) -> u64 {
    let mut hasher = TailcallHasher::default();
    let state = &mut hasher;
    for (name, value) in headers.iter() {
        name.hash(state);
        value.hash(state);
    }
    bytes.hash(state);
    hasher.finish()
}
