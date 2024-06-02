///
/// We need three executors for each query
/// 1. Global general purpose executor (WE have this currently)
/// 2. Query specific executor - optimized for each query
/// 4. ?? which is working a bit level
/// 5. Based on Data incoming and outgoing certain optimizations can further be
///    made.

mod model {
    use std::collections::HashMap;
    use std::fmt::{Debug, Formatter};

    use async_graphql::parser::types::{DocumentOperations, ExecutableDocument, Selection};
    use async_graphql::Positioned;
    
    use serde_json_borrow::{OwnedValue};

    use crate::core::blueprint::{Blueprint, Definition, FieldDefinition, InputFieldDefinition};
    use crate::core::ir::IR;
    use crate::core::merge_right::MergeRight;
    use crate::core::FromValue;

    trait IncrGen {
        fn gen(&mut self) -> Self;
    }

    #[derive(Debug)]
    pub enum Type {
        Named(String),
        List(Box<Type>),
        Required(Box<Type>),
    }

    #[derive(Debug)]
    pub struct Arg {
        pub id: ArgId,
        pub name: String,
        pub type_of: crate::core::blueprint::Type,
        pub value: Option<OwnedValue>,
        pub default_value: Option<OwnedValue>,
    }

    pub struct ArgId(usize);

    impl Debug for ArgId {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl IncrGen for ArgId {
        fn gen(&mut self) -> Self {
            let id = self.0;
            self.0 += 1;
            Self(id)
        }
    }

    impl ArgId {
        fn new(id: usize) -> Self {
            ArgId(id)
        }
    }

    trait Id {
        fn as_usize(&self) -> usize;
    }

    #[derive(Clone, PartialEq, Eq)]
    pub struct FieldId(usize);

    impl Debug for FieldId {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl FieldId {
        fn new(id: usize) -> Self {
            FieldId(id)
        }
    }

    impl IncrGen for FieldId {
        fn gen(&mut self) -> Self {
            let id = self.0;
            self.0 += 1;
            Self(id)
        }
    }

    pub struct Field<A> {
        pub id: FieldId,
        pub name: String,
        pub ir: Option<IR>,
        pub type_of: crate::core::blueprint::Type,
        pub args: Vec<Arg>,
        pub refs: Option<A>,
    }

    impl<A> Debug for Field<A> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let mut debug_struct = f.debug_struct("Field");
            debug_struct.field("id", &self.id);
            debug_struct.field("name", &self.name);
            if self.ir.is_some() {
                debug_struct.field("ir", &"Some(..)");
            }
            debug_struct.field("type_of", &self.type_of);
            if !self.args.is_empty() {
                debug_struct.field("args", &self.args);
            }
            if self.refs.is_some() {
                debug_struct.field("refs", &"Some(..)");
            }
            debug_struct.finish()
        }
    }

    #[derive(Debug)]
    pub struct Parent(FieldId);

    pub struct Children(Vec<FieldId>);

    #[derive(Debug)]
    pub struct QueryBlueprint {
        pub fields: Vec<Field<Parent>>,
    }

    impl QueryBlueprint {
        pub fn from_document(document: ExecutableDocument, blueprint: Blueprint) -> Self {
            let fields = convert_query_to_field(document, &blueprint.definitions).unwrap();
            Self { fields }
        }
    }

    fn convert_query_to_field<A>(
        document: ExecutableDocument,
        schema_definitions: &[Definition],
    ) -> anyhow::Result<Vec<Field<A>>> {
        let mut id = FieldId::new(0);
        let mut arg_id = ArgId::new(0);

        let mut fields = Vec::new();

        fn resolve_selection_set<A>(
            selection_set: Positioned<async_graphql_parser::types::SelectionSet>,
            schema_definitions: &[Definition],
            id: &mut FieldId,
            arg_id: &mut ArgId,
        ) -> Vec<Field<A>> {
            let mut fields = Vec::new();

            for selection in selection_set.node.items {
                if let Selection::Field(gql_field) = selection.node {
                    let field_name = gql_field.node.name.node.as_str();
                    let field_args = gql_field
                        .node
                        .arguments
                        .into_iter()
                        .map(|(k, v)| (k.node.as_str().to_string(), v.node.into_const().unwrap()))
                        .collect::<HashMap<String, async_graphql::Value>>();

                    if let Some(definition) = find_definition(field_name, schema_definitions) {
                        let mut args = vec![];
                        field_args.into_iter().for_each(|(k, v)| {
                            if let Some(arg) = find_definition_arg(&k, schema_definitions) {
                                let type_of = arg.of_type.clone();
                                let id = arg_id.gen();
                                let arg = Arg {
                                    id,
                                    name: k,
                                    type_of,
                                    value: Some(v.into_bvalue().into()),
                                    default_value: None,
                                };
                                args.push(arg);
                            }
                        });

                        let type_of = definition.of_type.clone();
                        fields = fields.merge_right(resolve_selection_set(
                            gql_field.node.selection_set.clone(),
                            schema_definitions,
                            id,
                            arg_id,
                        ));

                        let id = id.gen();
                        let field = Field {
                            id,
                            name: field_name.to_string(),
                            ir: definition.resolver.clone(),
                            type_of,
                            args,
                            refs: None,
                        };
                        fields.push(field);
                    }
                }
            }

            fields
        }

        match document.operations {
            DocumentOperations::Single(single) => {
                fields = resolve_selection_set(
                    single.node.selection_set,
                    schema_definitions,
                    &mut id,
                    &mut arg_id,
                );
            }
            DocumentOperations::Multiple(multiple) => {
                for (_, single) in multiple {
                    fields = resolve_selection_set(
                        single.node.selection_set,
                        schema_definitions,
                        &mut id,
                        &mut arg_id,
                    );
                }
            }
        }

        Ok(fields)
    }

