mod service_document;
mod graphql_writer;
mod utils;

use anyhow::Result;
use service_document::generate_service_document;
use graphql_writer::print_service_document;
use std::fs::File;

static GRAPHQL_SCHEMA_FILE: &str = "generated/.tailcallrc.graphql";

pub fn update_gql() -> Result<()> {
    let service_document = generate_service_document()?;
    let mut file = File::create(GRAPHQL_SCHEMA_FILE)?;
    print_service_document(&service_document, &mut file)?;
    Ok(())
}
