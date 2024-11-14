use std::{borrow::Cow, convert::Infallible, marker::PhantomData};

use tailcall_valid::Valid;

use crate::core::{
    ir::model::{IO, IR},
    jit::{Field, OperationPlan},
    Transform,
};

#[derive(Default)]
pub struct GraphQL<A>(PhantomData<A>);

impl<A> GraphQL<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<A: ToString> Transform for GraphQL<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;

    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        for field in plan.selection.iter_mut() {
            let rendered = field.render_graphql();
            if let Some(IR::IO(IO::GraphQL { req_template, .. })) = field.ir.as_mut() {
                req_template.selection = rendered;
            }
        }

        Valid::succeed(plan)
    }
}

impl<A: ToString> Field<A> {
    pub fn render_graphql(&self) -> Option<String> {
        format_selection_set(self.selection.iter())
    }
}

fn format_selection_set<'a, A: 'a + ToString>(
    selection_set: impl Iterator<Item = &'a Field<A>>,
) -> Option<String> {
    let set = selection_set
        .map(|field| format_selection_field(field, &field.name))
        .collect::<Vec<_>>();

    if set.is_empty() {
        return None;
    }

    Some(format!("{{ {} }}", set.join(" ")))
}

fn format_selection_field<A: ToString>(field: &Field<A>, name: &str) -> String {
    let arguments = format_selection_field_arguments(field);
    let selection_set = format_selection_set(field.selection.iter());

    let mut output = format!("{}{}", name, arguments);

    // if let Some(directives) = field.directives() {
    //     let directives = print_directives(directives.iter());

    //     if !directives.is_empty() {
    //         output.push(' ');
    //         output.push_str(&directives.escape_default().to_string());
    //     }
    // }

    if let Some(selection_set) = selection_set {
        output.push(' ');
        output.push_str(&selection_set);
    }

    output
}

fn format_selection_field_arguments<A: ToString>(field: &Field<A>) -> Cow<'static, str> {
    let arguments = field
        .args
        .iter()
        .filter_map(|a| a.value.as_ref())
        .collect::<Vec<_>>();

    if arguments.is_empty() {
        return Cow::Borrowed("");
    }

    let args = arguments
        .iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>()
        .join(",");

    Cow::Owned(format!("({})", args.escape_default()))
}
