use async_graphql::parser::types::{EnumType, ServiceDocument, TypeDefinition, TypeKind, TypeSystemDefinition, UnionType};
use async_graphql::Positioned;
use tailcall_valid::{Valid, Validator};

use crate::core::blueprint;
use crate::core::blueprint::{Blueprint, Definition, ScalarTypeDefinition};
use crate::core::blueprint::from_service_document::{Error, helpers, pos_name_to_string};
use crate::core::config::Alias;
use crate::core::directive::DirectiveCodec;
use crate::core::scalar::Scalar;
use crate::core::try_fold::TryFold;

pub struct BlueprintMetadata {
    pub path: String,
}

impl BlueprintMetadata {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn to_blueprint<'a>(&self, doc: ServiceDocument) -> Valid<Blueprint, super::Error> {
        let schema = self.to_schema().transform::<Blueprint>(
            |schema, blueprint| blueprint.schema(schema),
            |blueprint| blueprint.schema,
        );

        // Create Definitions with fields and types without resolvers
        // and blind conversion to Definition instead of starting from root node(s)
        let definitions = self.to_type_defs()
            .transform::<Blueprint>(
                |defs, blueprint| blueprint.definitions(defs),
                |blueprint| blueprint.definitions,
            );

        schema
            .and(definitions)
            .try_fold(&doc, Blueprint::default())
    }

    fn to_schema(&self) -> TryFold<ServiceDocument, blueprint::SchemaDefinition, super::Error> {
        TryFold::<ServiceDocument, blueprint::SchemaDefinition, super::Error>::new(|doc, schema| {
            self.schema_definition(doc).and_then(|schema_def| {
                self.to_bp_schema_def(doc, &schema_def)
            })
        })
    }
    fn to_type_defs(&self) -> TryFold<ServiceDocument, Vec<Definition>, super::Error> {
        TryFold::<ServiceDocument, Vec<Definition>, super::Error>::new(|doc, defs| {
            let type_defs = doc.definitions.iter().filter_map(|c| {
                match c {
                    TypeSystemDefinition::Type(ty) => Some(ty),
                    _ => None,
                }
            }).collect::<Vec<_>>();
            self.populate_defs(type_defs)
                .and_then(|defs| self.populate_resolvers(defs))
        })
    }

    fn populate_defs(
        &self,
        type_defs: Vec<&Positioned<TypeDefinition>>,
    ) -> Valid<Vec<Definition>, super::Error> {
        Valid::from_iter(type_defs.iter(), |type_definition| {
            let type_kind = &type_definition.node.kind;
            match type_kind {
                TypeKind::Scalar => self.to_scalar_ty(type_definition),
                TypeKind::Union(union_) => self.to_union_ty(union_, type_definition),
                TypeKind::Enum(enum_) => self.to_enum_ty(enum_, type_definition),
                TypeKind::Object(obj) => self.to_object_ty(obj, type_definition),
                TypeKind::Interface(interface) => self.to_object_ty(interface, type_definition),
                TypeKind::InputObject(inp) => self.to_input_object_ty(inp, type_definition),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use tailcall_valid::Validator;

    use crate::core::blueprint::from_service_document::from_service_document::BlueprintMetadata;

    #[test]
    fn test_from_bp() {
        // Test code here
        let path = format!("{}/examples/hello.graphql", env!("CARGO_MANIFEST_DIR"));
        println!("{}", path);
        let doc = async_graphql::parser::parse_schema(std::fs::read_to_string(&path).unwrap()).unwrap();
        let bp = BlueprintMetadata::new(path)
            .to_blueprint(doc)
            .to_result()
            .unwrap();

        println!("{:#?}", bp.definitions);
    }
}