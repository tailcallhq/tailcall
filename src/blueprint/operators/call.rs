use std::collections::hash_map::Iter;

use crate::blueprint::*;
use crate::config::group_by::GroupBy;
use crate::config::{Config, Field, GraphQLOperationType};
use crate::lambda::{DataLoaderId, Expression, IO};
use crate::mustache::{Mustache, Segment};
use crate::try_fold::TryFold;
use crate::valid::Valid;
use crate::{config, graphql, grpc, http};

fn find_value<'a>(args: &'a Iter<'a, String, String>, key: &'a String) -> Option<&'a String> {
    args.clone()
        .find_map(|(k, value)| if k == key { Some(value) } else { None })
}

pub fn update_call(
    operation_type: &GraphQLOperationType,
) -> TryFold<'_, (&Config, &Field, &config::Type, &str), FieldDefinition, String> {
    TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
        move |(config, field, _, _), b_field| {
            let Some(call) = &field.call else {
                return Valid::succeed(b_field);
            };

            compile_call(field, config, call, operation_type)
                .and_then(|resolver| Valid::succeed(b_field.resolver(Some(resolver))))
        },
    )
}

struct Http {
    pub req_template: http::RequestTemplate,
    pub group_by: Option<GroupBy>,
    pub dl_id: Option<DataLoaderId>,
}

struct GraphQLEndpoint {
    pub req_template: graphql::RequestTemplate,
    pub field_name: String,
    pub batch: bool,
    pub dl_id: Option<DataLoaderId>,
}

struct Grpc {
    pub req_template: grpc::RequestTemplate,
    pub group_by: Option<GroupBy>,
    pub dl_id: Option<DataLoaderId>,
}

impl TryFrom<Expression> for Http {
    type Error = String;

    fn try_from(expr: Expression) -> Result<Self, Self::Error> {
        match expr {
            Expression::IO(IO::Http { req_template, group_by, dl_id }) => {
                Ok(Http { req_template, group_by, dl_id })
            }
            _ => Err("not an http expression".to_string()),
        }
    }
}

impl TryFrom<Expression> for GraphQLEndpoint {
    type Error = String;

    fn try_from(expr: Expression) -> Result<Self, Self::Error> {
        match expr {
            Expression::IO(IO::GraphQLEndpoint { req_template, field_name, batch, dl_id }) => {
                Ok(GraphQLEndpoint { req_template, field_name, batch, dl_id })
            }
            _ => Err("not a graphql expression".to_string()),
        }
    }
}

impl TryFrom<Expression> for Grpc {
    type Error = String;

    fn try_from(expr: Expression) -> Result<Self, Self::Error> {
        match expr {
            Expression::IO(IO::Grpc { req_template, group_by, dl_id }) => {
                Ok(Grpc { req_template, group_by, dl_id })
            }
            _ => Err("not a grpc expression".to_string()),
        }
    }
}

