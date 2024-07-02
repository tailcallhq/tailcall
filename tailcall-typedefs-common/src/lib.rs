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

#[cfg(test)]
mod tests {
    use schemars::JsonSchema;

    #[derive(JsonSchema)]
    enum Schema {
        Obj(String),
        Str,
        Any,
    }

    #[derive(JsonSchema)]
    struct Inpt2Dummy {
        field1: String,
        field2: i32,
        schema: Schema,
    }

    #[derive(JsonSchema)]
    struct InputDummy {
        field1: String,
        field2: Option<String>,
        field3: Inpt2Dummy,
    }

    #[derive(JsonSchema)]
    enum EnumDummy {
        Variant1,
        Variant2,
    }

    #[derive(JsonSchema)]
    struct DirectiveDummy {
        field1: i32,
        field2: Option<i32>,
        field3: Vec<i32>,
        enum_dummy: EnumDummy,
        input_dummy: Vec<InputDummy>,
    }
    #[test]
    fn it_works_for_to_directives() {}
}
