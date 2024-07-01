use schemars::schema::SchemaObject;

pub fn get_description<'a>(schema: &'a SchemaObject) -> Option<&'a String> {
    schema
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref())
}
