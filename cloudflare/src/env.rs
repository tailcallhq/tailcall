use std::borrow::Cow;
use std::rc::Rc;

use tailcall::EnvIO;
use worker::Env;

pub struct CloudflareEnv {
    env: Rc<Env>,
}

unsafe impl Send for CloudflareEnv {}
unsafe impl Sync for CloudflareEnv {}

impl EnvIO for CloudflareEnv {
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        self.env.var(key).ok().map(Cow::from)
    }
}

impl CloudflareEnv {
    pub fn init(env: Rc<Env>) -> Self {
        Self { env }
    }
}
