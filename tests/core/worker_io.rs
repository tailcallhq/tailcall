use async_graphql_value::ConstValue;
use tailcall::javascript::{Command, Event};
use tailcall::WorkerIO;

pub struct JsRuntime {}

impl JsRuntime {
    pub fn init() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl WorkerIO<Event, Command> for JsRuntime {
    async fn call(&self, _: String, _: Event) -> anyhow::Result<Option<Command>> {
        todo!()
    }
}

#[async_trait::async_trait]
impl WorkerIO<Option<ConstValue>, ConstValue> for JsRuntime {
    async fn call(&self, _: String, _: Option<ConstValue>) -> anyhow::Result<Option<ConstValue>> {
        todo!()
    }
}
