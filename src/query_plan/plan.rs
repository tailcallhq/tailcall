use async_graphql::{
    parser::types::{Selection, SelectionSet},
    Name,
};
use indexmap::IndexMap;

use crate::{
    blueprint::{Blueprint, Definition, FieldDefinition, ObjectTypeDefinition},
    lambda::Expression,
    scalar::is_scalar,
};

use super::resolver::PlanResolver;

#[derive(Default, Clone)]
pub struct FieldPlan {
    resolver: Option<Expression>,
}

impl std::fmt::Debug for FieldPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldPlan")
            .field(
                "resolver",
                &if let Some(resolver) = &self.resolver {
                    Some(resolver.to_string())
                } else {
                    None
                },
            )
            .finish()
    }
}

#[derive(Debug)]
enum Plan {
    Scalar(FieldPlan),
    Complex {
        field_plan: FieldPlan,
        children: IndexMap<Name, Plan>,
    },
}

impl Plan {
    fn field_plan(&self) -> &FieldPlan {
        match self {
            Plan::Scalar(plan) => plan,
            Plan::Complex { field_plan, children } => field_plan,
        }
    }

    fn with_resolver(self, resolver: Option<Expression>) -> Self {
        match self {
            Plan::Scalar(_) => Plan::Scalar(FieldPlan { resolver }),
            Plan::Complex { children, .. } => {
                Plan::Complex { field_plan: FieldPlan { resolver }, children: children }
            }
        }
    }
}

impl Plan {
    pub fn from_operation(definitions: &Vec<Definition>, name: &str) -> Self {
        let definition = definitions.iter().find(|def| def.name() == name);
        let mut children = IndexMap::new();

        if let Some(Definition::Object(type_def)) = definition {
            for field in &type_def.fields {
                let type_name = field.of_type.name();
                let resolver = field.resolver.clone();

                let plan = if is_scalar(type_name) {
                    Plan::Scalar(FieldPlan::default())
                } else {
                    Self::from_operation(definitions, type_name)
                };
                children.insert(Name::new(&field.name), plan.with_resolver(resolver));
            }
        }

        Self::Complex { field_plan: FieldPlan::default(), children }
    }

    pub fn from_request(&self, selection_set: &SelectionSet) -> Self {
        let Self::Complex { field_plan, children } = self else {
            return Self::Scalar(self.field_plan().clone());
        };

        let mut req_children = IndexMap::new();
        for selection in &selection_set.items {
            match &selection.node {
                Selection::Field(field) => {
                    let name = &field.node.name.node;
                    let plan = children
                        .get(name)
                        .unwrap()
                        .from_request(&field.node.selection_set.node);
                    req_children.insert(name.clone(), plan);
                }
                Selection::FragmentSpread(_) => todo!(),
                Selection::InlineFragment(_) => todo!(),
            }
        }

        Self::Complex { field_plan: field_plan.clone(), children: req_children }
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
            query_plan::plan::Plan,
            valid::Validator,
        };

        #[test]
        fn test_simple() {
            let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/query_plan/tests");
            let config = fs::read_to_string(root_dir.join("user-posts.graphql")).unwrap();
            let config = Config::from_sdl(&config).to_result().unwrap();
            let config = ConfigModule::from(config);
            let blueprint = Blueprint::try_from(&config).unwrap();

            let plan = Plan::from_operation(&blueprint.definitions, &blueprint.query());

            insta::assert_debug_snapshot!(plan);

            let document = parse_query(
                r#"
                query PostsSimple { posts { title } }
                query PostsComplex { posts {title body user {name username}} }
            "#,
            )
            .unwrap();

            for (name, operation) in document.operations.iter() {
                let req_plan = plan.from_request(&operation.node.selection_set.node);

                insta::assert_debug_snapshot!(name.unwrap().to_string(), req_plan);
            }
        }
    }
}
