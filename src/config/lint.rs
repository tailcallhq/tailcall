use std::borrow::BorrowMut;
use std::collections::HashMap;

use async_graphql::parser::types::ServiceDocument;
use async_graphql::Name;
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

use crate::macros::MergeRight;
use crate::merge_right::MergeRight;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema, MergeRight)]
pub enum TextCase {
    Camel,
    Pascal,
    Snake,
    ScreamingSnake,
}

/// The @lint directive allows you to configure linting.
#[derive(
    Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema, MergeRight,
)]
pub struct Lint {
    ///
    /// To autoFix the lint.
    /// Example Usage lint:{autoFix:true}
    #[serde(rename = "autoFix")]
    pub auto_fix: Option<bool>,
    ///
    ///
    /// This enum is provided with appropriate TextCase.
    /// Example Usage: lint:{enum:Pascal}
    #[serde(rename = "enum")]
    pub enum_lint: Option<TextCase>,
    ///
    ///
    /// This enumValue is provided with appropriate TextCase.
    /// Example Usage: lint:{enumValue:ScreamingSnake}
    #[serde(rename = "enumValue")]
    pub enum_value_lint: Option<TextCase>,
    ///
    ///
    /// This field is provided with appropriate TextCase.
    /// Example Usage: lint:{field:Camel}
    #[serde(rename = "field")]
    pub field_lint: Option<TextCase>,
    ///
    ///
    /// This type is provided with appropriate TextCase.
    /// Example Usage: lint:{type:Pascal}
    #[serde(rename = "type")]
    pub type_lint: Option<TextCase>,
}

