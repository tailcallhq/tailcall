mod config;
mod resolver;
mod from_document;
pub mod group_by;
mod into_document;
mod key_values;
mod n_plus_one;
mod reader;
mod server;
mod source;
mod writer;

pub use config::*;
pub use key_values::*;
pub use server::*;
pub use source::*;
pub use resolver::*;

fn is_default<T: Default + Eq>(val: &T) -> bool {
  *val == T::default()
}
