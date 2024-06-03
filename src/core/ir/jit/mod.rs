///
/// We need three executors for each query
/// 1. Global general purpose executor (WE have this currently)
/// 2. Query specific executor - optimized for each query
/// 4. ?? which is working a bit level
/// 5. Based on Data incoming and outgoing certain optimizations can further be
///    made.
mod field_index;
mod model {
    use std::collections::HashMap;
    use std::fmt::{Debug, Formatter};

    use async_graphql::parser::types::{DocumentOperations, ExecutableDocument, Selection};
    use async_graphql::Positioned;
    use serde_json_borrow::OwnedValue;

    use super::field_index::{FieldIndex, QueryField};
    use crate::core::blueprint::Blueprint;
    use crate::core::ir::IR;
    use crate::core::merge_right::MergeRight;
    use crate::core::FromValue;

    #[allow(unused)]
    trait IncrGen {
        fn gen(&mut self) -> Self;
    }

    #[allow(unused)]
    #[derive(Debug)]
    pub enum Type {
        Named(String),
        List(Box<Type>),
        Required(Box<Type>),
    }

    #[allow(unused)]
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

    #[allow(unused)]
    impl ArgId {
        fn new(id: usize) -> Self {
            ArgId(id)
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    pub struct FieldId(usize);

    impl Debug for FieldId {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[allow(unused)]
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
    #[allow(unused)]
    pub struct Parent(FieldId);

    #[allow(unused)]
    pub struct Children(Vec<FieldId>);

    #[derive(Debug)]
    pub struct QueryPlan {
        pub fields: Vec<Field<Parent>>,
    }

    #[allow(unused)]
    pub struct QueryPlanBuilder {
        index: FieldIndex,
    }

    impl QueryPlanBuilder {
        #[allow(unused)]
        pub fn new(blueprint: Blueprint) -> Self {
            let blueprint_index = FieldIndex::init(&blueprint);
            Self { index: blueprint_index }
        }

        #[allow(unused)]
        pub fn build(&self, document: ExecutableDocument) -> anyhow::Result<QueryPlan> {
            let fields = self.create_field_set(document)?;
            Ok(QueryPlan { fields })
        }

        #[allow(clippy::too_many_arguments)]
        fn resolve_selection_set<A>(
            &self,
            selection_set: Positioned<async_graphql_parser::types::SelectionSet>,
            id: &mut FieldId,
            arg_id: &mut ArgId,
            current_type: &str,
        ) -> anyhow::Result<Vec<Field<A>>> {
            let mut fields = Vec::new();

            for selection in selection_set.node.items {
                if let Selection::Field(gql_field) = selection.node {
                    let field_name = gql_field.node.name.node.as_str();
                    let field_args = gql_field
                        .node
                        .arguments
                        .into_iter()
                        .map(|(k, v)| (k.node.as_str().to_string(), v.node.into_const()))
                        .collect::<HashMap<String, Option<async_graphql::Value>>>();

                    if let Some(field_def) = self.index.get_field(current_type, field_name) {
                        let mut args = vec![];
                        for (arg_name, v) in field_args {
                            if let Some(arg) = field_def.get_arg(&arg_name) {
                                let type_of = arg.of_type.clone();
                                let id = arg_id.gen();
                                let arg = Arg {
                                    id,
                                    name: arg_name.clone(),
                                    type_of,
                                    value: Some(
                                        v.ok_or(anyhow::anyhow!(
                                            "failed converting Const to Borrowed Value"
                                        ))?
                                        .into_bvalue()
                                        .into(),
                                    ),
                                    default_value: arg.default_value.clone(),
                                };
                                args.push(arg);
                            }
                        }

                        let type_of = match field_def {
                            QueryField::Field((field_def, _)) => field_def.of_type.clone(),
                            QueryField::InputField(field_def) => field_def.of_type.clone(),
                        };

                        fields = fields.merge_right(self.resolve_selection_set(
                            gql_field.node.selection_set.clone(),
                            id,
                            arg_id,
                            type_of.name(),
                        )?);

                        let id = id.gen();
                        let field = Field {
                            id,
                            name: field_name.to_string(),
                            ir: match field_def {
                                QueryField::Field((field_def, _)) => field_def.resolver.clone(),
                                _ => None,
                            },
                            type_of,
                            args,
                            refs: None,
                        };
                        fields.push(field);
                    }
                }
            }

            Ok(fields)
        }

        fn create_field_set<A>(
            &self,
            document: ExecutableDocument,
        ) -> anyhow::Result<Vec<Field<A>>> {
            let query = self.index.get_query();
            let mut id = FieldId::new(0);
            let mut arg_id = ArgId::new(0);

            let mut fields = Vec::new();

            for (_, fragment) in document.fragments {
                fields = self.resolve_selection_set(
                    fragment.node.selection_set,
                    &mut id,
                    &mut arg_id,
                    query,
                )?;
            }

            match document.operations {
                DocumentOperations::Single(single) => {
                    fields = self.resolve_selection_set(
                        single.node.selection_set,
                        &mut id,
                        &mut arg_id,
                        query,
                    )?;
                }
                DocumentOperations::Multiple(multiple) => {
                    for (_, single) in multiple {
                        fields = self.resolve_selection_set(
                            single.node.selection_set,
                            &mut id,
                            &mut arg_id,
                            query,
                        )?;
                    }
                }
            }

            Ok(fields)
        }
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

        let q_blueprint = model::QueryPlanBuilder::new(blueprint)
            .build(document)
            .unwrap();
        insta::assert_snapshot!(format!("{:#?}", q_blueprint));
    }
}

mod value {
    pub use serde_json_borrow::*;
}

mod cache {
    use super::model::FieldId;
    use super::value::OwnedValue;

    #[allow(unused)]
    pub struct Cache {
        map: Vec<(FieldId, OwnedValue)>,
    }

    #[allow(unused)]
    impl Cache {
        #[allow(unused)]
        pub fn empty() -> Self {
            Cache { map: Vec::new() }
        }

        #[allow(unused)]
        pub fn join(caches: Vec<Cache>) -> Self {
            todo!()
        }
        #[allow(unused)]
        pub fn get(&self, key: FieldId) -> Option<&OwnedValue> {
            todo!()
        }
    }
}

mod executor {
    use futures_util::future;

    use super::cache::Cache;
    use super::model::{Field, FieldId, Parent, QueryPlan};
    use super::value::OwnedValue;
    use crate::core::ir::IR;

    #[allow(unused)]
    pub struct ExecutionContext {
        plan: QueryPlan,
        cache: Cache,
    }
    #[allow(unused)]
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
            self.plan.fields.iter().find(|field| field.id == id)
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
            self.plan
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
    use super::model::QueryPlan;

    struct Synth {
        blueprint: QueryPlan,
        cache: Cache,
    }
    #[allow(unused)]
    impl Synth {
        pub fn new(blueprint: QueryPlan) -> Self {
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
