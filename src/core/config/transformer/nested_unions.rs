use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::core::config::{Config, Union};
use crate::core::transform::Transform;
use crate::core::valid::{Valid, Validator};

/// Transforms unions by replacing each nested union in union definition
/// recursively by their actual types
#[derive(Default)]
pub struct NestedUnions;

impl Transform for NestedUnions {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Config) -> Valid<Config, String> {
        let visitor = Visitor { unions: &config.unions };

        visitor.visit().map(|unions| {
            config.unions = unions;
            config
        })
    }
}

struct Visitor<'cfg> {
    unions: &'cfg BTreeMap<String, Union>,
}

impl<'cfg> Visitor<'cfg> {
    fn visit(self) -> Valid<BTreeMap<String, Union>, String> {
        let mut result = BTreeMap::new();

        Valid::from_iter(self.unions.iter(), |(union_name, union_)| {
            let mut union_types = BTreeSet::new();

            self.walk_union(union_, &mut union_types, &mut HashSet::new())
                .trace(union_name)
                .map(|_| {
                    let new_union = Union { types: union_types, ..union_.clone() };

                    result.insert(union_name.clone(), new_union);
                })
        })
        .map(|_| result)
    }

    fn walk_union(
        &'cfg self,
        union_: &'cfg Union,
        union_types: &mut BTreeSet<String>,
        seen: &mut HashSet<&'cfg String>,
    ) -> Valid<(), String> {
        Valid::from_iter(union_.types.iter(), |type_name| {
            if let Some(union_) = self.unions.get(type_name) {
                if seen.contains(type_name) {
                    return Valid::fail(format!("Recursive type {type_name}"));
                }

                seen.insert(type_name);
                self.walk_union(union_, union_types, seen)
            } else {
                union_types.insert(type_name.clone());
                Valid::succeed(())
            }
        })
        .unit()
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::NestedUnions;
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn test_nested_unions() {
        let config =
            std::fs::read_to_string(tailcall_fixtures::configs::YAML_NESTED_UNIONS).unwrap();
        let config = Config::from_yaml(&config).unwrap();
        let config = NestedUnions.transform(config).to_result().unwrap();

        assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_nested_unions_recursive() {
        let config =
            std::fs::read_to_string(tailcall_fixtures::configs::YAML_NESTED_UNIONS_RECURSIVE)
                .unwrap();
        let config = Config::from_yaml(&config).unwrap();
        let error = NestedUnions.transform(config).to_result().unwrap_err();

        assert_snapshot!(error);
    }
}
