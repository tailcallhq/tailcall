use async_graphql::Positioned;
use async_graphql_value::Name;

mod from_service_document;
mod schema;
mod helpers;


pub(super) type Error = String;

pub(super) fn pos_name_to_string(pos: &Positioned<Name>) -> String {
    pos.node.to_string()
}
