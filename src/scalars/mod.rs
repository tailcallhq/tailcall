use async_graphql_value::ConstValue;

pub use crate::scalars::email::Email;

mod email;

pub const CUSTOM_SCALARS: &[&str] = &["Email"];

#[derive(schemars::JsonSchema)]
/// A wrapper to store all custom scalar types
pub struct Scalars {
    pub email: Email,
}

pub fn get_scalar(name: &str) -> fn(&ConstValue) -> bool {
    match name {
        "Email" | "email" => Email::validate,
        &_ => |_| true,
    }
}
