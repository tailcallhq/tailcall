/// Main module for generating GraphQL schema.
mod entity;
mod functions;
mod static_vars;
mod to_graphql;

use async_graphql::ServiceDocument;
use std::fs::File;
use anyhow::Result;
use crate::document::print;
use static_vars::GRAPHQL_SCHEMA_FILE;
use functions::generate_rc_file;

// Entry point for generating and writing the GraphQL schema to a file.
// Updates the GraphQL schema file.
pub fn update_gql() -> Result<()> {
    let mut doc = ServiceDocument::new();
    generate_rc_file(&mut doc)?;
    let file = File::create(GRAPHQL_SCHEMA_FILE)?;
    print(&doc, file)?;
    Ok(())
}