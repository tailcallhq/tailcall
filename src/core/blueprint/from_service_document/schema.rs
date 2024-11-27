use async_graphql::parser::types::{SchemaDefinition, ServiceDocument, TypeSystemDefinition};
use tailcall_valid::{Valid, Validator};

use crate::core::blueprint::blueprint;
use crate::core::blueprint::from_service_document::{helpers, pos_name_to_string};
use crate::core::blueprint::from_service_document::from_service_document::BlueprintMetadata;

impl BlueprintMetadata {

    pub fn schema_definition(&self, doc: &ServiceDocument) -> Valid<SchemaDefinition, super::Error> {
        doc.definitions
            .iter()
            .find_map(|def| match def {
                TypeSystemDefinition::Schema(schema_definition) => Some(&schema_definition.node),
                _ => None,
            })
            .cloned()
            .map_or_else(|| Valid::succeed(SchemaDefinition {
                extend: false,
                directives: vec![],
                query: None,
                mutation: None,
                subscription: None,
            }), Valid::succeed)
    }

    // TODO: need to validate if type has resolvers
// on each step
    pub fn to_bp_schema_def(&self, doc: &ServiceDocument, schema_definition: &SchemaDefinition) -> Valid<blueprint::SchemaDefinition, super::Error> {
        helpers::extract_directives(schema_definition.directives.iter())
            .fuse(self.validate_query(doc, schema_definition.query.as_ref().map(pos_name_to_string)))
            .fuse(self.validate_mutation(schema_definition.mutation.as_ref().map(pos_name_to_string), schema_definition))
            .map(|(directives, query, mutation)| blueprint::SchemaDefinition { directives, query, mutation })
    }

    fn validate_query(&self, doc: &ServiceDocument, qry: Option<String>) -> Valid<String, super::Error> {
        Valid::from_option(qry, "Query root is missing".to_owned())
            .and_then(|query_type_name| {
                // println!("{:#?}", schema_definition);
                let find = doc.definitions.iter().find_map(|directive| {
                    if let TypeSystemDefinition::Type(ty) = directive {
                        if query_type_name.eq(ty.node.name.node.as_ref()) {
                            Some(())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
                Valid::from_option(find, "Query type is not defined".to_owned()).map(|_: ()| query_type_name)
            })
    }

    fn validate_mutation(&self, mutation: Option<String>, schema_definition: &SchemaDefinition) -> Valid<Option<String>, super::Error> {
        match mutation {
            Some(mutation_type_name) => {
                let find = schema_definition.directives.iter().find_map(|directive| {
                    if mutation_type_name.eq(directive.node.name.node.as_ref()) {
                        Some(())
                    } else {
                        None
                    }
                });
                Valid::from_option(find, "Mutation type is not defined".to_owned()).map(|_| Some(mutation_type_name))
            }
            None => Valid::succeed(None),
        }
    }
}