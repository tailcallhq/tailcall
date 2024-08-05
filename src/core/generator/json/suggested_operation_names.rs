use crate::core::{
    config::{Config, GraphQLOperationType},
    valid::Valid,
    Transform,
};

pub struct UserSuggestsOperationNames<'a> {
    suggest_op_name: &'a str,
    operation_type: &'a GraphQLOperationType,
}

impl<'a> UserSuggestsOperationNames<'a> {
    pub fn new(suggest_op_name: &'a str, operation_type: &'a GraphQLOperationType) -> Self {
        Self { suggest_op_name, operation_type }
    }
}

impl Transform for UserSuggestsOperationNames<'_> {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        match self.operation_type {
            GraphQLOperationType::Query => {
                let suggested_name = self.suggest_op_name.to_owned();
                if let Some(ty) = config
                    .types
                    .remove(self.operation_type.to_string().as_str())
                {
                    config.types.insert(suggested_name.to_string(), ty);
                }
                config.schema.query = Some(suggested_name);
            }
            GraphQLOperationType::Mutation => {
                let suggested_name = self.suggest_op_name.to_owned();
                if let Some(ty) = config
                    .types
                    .remove(self.operation_type.to_string().as_str())
                {
                    config.types.insert(suggested_name.to_string(), ty);
                }
                config.schema.mutation = Some(suggested_name)
            }
        }

        Valid::succeed(config)
    }
}
