use async_graphql_parser::types::{ServiceDocument, TypeSystemDefinition};

mod common;
pub mod directive_definition;
mod enum_definition;
pub mod input_definition;
pub mod scalar_definition;

pub struct ServiceDocumentBuilder {
    definitions: Vec<TypeSystemDefinition>,
}

impl Default for ServiceDocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceDocumentBuilder {
    pub fn new() -> Self {
        Self { definitions: vec![] }
    }

    pub fn add_directive(
        mut self,
        definitions: Vec<TypeSystemDefinition>,
    ) -> ServiceDocumentBuilder {
        self.definitions.extend(definitions);
        self
    }

    pub fn add_scalar(mut self, definitions: TypeSystemDefinition) -> ServiceDocumentBuilder {
        self.definitions.push(definitions);
        self
    }

    pub fn add_input(mut self, definitions: TypeSystemDefinition) -> ServiceDocumentBuilder {
        self.definitions.push(definitions);
        self
    }

    pub fn build(self) -> ServiceDocument {
        ServiceDocument { definitions: self.definitions }
    }
}
