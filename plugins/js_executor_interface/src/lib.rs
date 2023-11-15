use async_graphql_value::ConstValue;

pub trait JsExecutor {
  fn eval(&self, input: ConstValue) -> Result<ConstValue, String>;
}