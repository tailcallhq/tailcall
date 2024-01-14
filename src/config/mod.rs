pub use config::*;
pub use key_values::*;
pub use server::*;
pub use source::*;
pub use opentelemetry::*;

mod config;
mod from_document;
pub mod group_by;
mod into_document;
mod key_values;
mod n_plus_one;
mod opentelemetry;
pub mod reader;
mod server;
mod source;

fn is_default<T: Default + Eq>(val: &T) -> bool {
  *val == T::default()
}
