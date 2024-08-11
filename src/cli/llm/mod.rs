mod error;
pub mod infer_type_name;
pub use error::Error;
use error::Result;
pub use infer_type_name::InferTypeName;
mod adapter;
mod model;
mod wizard;

pub use wizard::Wizard;
