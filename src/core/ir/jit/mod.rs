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

    pub struct Field {
        pub parent_id: Option<FieldId>,
        pub id: FieldId,
        pub name: String,
        pub ir: Option<IR>,
        pub type_of: Type,
        pub args: Vec<Arg>,
    }

    pub struct QueryBlueprint {
        pub fields: Vec<Field>,
    }
}

mod value {

    #[derive(Default)]
    pub struct OwnedValue;
    pub struct Value<'a>(&'a OwnedValue);
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
    }
}

mod executor {
    use futures_util::future;

    use super::cache::Cache;
    use super::model::{Field, FieldId, QueryBlueprint};
    use super::value::OwnedValue;
    use crate::core::ir::IR;

    pub struct ExecutionContext {
        blueprint: QueryBlueprint,
        cache: Cache,
    }

    impl ExecutionContext {
        pub async fn execute_ir(&self, ir: &IR, parent: &OwnedValue) -> anyhow::Result<OwnedValue> {
            todo!()
        }

        pub fn find_children(&self, id: &FieldId) -> Vec<Field> {
            todo!()
        }

        pub fn with_context(&self, id: &FieldId) -> Self {
            todo!()
        }

        pub fn insert_field_value(&self, id: &FieldId, value: OwnedValue) {
            todo!()
        }

        pub fn find_field(&self, id: FieldId) -> Option<&Field> {
            self.blueprint.fields.iter().find(|field| field.id == id)
        }

        async fn execute_field(&self, id: FieldId, parent: &OwnedValue) -> anyhow::Result<()> {
            if let Some(field) = self.find_field(id) {
                if let Some(ir) = &field.ir {
                    let value = self.execute_ir(ir, parent).await?;

                    let children = self.find_children(&field.id);
                    future::join_all(
                        children
                            .into_iter()
                            .map(|child| self.execute_field(child.id, &value)),
                    )
                    .await
                    .into_iter()
                    .collect::<anyhow::Result<Vec<_>>>()?;

                    self.insert_field_value(&field.id, value);
                }
            }
            Ok(())
        }

        fn root_fields(&self) -> Vec<&Field> {
            self.blueprint
                .fields
                .iter()
                .filter(|field| field.parent_id.is_none())
                .collect::<Vec<_>>()
        }

        pub async fn execute(&self) -> anyhow::Result<()> {
            let fields = self.root_fields();
            let value = OwnedValue;
            future::join_all(
                fields
                    .iter()
                    .map(|field| self.execute_field(field.id.to_owned(), &value)),
            )
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
            Ok(())
        }
    }
}

mod synth {
    use super::cache::Cache;
    use super::model::QueryBlueprint;
    use super::value::Value;

    struct Synth {
        blueprint: QueryBlueprint,
        cache: Cache,
    }

    impl Synth {
        pub fn new(blueprint: QueryBlueprint) -> Self {
            Synth { blueprint, cache: Cache::empty() }
        }

        pub fn synthesize<'a>(&'a self) -> Value<'a> {
            todo!()
        }
    }
}
