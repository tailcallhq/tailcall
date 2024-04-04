use std::fmt::{Display, Write};

use anyhow::{anyhow, Result};
use async_graphql::{
    parser::types::{Selection, SelectionSet},
    Name, Value,
};
use futures_util::future::join_all;
use indenter::indented;
use indexmap::IndexMap;

use crate::{
    blueprint::Definition,
    http::RequestContext,
    lambda::{EvaluationContext, ResolverContextLike},
    scalar::is_scalar,
};

use super::{
    execution::{PlanExecutor, SimpleExecutor},
    resolver::{FieldPlan, FieldPlanSelection, Id},
};

#[derive(Debug)]
pub enum Fields {
    Scalar(Option<Id>),
    Complex {
        field_plan_id: Option<Id>,
        children: IndexMap<Name, Fields>,
    },
}

impl Display for Fields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fields::Scalar(_id) => write!(f, "Scalar"),
            Fields::Complex { children, .. } => {
                for (name, fields) in children.iter() {
                    writeln!(f, "{}({:?}):", name, fields.field_plan_id())?;
                    writeln!(indented(f), "{}", fields)?;
                }

                Ok(())
            }
        }
    }
}

pub struct GeneralPlan {
    fields: Fields,
    pub field_plans: Vec<FieldPlan>,
}

pub struct ExecutionPlan<'a> {
    pub fields: Fields,
    selections: IndexMap<Id, FieldPlanSelection>,
    general_plan: &'a GeneralPlan,
}

impl Fields {
    pub fn field_plan_id(&self) -> &Option<Id> {
        match self {
            Fields::Scalar(id) => id,
            Fields::Complex { field_plan_id, .. } => field_plan_id,
        }
    }

    fn with_field_plan_id(self, id: Option<Id>) -> Self {
        match self {
            Fields::Scalar(_) => Fields::Scalar(id),
            Fields::Complex { children, .. } => Fields::Complex { field_plan_id: id, children },
        }
    }

    fn from_operation(
        current_field_plan_id: Option<Id>,
        field_plans: &mut Vec<FieldPlan>,
        definitions: &Vec<Definition>,
        name: &str,
    ) -> Self {
        let definition = definitions.iter().find(|def| def.name() == name);
        let mut children = IndexMap::new();

        if let Some(Definition::Object(type_def)) = definition {
            for field in &type_def.fields {
                let type_name = field.of_type.name();
                let resolver = field.resolver.clone();

                let id = if let Some(resolver) = resolver {
                    // TODO: figure out dependencies, for now just dumb mock for parent resolver
                    let depends_on: Vec<Id> =
                        current_field_plan_id.map(|id| vec![id]).unwrap_or_default();
                    let id = field_plans.len().into();
                    let field_plan = FieldPlan { id, resolver, depends_on };
                    field_plans.push(field_plan);
                    Some(id)
                } else {
                    None
                };

                let plan = if is_scalar(type_name) {
                    Self::Scalar(id)
                } else {
                    Self::from_operation(
                        id.or(current_field_plan_id),
                        field_plans,
                        definitions,
                        type_name,
                    )
                };
                children.insert(Name::new(&field.name), plan.with_field_plan_id(id));
            }
        }

        Self::Complex { field_plan_id: None, children }
    }

    pub fn prepare_for_request(
        &self,
        result_selection: &mut FieldPlanSelection,
        selections: &mut IndexMap<Id, FieldPlanSelection>,
        input_selection_set: &SelectionSet,
    ) -> Self {
        let (field_plan_id, children) = match self {
            Fields::Scalar(id) => {
                assert!(input_selection_set.items.is_empty());

                return Self::Scalar(*id);
            }
            Fields::Complex { field_plan_id, children } => (field_plan_id, children),
        };

        let mut req_children = IndexMap::new();
        for selection in &input_selection_set.items {
            let mut current_selection_set = FieldPlanSelection::default();

            match &selection.node {
                Selection::Field(field) => {
                    let name = &field.node.name.node;
                    let fields = children.get(name).unwrap();
                    let fields = fields.prepare_for_request(
                        &mut current_selection_set,
                        selections,
                        &field.node.selection_set.node,
                    );

                    if let Some(field_plan_id) = fields.field_plan_id() {
                        let field_selection = selections.entry(*field_plan_id);

                        match field_selection {
                            indexmap::map::Entry::Occupied(mut entry) => {
                                entry.get_mut().extend(current_selection_set)
                            }
                            indexmap::map::Entry::Vacant(slot) => {
                                slot.insert(current_selection_set);
                            }
                        }
                    } else {
                        result_selection.add(selection, current_selection_set);
                    }

                    req_children.insert(name.clone(), fields);
                }
                Selection::FragmentSpread(_) => todo!(),
                Selection::InlineFragment(_) => todo!(),
            }
        }

        Self::Complex { field_plan_id: *field_plan_id, children: req_children }
    }
}

