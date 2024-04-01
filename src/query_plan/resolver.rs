use std::fmt::{Display, Write};

use crate::lambda::Expression;

use async_graphql::{
    parser::types::{Field, Selection, SelectionSet},
    Positioned,
};
use indenter::indented;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(usize);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<usize> for Id {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct FieldPlan {
    pub(super) id: Id,
    pub(super) resolver: Expression,
    pub(super) depends_on: Vec<Id>,
}

impl Display for FieldPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FieldPlan[{}] ({})", &self.id, &self.resolver)
    }
}

impl std::fmt::Debug for FieldPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldPlan")
            .field("id", &self.id)
            .field("resolver", &self.resolver.to_string())
            .field("depends_on", &self.depends_on)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct FieldPlanSelection(SelectionSet);

impl FieldPlanSelection {
    pub fn add(&mut self, selection: &Positioned<Selection>, plan_selection: FieldPlanSelection) {
        match &selection.node {
            Selection::Field(field) => self.0.items.push(Positioned::new(
                Selection::Field(Positioned::new(
                    Field {
                        selection_set: Positioned::new(
                            plan_selection.0,
                            field.node.selection_set.pos,
                        ),
                        ..field.node.clone()
                    },
                    field.pos,
                )),
                selection.pos,
            )),
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!(),
        }
    }

    pub fn extend(&mut self, other: FieldPlanSelection) {
        self.0.items.extend(other.0.items);
    }
}

impl Display for FieldPlanSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for selection in &self.0.items {
            match &selection.node {
                Selection::Field(field) => {
                    let name = field.node.name.node.as_str();
                    let mut f = indented(f);

                    let selection_set = &field.node.selection_set.node;

                    if selection_set.items.is_empty() {
                        writeln!(f, "{name}")?;
                    } else {
                        writeln!(f, "{name}:")?;
                        writeln!(f, "{}", FieldPlanSelection(selection_set.clone()))?;
                    }
                }
                Selection::FragmentSpread(_) => todo!(),
                Selection::InlineFragment(_) => todo!(),
            }
        }

        Ok(())
    }
}
