pub use crate::scalars::email::Email;

mod email;

#[derive(schemars::JsonSchema)]
/// A wrapper to store all custom scalar types
pub struct Scalars {
    pub email: Email,
}
