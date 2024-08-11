mod error;
pub mod infer_type_name;
pub use error::Error;
use error::Result;
pub use infer_type_name::InferTypeName;
mod wizard;
mod adapter;

pub use wizard::Wizard;