    fn find_definition<'a>(
        name: &str,
        definitions: &'a [Definition],
    ) -> Option<&'a FieldDefinition> {
        for def in definitions {
            if let Definition::Object(object) = def {
                for field in &object.fields {
                    if field.name == name {
                        return Some(field);
                    }
                }
            }
        }
        None
    }

    fn find_definition_arg<'a>(
        name: &str,
        definitions: &'a [Definition],
    ) -> Option<&'a InputFieldDefinition> {
        for def in definitions {
            if let Definition::Object(object) = def {
                for field in &object.fields {
                    for arg in &field.args {
                        if arg.name == name {
                            return Some(arg);
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::reader::ConfigReader;

    #[tokio::test]
    async fn test_from_document() {
        let rt = crate::core::runtime::test::init(None);
        let reader = ConfigReader::init(rt);
        let config = reader
            .read("examples/jsonplaceholder.graphql")
            .await
            .unwrap();
        let blueprint = Blueprint::try_from(&config).unwrap();
        let query = r#"
            query {
                posts { user { id } }
            }
        "#;
        let document = async_graphql::parser::parse_query(query).unwrap();
        let q_blueprint = model::QueryBlueprint::from_document(document, blueprint);
        insta::assert_snapshot!(format!("{:#?}", q_blueprint));
    }
}

mod value {
    pub use serde_json_borrow::*;
}

mod cache {
    use super::model::FieldId;
    use super::value::OwnedValue;

    pub struct Cache {
        map: Vec<(FieldId, OwnedValue)>,
    }

    impl Cache {
        pub fn empty() -> Self {
            Cache { map: Vec::new() }
        }
        pub fn join(caches: Vec<Cache>) -> Self {
            todo!()
        }
        pub fn get(&self, key: FieldId) -> Option<&OwnedValue> {
            todo!()
        }
    }
}

mod executor {
    use futures_util::future;

    use super::cache::Cache;
    use super::model::{Field, FieldId, Parent, QueryBlueprint};
    use super::value::OwnedValue;
    use crate::core::ir::IR;

    pub struct ExecutionContext {
        blueprint: QueryBlueprint,
        cache: Cache,
    }

    impl ExecutionContext {
        pub async fn execute_ir(
            &self,
            ir: &IR,
            parent: Option<&OwnedValue>,
        ) -> anyhow::Result<OwnedValue> {
            todo!()
        }

        fn find_children(&self, id: FieldId) -> Vec<Field<Parent>> {
            todo!()
        }

        fn insert_field_value(&self, id: FieldId, value: OwnedValue) {
            todo!()
        }

        fn find_field(&self, id: FieldId) -> Option<&Field<Parent>> {
            self.blueprint.fields.iter().find(|field| field.id == id)
        }

        async fn execute_field(
            &self,
            id: FieldId,
            parent: Option<&OwnedValue>,
        ) -> anyhow::Result<()> {
            if let Some(field) = self.find_field(id.clone()) {
                if let Some(ir) = &field.ir {
                    let value = self.execute_ir(ir, parent).await?;

                    let children = self.find_children(id.clone());
                    future::join_all(
                        children
                            .into_iter()
                            .map(|child| self.execute_field(child.id, Some(&value))),
                    )
                    .await
                    .into_iter()
                    .collect::<anyhow::Result<Vec<_>>>()?;

                    self.insert_field_value(id, value);
                }
            }
            Ok(())
        }

        fn root(&self) -> Vec<&Field<Parent>> {
            self.blueprint
                .fields
                .iter()
                .filter(|field| field.refs.is_none())
                .collect::<Vec<_>>()
        }

        pub async fn execute(&self) -> anyhow::Result<()> {
            future::join_all(
                self.root()
                    .iter()
                    .map(|field| self.execute_field(field.id.to_owned(), None)),
            )
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
            Ok(())
        }
    }
}

mod synth {
    pub use serde_json_borrow::*;

    use super::cache::Cache;
    use super::model::QueryBlueprint;

    struct Synth {
        blueprint: QueryBlueprint,
        cache: Cache,
    }

    impl Synth {
        pub fn new(blueprint: QueryBlueprint) -> Self {
            Synth { blueprint, cache: Cache::empty() }
        }

        pub fn synthesize(&self) -> Value<'_> {
            let mut object = ObjectAsVec::default();

            let root_fields = self.blueprint.fields.iter().filter(|a| a.refs.is_none());

            for root_field in root_fields {
                let key = &root_field.name;
                let id = root_field.id.to_owned();
                if let Some(value) = self.cache.get(id) {
                    object.insert(key, value.get_value().to_owned());
                }
            }

            Value::Object(object)
        }
    }
}
