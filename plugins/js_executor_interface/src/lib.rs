pub trait JsExecutor {
  fn eval(&self, input: &str) -> Result<String, String>;
}