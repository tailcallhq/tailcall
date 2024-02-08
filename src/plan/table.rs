use crate::lambda::Expression;

pub struct Name(String);
pub struct Resolver {
    expression: Expression,
}
pub struct Field {
    name: Name,
    expression: Resolver,
    is_list: bool,
    is_required: bool,
    children: Vec<SelectionSet>,
    id: u64,
}

pub struct SelectionSet {
    selections: Vec<Field>,
}

