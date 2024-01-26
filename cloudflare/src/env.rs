use std::rc::Rc;

use corex::EnvIO;
use worker::Env;

pub struct CloudflareEnv {
    env: Rc<Env>,
}

unsafe impl Send for CloudflareEnv {}
unsafe impl Sync for CloudflareEnv {}

impl EnvIO for CloudflareEnv {
    fn get(&self, key: &str) -> Option<String> {
        self.env.var(key).ok().map(|s| s.to_string())
    }
}

impl CloudflareEnv {
    pub fn init(env: Rc<Env>) -> Self {
        Self { env }
    }
}
