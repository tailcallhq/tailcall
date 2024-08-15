use indexmap::IndexMap;

use crate::core::blueprint::{
    Blueprint, Definition, FieldDefinition, InputFieldDefinition, SchemaDefinition,
};
use crate::core::scalar;

///
/// A read optimized index of all the types in the Blueprint. Provide O(1)
/// access to getting any field information.

#[derive(Debug)]
pub struct Index {
    map: IndexMap<String, (Definition, IndexMap<String, QueryField>)>,
    schema: SchemaDefinition,
}

#[derive(Debug)]
pub enum QueryField {
    Field((FieldDefinition, IndexMap<String, InputFieldDefinition>)),
    InputField(InputFieldDefinition),
}

impl QueryField {
    pub fn get_arg(&self, arg_name: &str) -> Option<&InputFieldDefinition> {
        match self {
            QueryField::Field((_, args)) => args.get(arg_name),
            QueryField::InputField(_) => None,
        }
    }
}

impl Index {
    pub fn type_is_scalar(&self, type_name: &str) -> bool {
        let def = self.map.get(type_name).map(|(def, _)| def);

        matches!(def, Some(Definition::Scalar(_))) || scalar::Scalar::is_predefined(type_name)
    }

    pub fn type_is_enum(&self, type_name: &str) -> bool {
        let def = self.map.get(type_name).map(|(def, _)| def);

        matches!(def, Some(Definition::Enum(_)))
    }

    pub fn validate_enum_value(&self, type_name: &str, value: &str) -> bool {
        let def = self.map.get(type_name).map(|(def, _)| def);

        if let Some(Definition::Enum(enum_)) = def {
            enum_.enum_values.iter().any(|v| v.name == value)
        } else {
            false
        }
    }

    pub fn get_field(&self, type_name: &str, field_name: &str) -> Option<&QueryField> {
        self.map
            .get(type_name)
            .and_then(|(_, fields_map)| fields_map.get(field_name))
    }

    pub fn get_query(&self) -> &String {
        &self.schema.query
    }

    pub fn get_mutation(&self) -> Option<&str> {
        self.schema.mutation.as_deref()
    }
}

impl From<&Blueprint> for Index {
    fn from(blueprint: &Blueprint) -> Self {
        let mut map = IndexMap::new();

        for definition in blueprint.definitions.iter() {
            match definition {
                Definition::Object(object_def) => {
                    let type_name = object_def.name.clone();
                    let mut fields_map = IndexMap::new();

                    for field in &object_def.fields {
                        let args_map = IndexMap::from_iter(
                            field
                                .args
                                .iter()
                                .map(|v| (v.name.clone(), v.clone()))
                                .collect::<Vec<_>>(),
                        );
                        fields_map.insert(
                            field.name.clone(),
                            QueryField::Field((field.clone(), args_map)),
                        );
                    }

                    map.insert(
                        type_name,
                        (Definition::Object(object_def.to_owned()), fields_map),
                    );
                }
                Definition::Interface(interface_def) => {
                    let type_name = interface_def.name.clone();
                    let mut fields_map = IndexMap::new();

                    for field in interface_def.fields.clone() {
                        let args_map = IndexMap::from_iter(
                            field
                                .args
                                .iter()
                                .map(|v| (v.name.clone(), v.clone()))
                                .collect::<Vec<_>>(),
                        );
                        fields_map.insert(field.name.clone(), QueryField::Field((field, args_map)));
                    }

                    map.insert(
                        type_name,
                        (Definition::Interface(interface_def.to_owned()), fields_map),
                    );
                }
                Definition::InputObject(input_object_def) => {
                    let type_name = input_object_def.name.clone();
                    let mut fields_map = IndexMap::new();

                    for field in input_object_def.fields.clone() {
                        fields_map.insert(field.name.clone(), QueryField::InputField(field));
                    }

                    map.insert(
                        type_name,
                        (
                            Definition::InputObject(input_object_def.to_owned()),
                            fields_map,
                        ),
                    );
                }
                Definition::Scalar(scalar_def) => {
                    let type_name = scalar_def.name.clone();
                    map.insert(
                        type_name.clone(),
                        (Definition::Scalar(scalar_def.to_owned()), IndexMap::new()),
                    );
                }
                Definition::Enum(enum_def) => {
                    let type_name = enum_def.name.clone();
                    map.insert(
                        type_name.clone(),
                        (Definition::Enum(enum_def.to_owned()), IndexMap::new()),
                    );
                }
                Definition::Union(union_def) => {
                    let type_name = union_def.name.clone();
                    map.insert(
                        type_name.clone(),
                        (Definition::Union(union_def.to_owned()), IndexMap::new()),
                    );
                }
            }
        }

        Self { map, schema: blueprint.schema.to_owned() }
    }
}

