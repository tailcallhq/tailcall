use std::collections::{HashMap, HashSet};

use tailcall_valid::{Valid, ValidationError, Validator};

use crate::core::config::{Config, ConfigModule, GraphQL, GraphQLOperationType};
use crate::core::graphql::RequestTemplate;
use crate::core::helpers;
use crate::core::ir::model::{IO, IR};
use crate::core::ir::RelatedFields;

fn create_related_fields(
    config: &Config,
    type_name: &str,
    visited: &mut HashSet<String>,
) -> RelatedFields {
    let mut map = HashMap::new();
    if visited.contains(type_name) {
        return RelatedFields(map);
    }
    visited.insert(type_name.to_string());

    if let Some(type_) = config.find_type(type_name) {
        for (name, field) in &type_.fields {
            if !field.has_resolver() {
                if let Some(modify) = &field.modify {
                    if let Some(modified_name) = &modify.name {
                        map.insert(
                            modified_name.clone(),
                            (
                                name.clone(),
                                create_related_fields(config, field.type_of.name(), visited),
                            ),
                        );
                    }
                } else {
                    map.insert(
                        name.clone(),
                        (
                            name.clone(),
                            create_related_fields(config, field.type_of.name(), visited),
                        ),
                    );
                }
            }
        }
    } else if let Some(union_) = config.find_union(type_name) {
        for type_name in &union_.types {
            map.extend(create_related_fields(config, type_name, visited).0);
        }
    };

    RelatedFields(map)
}

pub fn compile_graphql(
    config: &ConfigModule,
    operation_type: &GraphQLOperationType,
    type_name: &str,
    graphql: &GraphQL,
) -> Valid<IR, String> {
    let args = graphql.args.as_ref();
    Valid::succeed(graphql.url.as_str())
        .zip(helpers::headers::to_mustache_headers(&graphql.headers))
        .and_then(|(base_url, headers)| {
            Valid::from(
                RequestTemplate::new(
                    base_url.to_owned(),
                    operation_type,
                    &graphql.name,
                    args,
                    headers,
                    create_related_fields(config, type_name, &mut HashSet::new()),
                )
                .map_err(|e| ValidationError::new(e.to_string())),
            )
        })
        .map(|req_template| {
            let field_name = graphql.name.clone();
            let batch = graphql.batch;
            let dedupe = graphql.dedupe.unwrap_or_default();
            IR::IO(IO::GraphQL { req_template, field_name, batch, dl_id: None, dedupe })
        })
}
