use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub trait RateLimitKey<Ctx> {
    fn key(&self, ctx: Ctx) -> Key;
}

#[derive(PartialEq, Eq, Hash)]
pub enum Key {
    Local(u64),
    Global,
}

impl RateLimitKey<()> for hyper::Request<hyper::Body> {
    fn key(&self, _ctx: ()) -> Key {
        let mut hasher = DefaultHasher::new();
        self.uri().to_string().hash(&mut hasher);
        Key::Local(hasher.finish())
    }
}
