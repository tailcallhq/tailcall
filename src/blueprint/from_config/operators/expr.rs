use crate::try_fold::TryFold;
use crate::valid::Valid;
use crate::{blueprint::*, config};
use crate::config::{Config, Field, ExprNode};

pub fn update_expr<'a>(operation_type: &'a config::GraphQLOperationType) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
    TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(config, field, ty, name), b_field| {
        let Some(expr) = &field.expr else {
            return Valid::succeed(b_field);
        };

        match &expr.body {
            ExprNode::Http(http) => {
                let field_with_http = (*field).clone().http(http.clone());
                let http_field_def = update_http()
                    .try_fold(&(config, &field_with_http, ty, name), b_field);
                http_field_def
            },
            ExprNode::Const(const_field) => {
                let field_with_const = (*field).clone().const_field(const_field.clone());
                let const_field_def = update_const_field()
                    .try_fold(&(config, &field_with_const, ty, name), b_field);
                const_field_def
            }
            ExprNode::GraphQL(gql) => {
                let field_with_gql = (*field).clone().graphql(gql.clone());
                let gql_field_def = update_graphql(operation_type)
                    .try_fold(&(config, &field_with_gql, ty, name), b_field);
                gql_field_def
            }
            ExprNode::Grpc(grpc) => {
                let field_with_grpc = (*field).clone().grpc(grpc.clone());
                let grpc_field_def = update_grpc(operation_type)
                    .try_fold(&(config, &field_with_grpc, ty, name), b_field);
                grpc_field_def
            }
            ExprNode::If { condition, then, els } => {
                //
            }

            _ => Valid::fail(format!("invalid expr: unsupported operator in body"))
        }
    })
}
