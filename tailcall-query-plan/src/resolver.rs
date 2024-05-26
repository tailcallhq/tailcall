use std::fmt::{Display, Write};
use std::ops::Deref;

use anyhow::Result;
use async_graphql::parser::types::{Field, Selection, SelectionSet};
use async_graphql::{Positioned, Value};
use indenter::indented;
use tailcall::core::ir::{Eval, EvaluationContext, ResolverContextLike, IR};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(pub usize);

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

impl Deref for Id {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct FieldPlan {
    pub(super) id: Id,
    pub(super) resolver: IR,
    pub(super) depends_on: Vec<Id>,
}

impl Display for FieldPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FieldPlan[{}] ({}) depends on [{}]",
            &self.id,
            &self.resolver,
            &self
                .depends_on
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl FieldPlan {
    pub async fn eval<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
        &'a self,
        ctx: EvaluationContext<'a, Ctx>,
    ) -> Result<Value> {
        Ok(self.resolver.eval(ctx).await?)
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
