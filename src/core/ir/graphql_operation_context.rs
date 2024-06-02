pub trait GraphQLOperationContext {
    fn selection_set(&self) -> Option<String>;
}