impl Lint {
    pub fn lint(doc: &mut ServiceDocument) {
        let mut auto_fix = false;
        let mut lint_config: HashMap<String, Case> = HashMap::new();

        // extract lint config
        for definition in doc.definitions.iter() {
            match definition {
                async_graphql::parser::types::TypeSystemDefinition::Schema(schema_) => {
                    for directive in schema_.node.directives.iter() {
                        if directive.node.name.node == "server" {
                            for argument in directive.node.arguments.clone() {
                                if argument.0.node == "lint" {
                                    match argument.1.node {
                                        async_graphql::Value::Null => {}
                                        async_graphql::Value::Number(_) => {}
                                        async_graphql::Value::String(_) => {}
                                        async_graphql::Value::Boolean(_) => {}
                                        async_graphql::Value::Binary(_) => {}
                                        async_graphql::Value::Enum(_) => {}
                                        async_graphql::Value::List(_) => {}
                                        async_graphql::Value::Object(lint_config_) => {
                                            fn case_mapper(value: &str) -> Case {
                                                match value {
                                                    "Pascal" => Case::Pascal,
                                                    "Camel" => Case::Camel,
                                                    "Snake" => Case::Snake,
                                                    "ScreamingSnake" => Case::ScreamingSnake,
                                                    _ => Case::Pascal,
                                                }
                                            }
                                            for config in lint_config_ {
                                                println!("{:?}", config.1.to_string());

                                                if config.0 == "autoFix" {
                                                    // for true case
                                                    if config.1.to_string() == "true" {
                                                        auto_fix = true;
                                                    };
                                                    continue;
                                                }

                                                lint_config
                                                    .entry(config.0.to_string())
                                                    .or_insert(case_mapper(&config.1.to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                async_graphql::parser::types::TypeSystemDefinition::Type(_) => {}
                async_graphql::parser::types::TypeSystemDefinition::Directive(_) => {}
            }
        }

        let mut map1: HashMap<String, String> = HashMap::new();
        for definition in doc.definitions.iter_mut() {
            match definition {
                async_graphql::parser::types::TypeSystemDefinition::Schema(_) => {}
                async_graphql::parser::types::TypeSystemDefinition::Type(type_) => {
                    let type_name = type_.node.name.node.to_string();
                    match type_.node.kind.borrow_mut() {
                        async_graphql::parser::types::TypeKind::Scalar => {
                            map1.entry(type_name.clone())
                                .or_insert("scalar".to_string());
                            if auto_fix && lint_config.contains_key("scalar") {
                                type_.node.name.node = Name::new(
                                    type_name
                                        .to_case(Case::Lower)
                                        .to_case(lint_config["scalar"]),
                                );
                            }
                        }
                        async_graphql::parser::types::TypeKind::Object(object_) => {
                            map1.entry(type_name.clone()).or_insert("type".to_string());
                            if auto_fix && lint_config.contains_key("type") && type_name != "Query"
                            {
                                type_.node.name.node = Name::new(
                                    type_name.to_case(Case::Lower).to_case(lint_config["type"]),
                                );
                            }

                            for field in object_.fields.iter_mut() {
                                if auto_fix && lint_config.contains_key("field") {
                                    field.node.name.node = Name::new(
                                        &field
                                            .node
                                            .name
                                            .node
                                            .to_case(Case::Lower)
                                            .to_case(lint_config["field"]),
                                    );
                                }
                            }
                        }
                        async_graphql::parser::types::TypeKind::Interface(interface) => {
                            map1.entry(type_name.clone())
                                .or_insert("interface".to_string());
                            if auto_fix && lint_config.contains_key("interface") {
                                type_.node.name.node = Name::new(
                                    type_name
                                        .to_case(Case::Lower)
                                        .to_case(lint_config["interface"]),
                                );
                            }

                            // type fields
                            for field in interface.fields.iter_mut() {
                                let field_name = field.node.name.node.to_case(Case::Lower);
                                if auto_fix && lint_config.contains_key("field") {
                                    field.node.name.node =
                                        Name::new(field_name.to_case(lint_config["field"]));
                                }
                            }
                        }
                        async_graphql::parser::types::TypeKind::Union(_) => {
                            map1.entry(type_name.clone()).or_insert("union".to_string());
                            if auto_fix && lint_config.contains_key("union") {
                                type_.node.name.node = Name::new(
                                    type_name.to_case(Case::Lower).to_case(lint_config["union"]),
                                );
                            }
                        }
                        async_graphql::parser::types::TypeKind::Enum(enum_) => {
                            map1.entry(type_name.clone()).or_insert("enum".to_string());
                            if auto_fix && lint_config.contains_key("enum") {
                                type_.node.name.node = Name::new(
                                    type_name.to_case(Case::Lower).to_case(lint_config["enum"]),
                                );
                            }

                            for value in enum_.values.iter_mut() {
                                let enum_value = value.node.value.node.to_case(Case::Lower);
                                if auto_fix && lint_config.contains_key("enum_value") {
                                    value.node.value.node =
                                        Name::new(enum_value.to_case(lint_config["enum_value"]));
                                }
                            }
                        }
                        async_graphql::parser::types::TypeKind::InputObject(input_) => {
                            map1.entry(type_name.clone()).or_insert("input".to_string());
                            // if auto_fix && lint_config.contains_key("input") {
                            //     type_.node.name.node = Name::new(
                            //         type_name.to_case(Case::Lower).to_case(lint_config["input"]),
                            //     );
                            // }

                            for field in input_.fields.iter_mut() {
                                if auto_fix && lint_config.contains_key("field") {
                                    field.node.name.node = Name::new(
                                        &field
                                            .node
                                            .name
                                            .node
                                            .to_case(Case::Lower)
                                            .to_case(lint_config["field"]),
                                    );
                                }
                            }
                        }
                    }
                }
                async_graphql::parser::types::TypeSystemDefinition::Directive(_) => {}
            }
        }

        // check usage in other places

        for definition in doc.definitions.iter_mut() {
            match definition {
                async_graphql::parser::types::TypeSystemDefinition::Schema(_) => {}
                async_graphql::parser::types::TypeSystemDefinition::Type(type_) => {
                    match type_.node.kind.borrow_mut() {
                        async_graphql::parser::types::TypeKind::Scalar => {}
                        async_graphql::parser::types::TypeKind::Object(object_) => {
                            for field in object_.fields.iter_mut() {
                                match field.node.ty.node.base.borrow_mut() {
                                    async_graphql::parser::types::BaseType::Named(base_type) => {
                                        let type_name = base_type.to_string();
                                        if map1.contains_key(&type_name) && auto_fix && lint_config.contains_key(&map1[&type_name]) {
                                            *base_type = Name::new(
                                                type_name
                                                    .to_case(Case::Lower)
                                                    .to_case(lint_config[&map1[&type_name]]),
                                            );
                                        }
                                    }
                                    async_graphql::parser::types::BaseType::List(
                                        base_type_list,
                                    ) => {
                                        let type_name = base_type_list.base.to_string();
                                        if map1.contains_key(&type_name) && auto_fix && lint_config.contains_key(&map1[&type_name]) {
                                            base_type_list.base =
                                                async_graphql::parser::types::BaseType::Named(
                                                    Name::new(
                                                        type_name.to_case(Case::Lower).to_case(
                                                            lint_config[&map1[&type_name]],
                                                        ),
                                                    ),
                                                );
                                        }
                                    }
                                }
                            }
                        }
                        async_graphql::parser::types::TypeKind::Interface(_) => {}
                        async_graphql::parser::types::TypeKind::Union(union_type) => {
                            for member in union_type.members.iter_mut() {
                                let type_name = member.node.to_string();
                                if map1.contains_key(&type_name) && auto_fix && lint_config.contains_key(&map1[&type_name]) {
                                    member.node = Name::new(
                                        type_name
                                            .to_case(Case::Lower)
                                            .to_case(lint_config[&map1[&type_name]]),
                                    );
                                }
                            }
                        }
                        async_graphql::parser::types::TypeKind::Enum(_) => {}
                        async_graphql::parser::types::TypeKind::InputObject(input_) => {
                            for field in input_.fields.iter_mut() {
                                match field.node.ty.node.base.borrow_mut() {
                                    async_graphql::parser::types::BaseType::Named(base_type) => {
                                        let type_name = base_type.to_string();
                                        if map1.contains_key(&type_name) && auto_fix && lint_config.contains_key(&map1[&type_name]) {
                                            *base_type = Name::new(
                                                type_name
                                                    .to_case(Case::Lower)
                                                    .to_case(lint_config[&map1[&type_name]]),
                                            );
                                        }
                                    }
                                    async_graphql::parser::types::BaseType::List(
                                        base_type_list,
                                    ) => {
                                        let type_name = base_type_list.base.to_string();
                                        if map1.contains_key(&type_name) && auto_fix && lint_config.contains_key(&map1[&type_name]) {
                                            base_type_list.base =
                                                async_graphql::parser::types::BaseType::Named(
                                                    Name::new(
                                                        type_name.to_case(Case::Lower).to_case(
                                                            lint_config[&map1[&type_name]],
                                                        ),
                                                    ),
                                                );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                async_graphql::parser::types::TypeSystemDefinition::Directive(_) => {}
            }
        }
    }
}
