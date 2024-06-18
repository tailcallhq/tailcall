use std::collections::HashSet;

/// a utility module that facilitates the comparison of two types. It determines
/// whether two types are comparable and checks if they belong to certain
/// categories.
pub struct ComparableTypes {
    input_types: HashSet<String>,
    union_types: HashSet<String>,
}

impl ComparableTypes {
    pub fn new(input_types: HashSet<String>, union_types: HashSet<String>) -> Self {
        Self { input_types, union_types }
    }

    /// checks if the given type is an object type.
    fn is_object_type(&self, type_: &str) -> bool {
        !self.is_input_type(type_) && !self.is_union_type(type_)
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

    /// returns the threshold required to calculate the similarity between two
    /// types.
    ///
    /// If both types are input types or union types, the threshold is set to
    /// 1.0, indicating that they must match completely. Otherwise, the
    /// provided `threshold` value is returned.
    pub fn get_threshold(&self, type_1: &str, type_2: &str, threshold: f32) -> f32 {
        if self.are_input_type(type_1, type_2) || self.are_union_type(type_1, type_2) {
            // if the type is input or union then they're similar only when they've exact
            // same fields.
            1.0
        } else {
            threshold
        }
    }

    /// determines whether two type names are comparable.
    ///
    /// types are comparable if they are both input types, both union types, or
    /// neither. input types can only be compared with input types, union
    /// types with union types, and object type with object type.
    pub fn comparable(&self, type_1: &str, type_2: &str) -> bool {
        if self.are_input_type(type_1, type_2) || self.are_union_type(type_1, type_2) {
            true
        } else {
            self.is_object_type(type_1) && self.is_object_type(type_2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_input_type() {
        let input_types: HashSet<String> = ["InputType1", "InputType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let union_types: HashSet<String> = ["UnionType1", "UnionType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let comparable_types = ComparableTypes::new(input_types, union_types);

        assert!(comparable_types.is_input_type("InputType1"));
        assert!(!comparable_types.is_input_type("UnionType1"));
    }

    #[test]
    fn test_is_union_type() {
        let input_types: HashSet<String> = ["InputType1", "InputType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let union_types: HashSet<String> = ["UnionType1", "UnionType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let comparable_types = ComparableTypes::new(input_types, union_types);

        assert!(comparable_types.is_union_type("UnionType1"));
        assert!(!comparable_types.is_union_type("InputType1"));
    }

    #[test]
    fn test_are_input_type() {
        let input_types: HashSet<String> = ["InputType1", "InputType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let union_types: HashSet<String> = ["UnionType1", "UnionType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let comparable_types = ComparableTypes::new(input_types, union_types);

        assert!(comparable_types.are_input_type("InputType1", "InputType2"));
        assert!(!comparable_types.are_input_type("InputType1", "UnionType1"));
    }

    #[test]
    fn test_are_union_type() {
        let input_types: HashSet<String> = ["InputType1", "InputType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let union_types: HashSet<String> = ["UnionType1", "UnionType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let comparable_types = ComparableTypes::new(input_types, union_types);

        assert!(comparable_types.are_union_type("UnionType1", "UnionType2"));
        assert!(!comparable_types.are_union_type("UnionType1", "InputType1"));
    }

    #[test]
    fn test_get_threshold() {
        let input_types: HashSet<String> = ["InputType1", "InputType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let union_types: HashSet<String> = ["UnionType1", "UnionType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let comparable_types = ComparableTypes::new(input_types, union_types);

        assert_eq!(
            comparable_types.get_threshold("InputType1", "InputType2", 0.5),
            1.0
        );
        assert_eq!(
            comparable_types.get_threshold("UnionType1", "UnionType2", 0.5),
            1.0
        );
        assert_eq!(
            comparable_types.get_threshold("InputType1", "UnionType1", 0.5),
            0.5
        );
    }

    #[test]
    fn test_comparable() {
        let input_types: HashSet<String> = ["InputType1", "InputType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let union_types: HashSet<String> = ["UnionType1", "UnionType2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let comparable_types = ComparableTypes::new(input_types, union_types);

        assert!(comparable_types.comparable("InputType1", "InputType2"));
        assert!(comparable_types.comparable("UnionType1", "UnionType2"));
        assert!(comparable_types.comparable("ObjectType1", "ObjectType2"));

        assert!(!comparable_types.comparable("InputType1", "UnionType1"));
        assert!(!comparable_types.comparable("InputType1", "ObjectType1"));
        assert!(!comparable_types.comparable("ObjectType1", "UnionType1"));
    }
}
