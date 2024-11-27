#![allow(unused)]

use async_graphql::Positioned;
use async_graphql_value::Name;

mod from_service_document;
mod schema;
mod helpers;
mod object;
mod union;
mod scalar;
mod enum_ty;
mod input_object_ty;
pub(super) mod resolvers;
mod populate_resolvers;


pub(super) type Error = String;

pub(super) fn pos_name_to_string(pos: &Positioned<Name>) -> String {
    pos.node.to_string()
}