#[cfg(test)]
mod test {
    use crate::core::{
        blueprint::Blueprint,
        config::{Config, ConfigModule},
        valid::Validator,
    };

    use super::Index;

    fn setup() -> Index {
        let sdl = r#"
            schema
                @server(port: 8000)
                @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 100}) {
                query: Query
                mutation: Mutation
            }

            # Enum Type
            enum Status {
                ACTIVE
                INACTIVE
                PENDING
            }

            # Input Object Type
            input UserInput {
                name: String!
                email: String!
                status: Status
            }

            # Interface Type
            interface Node {
                id: ID!
                createdAt: DateTime!
                updatedAt: DateTime!
            }

            # Object Type
            type User implements Node {
                id: ID!
                name: String!
                email: String!
                status: Status
                createdAt: DateTime!
                updatedAt: DateTime!
            }

            # Union Type
            union SearchResult = User | Post

            # Object Type
            type Post implements Node {
                id: ID!
                title: String!
                content: String!
                author: User!
                createdAt: DateTime!
                updatedAt: DateTime!
            }

            # Query Type
            type Query {
                user(id: ID!): User @http(path: "/users/{{.args.id}}")
                search(term: String!): [SearchResult!] @http(path: "/search", query: [{key: "q", value: "{{.args.term}}"}])
            }

            input PostInput {
                title: String!
                content: String!
                authorId: ID!
            }

            # Mutation Type
            type Mutation {
                createUser(input: UserInput!): User! @http(path: "/users", body: "{{.args.input}}", method: "POST")
                createPost(input: PostInput): Post! @http(path: "/posts", body: "{{.args.input}}", method: "POST")
            }
        "#;

        let config = Config::from_sdl(sdl).to_result().unwrap();
        let cfg_module = ConfigModule::from(config);
        let blueprint = Blueprint::try_from(&cfg_module).unwrap();
        let index = Index::from(&blueprint);

        index
    }

    #[test]
    fn test_from_blueprint() {
        let index = setup();
        insta::assert_debug_snapshot!(index);
    }

    #[test]
    fn test_is_scalar() {
        let index = setup();
        assert!(index.type_is_scalar("Int"));
        assert!(index.type_is_scalar("String"));

        assert!(!index.type_is_scalar("Color"));
    }

    #[test]
    fn test_is_enum() {
        let index = setup();
        assert!(index.type_is_enum("Status"));
        assert!(!index.type_is_enum("Int"));
    }

    #[test]
    fn test_validate_enum_value() {
        let index = setup();
        assert!(index.validate_enum_value("Status", "ACTIVE"));
        assert!(!index.validate_enum_value("Status", "YELLOW"));
        assert!(!index.validate_enum_value("Int", "1"));
    }

    #[test]
    fn test_get_field() {
        let index = setup();
        assert!(index.get_field("Query", "user").is_some());
        assert!(index.get_field("Query", "non_existent_field").is_none());
        assert!(index.get_field("Status", "Pending").is_none());
    }

    #[test]
    fn test_get_query() {
        let index = setup();
        assert_eq!(index.get_query(), "Query");
    }

    #[test]
    fn test_get_mutation() {
        let index = setup();
        assert_eq!(index.get_mutation(), Some("Mutation"));
    }

    #[test]
    fn test_get_mutation_none() {
        let mut index = setup();
        index.schema.mutation = None;
        assert_eq!(index.get_mutation(), None);
    }
}
