use async_graphql::ServiceDocument;
use std::io::Write;

pub fn print_service_document(service_document: &ServiceDocument, writer: &mut impl Write) -> std::io::Result<()> {
    let output = crate::document::print(service_document);
    writer.write_all(output.as_bytes())?;
    Ok(())
}
