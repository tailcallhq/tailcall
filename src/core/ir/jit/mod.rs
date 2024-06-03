///
/// We need three executors for each query
/// 1. Global general purpose executor (WE have this currently)
/// 2. Query specific executor - optimized for each query
/// 4. ?? which is working a bit level
/// 5. Based on Data incoming and outgoing certain optimizations can further be
///    made.

mod model {
    use crate::core::ir::IR;

    pub enum Type {
        Named(String),
        List(Box<Type>),
        Required(Box<Type>),
    }

    pub struct Arg {
        pub id: ArgId,
        pub name: String,
        pub type_of: Type,
    }

    pub struct ArgId(usize);
    impl ArgId {
        fn new(id: usize) -> Self {
            ArgId(id)
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    pub struct FieldId(usize);
    impl FieldId {
        fn new(id: usize) -> Self {
            FieldId(id)
        }
    }

    pub struct Field<A> {
        pub id: FieldId,
        pub name: String,
        pub ir: Option<IR>,
        pub type_of: Type,
        pub args: Vec<Arg>,
        pub refs: Option<A>,
    }

    pub struct Parent(FieldId);
    pub struct Children(pub(crate) Vec<Field<Children>>);

    pub struct QueryBlueprint {
        pub fields: Vec<Field<Parent>>,
    }

    impl QueryBlueprint {
        pub fn to_children(&self) -> Vec<Field<Children>> {
            todo!()
        }

        pub fn find_field(&self, id: FieldId) -> Option<&Field<Parent>> {
            self.fields.iter().find(|field| field.id == id)
        }
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
    use serde_json_borrow::Map;
    pub use serde_json_borrow::*;

    use super::cache::Cache;
    use super::model::{Children, Field, FieldId, QueryBlueprint};

    struct Synth {
        blueprint: QueryBlueprint,
        cache: Cache,
    }

    impl Synth {
        pub fn new(blueprint: QueryBlueprint) -> Self {
            Synth { blueprint, cache: Cache::empty() }
        }

        fn build_children(
            &self,
            field: Field<Children>,
            query_blueprint: QueryBlueprint,
        ) -> ObjectAsVec {
            let mut object = ObjectAsVec::default();
            match field.refs {
                None => (),
                Some(children) => {
                    for field in children.0 {
                        let field = query_blueprint.find_field(fieldId.clone()).unwrap();
                        let key = field.name.clone();
                        let id = field.id.clone();
                        match self.cache.get(id) {
                            Some(value) => {
                                object.insert(key.as_str(), value.get_value().to_owned());
                            }
                            None => (),
                        }
                    }
                }
            }
            object
        }

        pub fn synthesize<'a>(&'a self) -> Value<'a> {
            let mut object = ObjectAsVec::default();
            let root_fields = self.blueprint.to_children();
            for root_field in root_fields {
                let key = &root_field.name;
                let id = root_field.id.to_owned();
                match self.cache.get(id) {
                    Some(value) => {
                        object.insert(key, value.get_value().to_owned());
                    }
                    None => (),
                }
            }

            // let root_fields = self.blueprint.fields.iter().filter(|a| a.refs.is_none());
            //
            // for root_field in root_fields {
            //     let field = root_field.
            //     let key = &root_field.name;
            //     let id = root_field.id.to_owned();
            //     match self.cache.get(id) {
            //         Some(value) => {
            //             object.insert(key, value.get_value().to_owned());
            //         }
            //         None => (),
            //     }
            // }

            Value::Object(object)
        }
    }
}
