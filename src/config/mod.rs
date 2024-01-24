pub use auth::*;
pub use config::*;
pub use expr::*;
pub use key_values::*;
pub use server::*;
pub use source::*;
pub use upstream::*;
mod auth;
mod config;
mod expr;
mod from_document;
pub mod group_by;
mod into_document;
mod key_values;
mod n_plus_one;
pub mod reader;
mod server;
mod source;
mod upstream;

fn is_default<T: Default + Eq>(val: &T) -> bool {
  *val == T::default()
}
