use std::fmt::{Display, Write};

use anyhow::{anyhow, Result};
use async_graphql::{
    parser::types::{Selection, SelectionSet},
    Name, Value,
};
use indenter::indented;
use indexmap::IndexMap;

use crate::{blueprint::Definition, scalar::is_scalar};

use super::{
    execution::executor::ExecutionResult,
    resolver::{FieldPlan, FieldPlanSelection, Id},
};

#[derive(Debug)]
pub struct FieldTree {
    pub field_plan_id: Option<Id>,
    pub children: Option<IndexMap<Name, FieldTree>>,
}

impl Display for FieldTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(children) = &self.children {
            for (name, tree) in children.iter() {
                writeln!(f, "{}({:?}):", name, tree.field_plan_id)?;
                writeln!(indented(f), "{}", tree)?;
            }
        } else {
            write!(f, "Scalar")?;
        }

        Ok(())
    }
}

pub struct GeneralPlan {
    fields: FieldTree,
    pub field_plans: Vec<FieldPlan>,
}

pub struct OperationPlan {
    pub field_tree: FieldTree,
    selections: IndexMap<Id, FieldPlanSelection>,
}

impl FieldTree {
    fn is_scalar(&self) -> bool {
        self.children.is_none()
    }

    fn scalar(field_plan_id: Option<Id>) -> Self {
        Self { field_plan_id, children: None }
    }

    fn with_field_plan_id(self, id: Option<Id>) -> Self {
        Self { field_plan_id: id, children: self.children }
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
                    Self { field_plan_id: id, children: None }
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

        Self { field_plan_id: None, children: Some(children) }
    }

    pub fn prepare_for_request(
        &self,
        result_selection: &mut FieldPlanSelection,
        selections: &mut IndexMap<Id, FieldPlanSelection>,
        input_selection_set: &SelectionSet,
    ) -> Self {
        let Some(children) = &self.children else {
            assert!(input_selection_set.items.is_empty());
            return Self::scalar(self.field_plan_id);
        };

        let mut req_children = IndexMap::new();
        for selection in &input_selection_set.items {
            let mut current_selection_set = FieldPlanSelection::default();

            match &selection.node {
                Selection::Field(field) => {
                    let name = &field.node.name.node;
                    let fields = children.get(name).unwrap();
                    let tree = fields.prepare_for_request(
                        &mut current_selection_set,
                        selections,
                        &field.node.selection_set.node,
                    );

                    if let Some(field_plan_id) = tree.field_plan_id {
                        let field_selection = selections.entry(field_plan_id);

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

                    req_children.insert(name.clone(), tree);
                }
                Selection::FragmentSpread(_) => todo!(),
                Selection::InlineFragment(_) => todo!(),
            }
        }

        Self {
            field_plan_id: self.field_plan_id,
            children: Some(req_children),
        }
    }
}

impl GeneralPlan {
    pub fn from_operation(definitions: &Vec<Definition>, name: &str) -> Self {
        let mut field_plans = Vec::new();
        let fields = FieldTree::from_operation(None, &mut field_plans, definitions, name);

        Self { fields, field_plans }
    }
}

impl Display for GeneralPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GeneralPlan")?;
        let f = &mut indented(f);

        writeln!(f, "fields:")?;
        writeln!(indented(f), "{}", &self.fields)?;
        writeln!(f, "field_plans:")?;

        let f = &mut indented(f);
        for plan in self.field_plans.iter() {
            writeln!(f, "{}", plan)?;
        }

        Ok(())
    }
}

impl OperationPlan {
    pub fn from_request(general_plan: &GeneralPlan, selection_set: &SelectionSet) -> Self {
        let mut selections = IndexMap::new();
        let mut result_selection = FieldPlanSelection::default();
        let fields = general_plan.fields.prepare_for_request(
            &mut result_selection,
            &mut selections,
            selection_set,
        );

        Self { field_tree: fields, selections }
    }

    fn inner_collect(
        tree: &FieldTree,
        execution_result: &mut ExecutionResult,
        current_value: Option<Value>,
    ) -> Result<Value> {
        let value = if let Some(id) = &tree.field_plan_id {
            execution_result.resolved(&id).transpose()?
        } else {
            current_value
        };

        if let Some(children) = &tree.children {
            let mut current_map = if let Some(Value::Object(current_map)) = value {
                Some(current_map)
            } else {
                None
            };
            let mut new_map = IndexMap::with_capacity(children.len());

            for (name, tree) in children {
                let value = Self::inner_collect(
                    tree,
                    execution_result,
                    current_map.as_mut().and_then(|map| map.swap_remove(name)),
                )?;

                new_map.insert(name.clone(), value);
            }

            Ok(Value::Object(new_map))
        } else {
            value.ok_or(anyhow!("Can't resolve value for field"))
        }
    }

    pub fn collect_value(&self, mut execution_result: ExecutionResult) -> Result<Value> {
        Self::inner_collect(&self.field_tree, &mut execution_result, None)
    }
}

impl Display for OperationPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "OperationPlan")?;
        let f = &mut indented(f);
        writeln!(f, "fields:")?;
        writeln!(indented(f), "{}", &self.field_tree)?;
        writeln!(f, "selections:")?;

        let mut f = &mut indented(f);

        for (id, selection) in &self.selections {
            writeln!(f, "Resolver({}):", id)?;
            writeln!(indented(&mut f), "{}", selection)?;
        }

        Ok(())
    }
}
