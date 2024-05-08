use async_graphql::parser::types::ExecutableDocument;

use crate::blueprint::Definition;
use crate::{
    blueprint::{Blueprint, Type},
    lambda::Expression,
};

struct Resolver {
    resolver: Option<Expression>,
    type_info: Type,
}

struct Field {
    name: String,
    selection: Selection,
    resolver: Resolver,
}

struct Selection {
    fields: Vec<Field>,
}

impl Selection {
    pub fn recurse(
        blueprint: &Blueprint,
        selection_set: &async_graphql::parser::types::SelectionSet,
        name: &str,
    ) -> anyhow::Result<Vec<Field>> {
        let mut fields = Vec::new();
        let def = blueprint.definitions.iter().find_map(|def| {
            if def.name() == name {
                Some(def)
            } else {
                None
            }
        });
        let out = if let Some(def) = def {
            match def {
                Definition::Object(def) => {
                    for selection in selection_set.items.iter() {
                        match &selection.node {
                            async_graphql::parser::types::Selection::Field(field) => {
                                let p = def.fields.iter().find_map(|field_def| {
                                    if field_def.name == field.node.name.node {
                                        Some(field_def)
                                    } else {
                                        None
                                    }
                                });
                                if let Some(p) = p {
                                    let node = Field {
                                        name: field.node.name.node.to_string(),
                                        resolver: Resolver {
                                            resolver: p.resolver.clone(),
                                            type_info: p.of_type.clone(),
                                        },
                                        selection: if !field
                                            .node
                                            .selection_set
                                            .node
                                            .items
                                            .is_empty()
                                        {
                                            Selection {
                                                fields: Selection::recurse(
                                                    blueprint,
                                                    &field.node.selection_set.node,
                                                    &p.of_type.name(),
                                                )?,
                                            }
                                        } else {
                                            Selection { fields: vec![] }
                                        },
                                    };
                                    fields.push(node);
                                }
                            }

                            _ => {}
                        }
                    }
                    Ok(fields)
                }
                _ => Err(anyhow::anyhow!("unsupported definition type")),
            }
        } else {
            Err(anyhow::anyhow!("definition not found"))
        };
        out
    }
}

pub struct QueryBlueprint {
    selection: Selection,
}

impl QueryBlueprint {
    pub fn new(document: ExecutableDocument, blueprint: &Blueprint) -> anyhow::Result<Self> {
        let mut fields = Vec::new();
        for (_, operation) in document.operations.iter() {
            let mut root_name = operation.node.ty.to_string();
            if root_name == "query" {
                root_name = "Query".to_string();
            }
            let selection_set = &operation.node.selection_set.node;
            fields = Selection::recurse(blueprint, selection_set, &root_name)?;
        }
        let selection = Selection { fields };
        Ok(Self { selection })
    }
}
