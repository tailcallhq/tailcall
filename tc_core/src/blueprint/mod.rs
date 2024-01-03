mod blueprint;
pub mod compress;
mod const_utils;
pub mod for_config;
mod into_schema;
mod timeout;
pub use blueprint::*;
pub use const_utils::*;
pub use for_config::*;
pub use timeout::GlobalTimeout;

pub fn is_default<T: Default + Eq>(val: &T) -> bool {
  *val == T::default()
}
