use std::collections::HashSet;

use crate::core::config::Config;

/// a utility module that facilitates the comparison of two types. It provides
/// methods to determine whether two types are comparable and checks if they
/// belong to certain categories. Additionally, it identifies type names that
/// are eligible for the merging process.
pub struct MergeableTypes {
    input_types: HashSet<String>,
    union_types: HashSet<String>,
    output_types: HashSet<String>,
    interface_types: HashSet<String>,
    threshold: f32,
}

impl MergeableTypes {
    pub fn new(config: &Config, threshold: f32) -> Self {
        Self {
            input_types: config.input_types(),
            union_types: config.union_types(),
            output_types: config.output_types(),
            interface_types: config.interfaces_types_map().keys().cloned().collect(),
            threshold,
        }
    }

    // Iterator function to iterate over all string items
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.input_types
            .iter()
            .chain(self.union_types.iter())
            .chain(self.output_types.iter())
            .chain(self.interface_types.iter())
    }

    /// checks if the given type is an object type.
    fn is_output_type(&self, type_: &str) -> bool {
        self.output_types.contains(type_)
    }

    /// checks if the given type is an object type.
    fn is_interface_type(&self, type_: &str) -> bool {
        self.interface_types.contains(type_)
    }

    /// checks if the given type is an input type.
    fn is_input_type(&self, type_: &str) -> bool {
        self.input_types.contains(type_)
    }

    /// checks if the given type is a union type.
    fn is_union_type(&self, type_: &str) -> bool {
        self.union_types.contains(type_)
    }

    /// determines whether both type names represent input types.
    fn are_input_type(&self, type_1: &str, type_2: &str) -> bool {
        self.is_input_type(type_1) && self.is_input_type(type_2)
    }

    /// determines whether both type names represent union types.
    fn are_union_type(&self, type_1: &str, type_2: &str) -> bool {
        self.is_union_type(type_1) && self.is_union_type(type_2)
    }

    /// determines whether both type names represent output types.
    fn are_output_type(&self, type_1: &str, type_2: &str) -> bool {
        self.is_output_type(type_1) && self.is_output_type(type_2)
    }

    /// determines whether both type names represent output types.
    fn are_interface_type(&self, type_1: &str, type_2: &str) -> bool {
        self.is_interface_type(type_1) && self.is_interface_type(type_2)
    }

    /// returns the threshold required to calculate the similarity between two
    /// types.
    ///
    /// If both types are input types or union types and interface then the
    /// threshold is set to 1.0, indicating that they must match completely.
    /// Otherwise, the provided `threshold` value is returned.
    pub fn get_threshold(&self, type_1: &str, type_2: &str) -> f32 {
        if self.are_input_type(type_1, type_2)
            || self.are_union_type(type_1, type_2)
            || self.are_interface_type(type_1, type_2)
        {
            // if the type is input or union then they're similar only when they've exact
            // same fields.
            1.0
        } else {
            self.threshold
        }
    }

    /// determines whether two type names are mergeable.
    ///
    /// types are mergeable if they are both input types, both union types, both
    /// output types and interface types
    pub fn mergeable(&self, type_1: &str, type_2: &str) -> bool {
        self.are_input_type(type_1, type_2)
            || self.are_union_type(type_1, type_2)
            || self.are_output_type(type_1, type_2)
            || self.are_interface_type(type_1, type_2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Default for MergeableTypes {
        fn default() -> Self {
            Self {
                input_types: ["InputType1", "InputType2"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                union_types: ["UnionType1", "UnionType2"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                output_types: ["OutputType1", "OutputType2"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                interface_types: ["InterfaceType1", "InterfaceType2"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                threshold: 0.5,
            }
        }
    }

    #[test]
    fn test_is_input_type() {
        let comparable_types = MergeableTypes::default();

        assert!(comparable_types.is_input_type("InputType1"));
        assert!(!comparable_types.is_input_type("UnionType1"));
    }

    #[test]
    fn test_is_union_type() {
        let comparable_types = MergeableTypes::default();

        assert!(comparable_types.is_union_type("UnionType1"));
        assert!(!comparable_types.is_union_type("InputType1"));
    }

    #[test]
    fn test_is_output_type() {
        let comparable_types = MergeableTypes::default();

        assert!(comparable_types.is_output_type("OutputType1"));
        assert!(comparable_types.is_output_type("OutputType2"));
        assert!(!comparable_types.is_output_type("InputType1"));
    }
    #[test]
    fn test_is_interface_type() {
        let type_comparator = MergeableTypes::default();

        assert!(type_comparator.is_interface_type("InterfaceType1"));
        assert!(!type_comparator.is_interface_type("InputType1"));
    }

    #[test]
    fn test_are_interface_type() {
        let type_comparator = MergeableTypes::default();

        assert!(type_comparator.are_interface_type("InterfaceType1", "InterfaceType2"));
        assert!(!type_comparator.are_interface_type("InterfaceType1", "InputType1"));
    }

    #[test]
    fn test_are_input_type() {
        let comparable_types = MergeableTypes::default();

        assert!(comparable_types.are_input_type("InputType1", "InputType2"));
        assert!(!comparable_types.are_input_type("InputType1", "UnionType1"));
    }

    #[test]
    fn test_are_union_type() {
        let comparable_types = MergeableTypes::default();

        assert!(comparable_types.are_union_type("UnionType1", "UnionType2"));
        assert!(!comparable_types.are_union_type("UnionType1", "InputType1"));
    }

    #[test]
    fn test_are_output_type() {
        let comparable_types = MergeableTypes::default();

        assert!(comparable_types.are_output_type("OutputType1", "OutputType2"));
        assert!(!comparable_types.are_output_type("UnionType1", "InputType1"));
    }

    #[test]
    fn test_get_threshold() {
        let comparable_types = MergeableTypes::default();

        assert_eq!(
            comparable_types.get_threshold("InputType1", "InputType2"),
            1.0
        );
        assert_eq!(
            comparable_types.get_threshold("OutputType1", "OutputType2"),
            0.5
        );
        assert_eq!(
            comparable_types.get_threshold("UnionType1", "UnionType2"),
            1.0
        );
        assert_eq!(
            comparable_types.get_threshold("InterfaceType1", "InterfaceType1"),
            1.0
        );
        assert_eq!(
            comparable_types.get_threshold("InputType1", "UnionType1"),
            0.5
        );
    }

    #[test]
    fn test_comparable() {
        let comparable_types = MergeableTypes::default();

        assert!(comparable_types.mergeable("InputType1", "InputType2"));
        assert!(comparable_types.mergeable("UnionType1", "UnionType2"));
        assert!(comparable_types.mergeable("OutputType1", "OutputType2"));
        assert!(comparable_types.mergeable("InterfaceType1", "InterfaceType2"));

        assert!(!comparable_types.mergeable("InputType1", "UnionType1"));
        assert!(!comparable_types.mergeable("InputType1", "OutputType1"));
        assert!(!comparable_types.mergeable("OutputType1", "UnionType1"));
        assert!(!comparable_types.mergeable("InterfaceType1", "OutputType1"));
    }
}