pub fn compile_call(
    field: &Field,
    config: &Config,
    call: &config::Call,
    operation_type: &GraphQLOperationType,
) -> Valid<Expression, String> {
    Valid::from_option(call.query.clone(), "call must have query".to_string())
        .and_then(|field_name| {
            Valid::from_option(
                config.find_type("Query"),
                "Query type not found on config".to_string(),
            )
            .zip(Valid::succeed(field_name))
        })
        .and_then(|(query_type, field_name)| {
            Valid::from_option(
                query_type.fields.get(&field_name),
                format!("{} field not found", field_name),
            )
            .zip(Valid::succeed(field_name))
            .and_then(|(field, field_name)| {
                if field.has_resolver() {
                    Valid::succeed((field, field_name, call.args.iter()))
                } else {
                    Valid::fail(format!("{} field has no resolver", field_name))
                }
            })
        })
        .and_then(|(_field, field_name, args)| {
            let empties: Vec<(&String, &config::Arg)> = _field
                .args
                .iter()
                .filter(|(k, _)| !args.clone().any(|(k1, _)| k1.eq(*k)))
                .collect();

            if empties.len().gt(&0) {
                return Valid::fail(format!(
                    "no argument {} found",
                    empties
                        .iter()
                        .map(|(k, _)| format!("'{}'", k))
                        .collect::<Vec<String>>()
                        .join(", ")
                ))
                .trace(field_name.as_str());
            }

            if let Some(http) = _field.http.clone() {
                compile_http(config, field, &http).and_then(|expr| {
                    let http = Http::try_from(expr).unwrap();

                    Valid::succeed(
                        http.req_template
                            .clone()
                            .root_url(replace_url(&http.req_template.root_url, &args)),
                    )
                    .map(|req_template| {
                        req_template.clone().query(
                            req_template
                                .clone()
                                .query
                                .iter()
                                .map(replace_mustache(&args))
                                .collect(),
                        )
                    })
                    .map(|req_template| {
                        req_template.clone().headers(
                            req_template
                                .headers
                                .iter()
                                .map(replace_mustache(&args))
                                .collect(),
                        )
                    })
                    .map(|req_template| {
                        Expression::IO(IO::Http {
                            req_template,
                            dl_id: http.dl_id,
                            group_by: http.group_by,
                        })
                    })
                })
            } else if let Some(graphql) = _field.graphql.clone() {
                compile_graphql(config, operation_type, &graphql).and_then(|expr| {
                    let graphql = GraphQLEndpoint::try_from(expr).unwrap();

                    Valid::succeed(
                        graphql.req_template.clone().headers(
                            graphql
                                .req_template
                                .headers
                                .iter()
                                .map(replace_mustache(&args))
                                .collect(),
                        ),
                    )
                    .map(|req_template| {
                        if req_template.operation_arguments.is_some() {
                            let operation_arguments = req_template
                                .clone()
                                .operation_arguments
                                .unwrap()
                                .iter()
                                .map(replace_mustache(&args))
                                .collect();

                            req_template.operation_arguments(Some(operation_arguments))
                        } else {
                            req_template
                        }
                    })
                    .and_then(|req_template| {
                        Valid::succeed(Expression::IO(IO::GraphQLEndpoint {
                            req_template,
                            field_name: graphql.field_name,
                            batch: graphql.batch,
                            dl_id: graphql.dl_id,
                        }))
                    })
                })
            } else if let Some(grpc) = _field.grpc.clone() {
                // todo!("needs to be implemented");
                let inputs: CompileGrpc<'_> = CompileGrpc {
                    config,
                    operation_type,
                    field,
                    grpc: &grpc,
                    validate_with_schema: false,
                };
                compile_grpc(inputs).and_then(|expr| {
                    let grpc = Grpc::try_from(expr).unwrap();

                    Valid::succeed(
                        grpc.req_template
                            .clone()
                            .url(replace_url(&grpc.req_template.url, &args)),
                    )
                    .map(|req_template| {
                        req_template.clone().headers(
                            req_template
                                .headers
                                .iter()
                                .map(replace_mustache(&args))
                                .collect(),
                        )
                    })
                    .map(|req_template| {
                        if let Some(body) = req_template.clone().body {
                            req_template.clone().body(Some(replace_url(&body, &args)))
                        } else {
                            req_template
                        }
                    })
                    .map(|req_template| {
                        Expression::IO(IO::Grpc {
                            req_template,
                            group_by: grpc.group_by,
                            dl_id: grpc.dl_id,
                        })
                    })
                })
            } else {
                return Valid::fail(format!("{} field has no resolver", field_name));
            }
        })
}

fn replace_url(url: &Mustache, args: &Iter<'_, String, String>) -> Mustache {
    url.get_segments()
        .iter()
        .map(|segment| match segment {
            Segment::Literal(literal) => Segment::Literal(literal.clone()),
            Segment::Expression(expression) => {
                if expression[0] == "args" {
                    let value = find_value(args, &expression[1]).unwrap();
                    let item = Mustache::parse(value).unwrap();

                    let expression = item.get_segments().first().unwrap().to_owned().to_owned();

                    expression
                } else {
                    Segment::Expression(expression.clone())
                }
            }
        })
        .collect::<Vec<Segment>>()
        .into()
}

fn replace_mustache<'a, T: Clone>(
    args: &'a Iter<'a, String, String>,
) -> impl Fn(&(T, Mustache)) -> (T, Mustache) + 'a {
    |(key, value)| {
        let value: Mustache = value
            .expression_segments()
            .iter()
            .map(|expression| {
                if expression[0] == "args" {
                    let value = find_value(args, &expression[1]).unwrap();
                    let item = Mustache::parse(value).unwrap();

                    let expression = item.get_segments().first().unwrap().to_owned().to_owned();

                    expression
                } else {
                    Segment::Expression(expression.to_owned().to_owned())
                }
            })
            .collect::<Vec<Segment>>()
            .into();

        (key.clone().to_owned(), value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_http_fail() {
        let expr = Expression::Literal("test".into());

        let http = Http::try_from(expr);

        assert!(http.is_err());
    }

    #[test]
    fn try_from_graphql_fail() {
        let expr = Expression::Literal("test".into());

        let graphql = GraphQLEndpoint::try_from(expr);

        assert!(graphql.is_err());
    }

    #[test]
    fn try_from_grpc_fail() {
        let expr = Expression::Literal("test".into());

        let grpc = Grpc::try_from(expr);

        assert!(grpc.is_err());
    }
}
