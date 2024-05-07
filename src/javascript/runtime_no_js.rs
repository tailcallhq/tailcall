use async_graphql_value::ConstValue;

use crate::{blueprint, WorkerIO};

#[derive(Debug)]
pub struct Runtime;

impl Runtime {
    pub fn new(_: String, _: blueprint::Script) -> Self {
        panic!("JavaScript runtime is not supported in this build")
    }
}

#[async_trait::async_trait]
impl WorkerIO<Option<ConstValue>, ConstValue> for Runtime {
    async fn call(&self, _: Option<ConstValue>) -> anyhow::Result<Option<ConstValue>> {
        panic!("JavaScript runtime is not supported in this build")
    }
}
