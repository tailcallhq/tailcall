use std::fmt::{Display, Write};

use async_graphql::{
    parser::types::{Field, Selection, SelectionSet},
    Name, Pos, Positioned,
};
use indenter::indented;
use indexmap::IndexMap;

use crate::{
    blueprint::{Blueprint, Definition, FieldDefinition, ObjectTypeDefinition},
    lambda::Expression,
    scalar::is_scalar,
};

use super::resolver::{FieldPlan, FieldPlanSelection, Id};

#[derive(Debug)]
enum Fields {
    Scalar(Option<Id>),
    Complex {
        field_plan_id: Option<Id>,
        children: IndexMap<Name, Fields>,
    },
}

impl Display for Fields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fields::Scalar(id) => write!(f, "Scalar"),
            Fields::Complex { children, .. } => {
                let mut f = indented(f);

                for (name, fields) in children.iter() {
                    writeln!(f, "{}({:?}):", name, fields.field_plan_id())?;
                    writeln!(indented(&mut f), "{}", fields)?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug)]
struct GeneralPlan {
    fields: Fields,
    field_plans: Vec<FieldPlan>,
}

#[derive(Debug)]
struct ExecutionPlan {
    fields: Fields,
    selections: IndexMap<Id, FieldPlanSelection>,
}

impl Fields {
    fn field_plan_id(&self) -> &Option<Id> {
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
                    // TODO: figure out dependencies
                    let depends_on: Vec<Id> = vec![];
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
                    Self::from_operation(field_plans, definitions, type_name)
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
        let fields = Fields::from_operation(&mut field_plans, definitions, name);

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

        for plan in self.field_plans.iter() {
            writeln!(f, "{}", plan)?;
        }

        Ok(())
    }
}

impl ExecutionPlan {
    pub fn from_request(plan: &GeneralPlan, selection_set: &SelectionSet) -> Self {
        let mut selections = IndexMap::new();
        let mut result_selection = FieldPlanSelection::default();
        let fields =
            plan.fields
                .prepare_for_request(&mut result_selection, &mut selections, selection_set);

        Self { fields, selections }
    }
}

impl Display for ExecutionPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ExecutionPlan")?;
        writeln!(f, "fields:")?;
        writeln!(indented(f), "{}", &self.fields)?;
        writeln!(f, "selections:")?;

        let mut f = indented(f);

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
            query_plan::plan::{ExecutionPlan, GeneralPlan},
            valid::Validator,
        };

        #[test]
        fn test_simple() {
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
                let execution_plan =
                    ExecutionPlan::from_request(&general_plan, &operation.node.selection_set.node);

                insta::assert_snapshot!(name.unwrap().to_string(), execution_plan);
            }
        }
    }
}
