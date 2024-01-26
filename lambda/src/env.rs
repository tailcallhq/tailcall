use tailcall::EnvIO;

#[derive(Clone)]
pub struct LambdaEnv;

impl EnvIO for LambdaEnv {
    fn get(&self, key: &str) -> Option<String> {
        // AWS Lambda sets environment variables
        // as real env vars in the runtime.
        std::env::var(key).ok()
    }
}
