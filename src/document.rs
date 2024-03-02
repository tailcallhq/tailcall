use async_graphql::parser::types::*;
use async_graphql::{Pos, Positioned};
use async_graphql_value::{ConstValue, Name};

fn pos<A>(a: A) -> Positioned<A> {
    Positioned::new(a, Pos::default())
}

fn print_schema(schema: &SchemaDefinition) -> String {
    let directives = schema
        .directives
        .iter()
        .map(|d| print_directive(&const_directive_to_sdl(&d.node)))
        .collect::<Vec<String>>()
        .join(" ");

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
    if directives.is_empty() {
        format!("schema {{\n{}{}{}}}\n", query, mutation, subscription)
    } else {
        format!(
            "schema {} {{\n{}{}{}}}\n",
            directives, query, mutation, subscription
        )
    }
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
            format!(
                "input {} {{\n{}\n}}\n",
                type_def.name.node,
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
            let directives = if !type_def.directives.is_empty() {
                format!(
                    "{} ",
                    type_def
                        .directives
                        .iter()
                        .map(|d| print_directive(&const_directive_to_sdl(&d.node)))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            } else {
                String::new()
            };

            format!(
                "type {} {}{}{{\n{}\n}}\n",
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
        TypeKind::Enum(en) => format!(
            "enum {} {{\n{}\n}}\n",
            type_def.name.node,
            en.values
                .iter()
                .map(|v| format!("  {}", v.node.value))
                .collect::<Vec<String>>()
                .join("\n")
        ),
        // Handle other type kinds...
    }
}

fn print_field(field: &async_graphql::parser::types::FieldDefinition) -> String {
    let directives: Vec<String> = field
        .directives
        .iter()
        .map(|d| print_directive(&const_directive_to_sdl(&d.node)))
        .collect();
    let directives_str = if !directives.is_empty() {
        format!(" {}", directives.join(" "))
    } else {
        String::new()
    };

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
        "  {}{}: {}{}",
        field.name.node, args_str, field.ty.node, directives_str
    );
    format!("{}{}", doc, node)
}

fn print_input_value(field: &async_graphql::parser::types::InputValueDefinition) -> String {
    let directives: Vec<String> = field
        .directives
        .iter()
        .map(|d| print_directive(&const_directive_to_sdl(&d.node)))
        .collect();

    let directives_str = if !directives.is_empty() {
        format!(" {}", directives.join(" "))
    } else {
        String::new()
    };

    format!("  {}: {}{}", field.name.node, field.ty.node, directives_str)
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
