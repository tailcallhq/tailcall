mod auth;
mod blueprint;
mod compress;
mod cors;
mod definitions;
mod dynamic_value;
mod from_config;
mod index;
mod into_schema;
mod links;
mod mustache;
mod operators;
mod schema;
mod server;
pub mod telemetry;
mod timeout;
mod union_resolver;
mod upstream;

pub use auth::*;
pub use blueprint::*;
pub use cors::*;
pub use definitions::*;
pub use dynamic_value::*;
pub use from_config::*;
pub use index::*;
pub use links::*;
pub use operators::*;
pub use schema::*;
pub use server::*;
pub use timeout::GlobalTimeout;
pub use upstream::*;

use crate::core::config::{Arg, ConfigModule, Field};
use crate::core::try_fold::TryFold;

pub type TryFoldConfig<'a, A> = TryFold<'a, ConfigModule, A, String>;

pub(crate) trait TypeLike {
    fn name(&self) -> &str;
    fn list(&self) -> bool;
    fn non_null(&self) -> bool;
    fn list_type_required(&self) -> bool;
}

impl TypeLike for Field {
    fn name(&self) -> &str {
        &self.type_of
    }

    fn list(&self) -> bool {
        self.list
    }

    fn non_null(&self) -> bool {
        self.required
    }

    fn list_type_required(&self) -> bool {
        self.list_type_required
    }
}

impl TypeLike for Arg {
    fn name(&self) -> &str {
        &self.type_of
    }

    fn list(&self) -> bool {
        self.list
    }

    fn non_null(&self) -> bool {
        self.required
    }

    fn list_type_required(&self) -> bool {
        false
    }
}

pub(crate) fn to_type<T>(field: &T, override_non_null: Option<bool>) -> Type
where
    T: TypeLike,
{
    let name = field.name();
    let list = field.list();
    let list_type_required = field.list_type_required();
    let non_null = if let Some(non_null) = override_non_null {
        non_null
    } else {
        field.non_null()
    };

    if list {
        Type::ListType {
            of_type: Box::new(Type::NamedType {
                name: name.to_string(),
                non_null: list_type_required,
            }),
            non_null,
        }
    } else {
        Type::NamedType { name: name.to_string(), non_null }
    }
}
