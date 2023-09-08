use std::collections::{HashMap, HashSet};

use super::{Blueprint, Definition};

pub fn compress(mut blueprint: Blueprint) -> Blueprint {
    let graph = build_dependency_graph(&blueprint);
    let mut referenced_types = identify_referenced_types(&graph, vec!["Query", "Mutation", "Subscription"]);
    referenced_types.insert("Query".to_string());
    referenced_types.insert("Mutation".to_string());
    referenced_types.insert("Subscription".to_string());
    referenced_types.insert("__Schema".to_string());
    referenced_types.insert("__Type".to_string());
    referenced_types.insert("__Field".to_string());
    referenced_types.insert("__InputValue".to_string());
    referenced_types.insert("__EnumValue".to_string());
    referenced_types.insert("__Directive".to_string());
    referenced_types.insert("__DirectiveLocation".to_string());

    let mut definitions = Vec::new();
    for def in blueprint.definitions.iter() {
        if referenced_types.contains(def.name()) {
            definitions.push(def.clone());
        }
    }

    blueprint.definitions = definitions;
    blueprint
}

fn build_dependency_graph(blueprint: &Blueprint) -> HashMap<&str, Vec<&str>> {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

    for def in &blueprint.definitions {
        let type_name = def.name();
        let mut dependencies: Vec<&str> = Vec::new();

        match def {
            Definition::ObjectTypeDefinition(def) => {
                dependencies.extend(def.fields.iter().map(|field| field.of_type.name()));
                for field in &def.fields {
                    dependencies.extend(field.args.iter().map(|arg| arg.of_type.name()));
                }
                dependencies.extend(def.implements.iter().map(|s| s.as_str()));
            }
            Definition::InterfaceTypeDefinition(def) => {
                dependencies.extend(def.fields.iter().map(|field| field.of_type.name()));
            }
            Definition::InputObjectTypeDefinition(def) => {
                dependencies.extend(def.fields.iter().map(|field| field.of_type.name()));
            }
            Definition::EnumTypeDefinition(def) => {
                dependencies.extend(def.enum_values.iter().map(|value| value.name.as_str()));
            }
            Definition::UnionTypeDefinition(def) => {
                dependencies.extend(def.types.iter().map(|s| s.as_str()));
            }
            Definition::ScalarTypeDefinition(sc) => {
                dependencies.push(sc.name.as_str());
            }
        }

        graph.insert(type_name, dependencies);
    }
    graph
}

// Function to perform DFS and identify all reachable types
fn identify_referenced_types(graph: &HashMap<&str, Vec<&str>>, root: Vec<&str>) -> HashSet<String> {
    let mut stack = root;
    let mut referenced_types = HashSet::new();

    while let Some(type_name) = stack.pop() {
        if referenced_types.insert(type_name.to_string()) {
            if let Some(dependencies) = graph.get(type_name) {
                for dependency in dependencies {
                    stack.push(dependency);
                }
            }
        }
    }

    referenced_types
}