impl GeneralPlan {
    pub fn from_operation(definitions: &Vec<Definition>, name: &str) -> Self {
        let mut field_plans = Vec::new();
        let fields = Fields::from_operation(None, &mut field_plans, definitions, name);

        Self { fields, field_plans }
    }
}

impl Display for GeneralPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GeneralPlan")?;
        let mut f = indented(f);

        writeln!(f, "fields:")?;
        writeln!(f, "{}", &self.fields)?;
        writeln!(f, "field_plans:")?;

        let mut f = indented(&mut f);
        for plan in self.field_plans.iter() {
            writeln!(f, "{}", plan)?;
        }

        Ok(())
    }
}

impl<'a> ExecutionPlan<'a> {
    pub fn from_request(general_plan: &'a GeneralPlan, selection_set: &SelectionSet) -> Self {
        let mut selections = IndexMap::new();
        let mut result_selection = FieldPlanSelection::default();
        let fields = general_plan.fields.prepare_for_request(
            &mut result_selection,
            &mut selections,
            selection_set,
        );

        Self { fields, selections, general_plan }
    }

    fn inner_execute<Executor: PlanExecutor + Send + Sync>(
        &self,
        value: Option<Value>,
        executor: &mut Executor,
        fields: &Fields,
    ) -> Result<Value> {
        match &fields {
            Fields::Scalar(id) => value.ok_or(anyhow!("Can't resolve value for scalar")),
            Fields::Complex { field_plan_id, children } => {
                let value = if let Some(id) = field_plan_id {
                    executor.resolved_value(id).transpose()?
                } else {
                    value
                };

                let Some(Value::Object(mut current_value_map)) = value else {
                    return Err(anyhow!("Can't resolve value as object"));
                };

                for (name, fields) in children {
                    let value = current_value_map.get(name);
                    let value = self.inner_execute(value.cloned(), executor, fields)?;

                    current_value_map.insert(name.to_owned(), value);
                }

                Ok(Value::Object(current_value_map))
            }
        }
    }

    pub async fn execute<Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        req_ctx: &'a RequestContext,
        graphql_ctx: &'a Ctx,
    ) -> Result<Value> {
        let mut executor = SimpleExecutor::new(&self.general_plan, &self);

        executor.resolve(req_ctx, graphql_ctx).await;

        self.inner_execute(
            Some(Value::Object(IndexMap::default())),
            &mut executor,
            &self.fields,
        )
    }
}

impl<'a> Display for ExecutionPlan<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ExecutionPlan")?;
        let f = &mut indented(f);
        writeln!(f, "fields:")?;
        writeln!(indented(f), "{}", &self.fields)?;
        writeln!(f, "selections:")?;

        let mut f = &mut indented(f);

        for (id, selection) in &self.selections {
            writeln!(f, "Resolver({}):", id)?;
            writeln!(indented(&mut f), "{}", selection)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod from_operation {
        use std::{fs, path::Path};

        use async_graphql::parser::parse_query;

        use crate::{
            blueprint::Blueprint,
            config::{Config, ConfigModule},
            http::RequestContext,
            lambda::EmptyResolverContext,
            query_plan::plan::{ExecutionPlan, GeneralPlan},
            valid::Validator,
        };

        #[tokio::test]
        async fn test_simple() {
            let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/query_plan/tests");
            let config = fs::read_to_string(root_dir.join("user-posts.graphql")).unwrap();
            let config = Config::from_sdl(&config).to_result().unwrap();
            let config = ConfigModule::from(config);
            let blueprint = Blueprint::try_from(&config).unwrap();

            let general_plan =
                GeneralPlan::from_operation(&blueprint.definitions, &blueprint.query());

            insta::assert_snapshot!(general_plan);

            let document = parse_query(
                r#"
                query Users {
                    user(id: 1) {
                        name,
                        email
                    }
                }
                query PostsSimple {
                    posts { title }
                }
                query PostsComplex {
                    posts {title body user {name username website}}
                }
                query PostAndUser {
                    posts { title user { name } }
                    user { username email }
                }
            "#,
            )
            .unwrap();

            for (name, operation) in document.operations.iter() {
                let name = name.unwrap().to_string();
                let execution_plan =
                    ExecutionPlan::from_request(&general_plan, &operation.node.selection_set.node);

                insta::assert_snapshot!(name.clone(), execution_plan);

                let runtime = crate::cli::runtime::init(&Blueprint::default());
                let req_ctx = RequestContext::new(runtime);
                let graphql_ctx = EmptyResolverContext {};
                let result = execution_plan.execute(&req_ctx, &graphql_ctx).await;

                // TODO: remove error check
                if let Ok(result) = result {
                    insta::assert_json_snapshot!(name.clone(), result);
                }
            }
        }
    }
}
