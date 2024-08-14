mod error;
pub mod infer_arg_name;
pub mod infer_type_name;
pub use error::Error;
use error::Result;
pub use infer_arg_name::InferArgName;
pub use infer_type_name::InferTypeName;
mod model;
mod wizard;

pub use wizard::Wizard;
