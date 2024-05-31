use async_graphql::parser::types::*;
use async_graphql::{Pos, Positioned};
use async_graphql_value::{ConstValue, Name};

fn print_directives(directives: &[Positioned<ConstDirective>]) -> String {
    if directives.is_empty() {
        return String::new();
    }
    directives
        .iter()
        .map(|d| print_directive(&const_directive_to_sdl(&d.node)))
        .collect::<Vec<String>>()
        .join(" ")
        + " "
}

fn pos<A>(a: A) -> Positioned<A> {
    Positioned::new(a, Pos::default())
}

fn print_schema(schema: &SchemaDefinition) -> String {
    let directives = print_directives(&schema.directives);

    let query = schema
        .query
        .as_ref()
        .map_or(String::new(), |q| format!("  query: {}\n", q.node));
    let mutation = schema
        .mutation
        .as_ref()
        .map_or(String::new(), |m| format!("  mutation: {}\n", m.node));
    let subscription = schema
        .subscription
        .as_ref()
        .map_or(String::new(), |s| format!("  subscription: {}\n", s.node));
    if mutation.is_empty() && query.is_empty() {
        return String::new();
    }
    format!(
        "schema {}{{\n{}{}{}}}\n",
        directives, query, mutation, subscription
    )
}
fn const_directive_to_sdl(directive: &ConstDirective) -> DirectiveDefinition {
    DirectiveDefinition {
        description: None,
        name: pos(Name::new(directive.name.node.clone())),
        arguments: directive
            .arguments
            .iter()
            .filter_map(|(name, value)| {
                if value.node.clone() != ConstValue::Null {
                    Some(pos(InputValueDefinition {
                        description: None,
                        name: pos(Name::new(name.node.clone())),
                        ty: pos(Type {
                            nullable: true,
                            base: async_graphql::parser::types::BaseType::Named(Name::new(
                                value.node.clone().to_string(),
                            )),
                        }),
                        default_value: Some(pos(ConstValue::String(
                            value.node.clone().to_string(),
                        ))),
                        directives: Vec::new(),
                    }))
                } else {
                    None
                }
            })
            .collect(),
        is_repeatable: true,
        locations: vec![],
    }
}
fn print_type_def(type_def: &TypeDefinition) -> String {
    match &type_def.kind {
        TypeKind::Scalar => {
            format!("scalar {}\n", type_def.name.node)
        }
        TypeKind::Union(union) => {
            format!(
                "union {} = {}\n",
                type_def.name.node,
                union
                    .members
                    .iter()
                    .map(|name| name.node.clone())
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        }
        TypeKind::InputObject(input) => {
            let directives = print_directives(&type_def.directives);
            let doc = type_def.description.as_ref().map_or(String::new(), |d| {
                format!(r#"  """{}  {}{}  """{}"#, "\n", d.node, "\n", "\n")
            });
            format!(
                "{}input {} {}{{\n{}\n}}\n",
                doc,
                type_def.name.node,
                directives,
                input
                    .fields
                    .iter()
                    .map(|f| print_input_value(&f.node))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        }
        TypeKind::Interface(interface) => {
            let implements = if !interface.implements.is_empty() {
                format!(
                    "implements {} ",
                    interface
                        .implements
                        .iter()
                        .map(|name| name.node.clone())
                        .collect::<Vec<_>>()
                        .join(" & ")
                )
            } else {
                String::new()
            };
            format!(
                "interface {} {}{{\n{}\n}}\n",
                type_def.name.node,
                implements,
                interface
                    .fields
                    .iter()
                    .map(|f| print_field(&f.node))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        }
        TypeKind::Object(object) => {
            let implements = if !object.implements.is_empty() {
                format!(
                    "implements {} ",
                    object
                        .implements
                        .iter()
                        .map(|name| name.node.clone())
                        .collect::<Vec<_>>()
                        .join(" & ")
                )
            } else {
                String::new()
            };
            let directives = print_directives(&type_def.directives);
            let doc = type_def.description.as_ref().map_or(String::new(), |d| {
                format!(r#"  """{}  {}{}  """{}"#, "\n", d.node, "\n", "\n")
            });
            format!(
                "{}type {} {}{}{{\n{}\n}}\n",
                doc,
                type_def.name.node,
                implements,
                directives,
                object
                    .fields
                    .iter()
                    .map(|f| print_field(&f.node))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        }
        TypeKind::Enum(en) => {
            let directives = print_directives(&type_def.directives);
            let enum_def = format!(
                "enum {} {}{{\n{}\n}}\n",
                type_def.name.node,
                directives,
                en.values
                    .iter()
                    .map(|v| format!("  {}", v.node.value))
                    .collect::<Vec<String>>()
                    .join("\n")
            );

            if let Some(desc) = &type_def.description {
                let ds = format!("\"\"\"\n{}\n\"\"\"\n", desc.node.as_str());
                ds + &enum_def
            } else {
                enum_def
            }
        } // Handle other type kinds...
    }
}

fn print_field(field: &async_graphql::parser::types::FieldDefinition) -> String {
    let directives = print_directives(&field.directives);
    let args_str = if !field.arguments.is_empty() {
        let args = field
            .arguments
            .iter()
            .map(|arg| {
                let nullable = if arg.node.ty.node.nullable { "" } else { "!" };
                format!("{}: {}{}", arg.node.name, arg.node.ty.node.base, nullable)
            })
            .collect::<Vec<String>>()
            .join(", ");
        format!("({})", args)
    } else {
        String::new()
    };
    let doc = field.description.as_ref().map_or(String::new(), |d| {
        format!(r#"  """{}  {}{}  """{}"#, "\n", d.node, "\n", "\n")
    });
    let node = &format!(
        "  {}{}: {} {}",
        field.name.node, args_str, field.ty.node, directives
    );
    doc + node.trim_end()
}

fn print_input_value(field: &async_graphql::parser::types::InputValueDefinition) -> String {
    let directives_str = print_directives(&field.directives);
    let doc = field.description.as_ref().map_or(String::new(), |d| {
        format!(r#"  """{}  {}{}  """{}"#, "\n", d.node, "\n", "\n")
    });
    format!(
        "{}  {}: {}{}",
        doc, field.name.node, field.ty.node, directives_str
    )
}
fn print_directive(directive: &DirectiveDefinition) -> String {
    let args = directive
        .arguments
        .iter()
        .map(|arg| format!("{}: {}", arg.node.name.node, arg.node.ty.node))
        .collect::<Vec<String>>()
        .join(", ");

    if args.is_empty() {
        format!("@{}", directive.name.node)
    } else {
        format!("@{}({})", directive.name.node, args)
    }
}
pub fn print(sd: ServiceDocument) -> String {
    // Separate the definitions by type
    let definitions_len = sd.definitions.len();
    let mut schemas = Vec::with_capacity(definitions_len);
    let mut scalars = Vec::with_capacity(definitions_len);
    let mut interfaces = Vec::with_capacity(definitions_len);
    let mut objects = Vec::with_capacity(definitions_len);
    let mut enums = Vec::with_capacity(definitions_len);
    let mut unions = Vec::with_capacity(definitions_len);
    let mut inputs = Vec::with_capacity(definitions_len);

    for def in sd.definitions.iter() {
        match def {
            TypeSystemDefinition::Schema(schema) => schemas.push(print_schema(&schema.node)),
            TypeSystemDefinition::Type(type_def) => match &type_def.node.kind {
                TypeKind::Scalar => scalars.push(print_type_def(&type_def.node)),
                TypeKind::Interface(_) => interfaces.push(print_type_def(&type_def.node)),
                TypeKind::Enum(_) => enums.push(print_type_def(&type_def.node)),
                TypeKind::Object(_) => objects.push(print_type_def(&type_def.node)),
                TypeKind::Union(_) => unions.push(print_type_def(&type_def.node)),
                TypeKind::InputObject(_) => inputs.push(print_type_def(&type_def.node)),
            },
            TypeSystemDefinition::Directive(_) => todo!("Directives are not supported yet"),
        }
    }

    // Concatenate the definitions in the desired order
    let sdl_string = schemas
        .into_iter()
        .chain(scalars)
        .chain(inputs)
        .chain(interfaces)
        .chain(unions)
        .chain(enums)
        .chain(objects)
        // Chain other types as needed...
        .collect::<Vec<String>>()
        .join("\n");

    sdl_string.trim_end_matches('\n').to_string()
}
