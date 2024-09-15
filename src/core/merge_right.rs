use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use crate::core::valid::{Valid, ValidationError, Validator};
use serde_yaml::Value;

pub trait MergeRight {
    fn merge_right(self, other: Self) -> Valid<Self, String>;
}

impl<A: MergeRight> MergeRight for Option<A> {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        match (self, other) {
            (Some(this), Some(that)) => this.merge_right(that).map(Some),
            (None, Some(that)) => Valid::succeed(Some(that)),
            (Some(this), None) => Valid::succeed(Some(this)),
            (None, None) => Valid::succeed(None),
        }
    }
}

impl<A: MergeRight + Default> MergeRight for Arc<A> {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        let l = Arc::try_unwrap(self).unwrap_or_else(|arc| (*arc).clone());
        let r = Arc::try_unwrap(other).unwrap_or_else(|arc| (*arc).clone());
        l.merge_right(r).map(Arc::new)
    }
}

impl<A> MergeRight for Vec<A> {
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        self.extend(other);
        Valid::succeed(self)
    }
}

impl<K, V> MergeRight for BTreeMap<K, V>
where
    K: Ord,
    V: Clone + MergeRight,
{
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        let mut errors = ValidationError::empty();

        for (other_key, other_value) in other {
            if let Some(self_value) = self.remove(&other_key) {
                match self_value.merge_right(other_value).to_result() {
                    Ok(merged_value) => {
                        self.insert(other_key, merged_value);
                    }
                    Err(err) => {
                        errors = errors.combine(err);
                    }
                }
            } else {
                self.insert(other_key, other_value);
            }
        }

        if errors.is_empty() {
            Valid::succeed(self)
        } else {
            Valid::from_validation_err(errors)
        }
    }
}

impl<V> MergeRight for BTreeSet<V>
where
    V: Ord,
{
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        self.extend(other);
        Valid::succeed(self)
    }
}

impl<V> MergeRight for HashSet<V>
where
    V: Eq + std::hash::Hash,
{
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        self.extend(other);
        Valid::succeed(self)
    }
}

impl<K, V> MergeRight for HashMap<K, V>
where
    K: Eq + std::hash::Hash + Clone + std::fmt::Display,
    V: MergeRight,
{
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        let mut errors = ValidationError::empty();

        for (key, other_value) in other {
            if let Some(self_value) = self.remove(&key) {
                match self_value.merge_right(other_value).to_result() {
                    Ok(merged_value) => {
                        self.insert(key, merged_value);
                    }
                    Err(err) => {
                        errors = errors.combine(err);
                    }
                }
            } else {
                self.insert(key, other_value);
            }
        }

        if errors.is_empty() {
            Valid::succeed(self)
        } else {
            Valid::from_validation_err(errors)
        }
    }
}

impl MergeRight for Value {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        match (self, other) {
            (Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_), other) => {
                Valid::succeed(other)
            }
            (Value::Sequence(mut lhs), other) => match other {
                Value::Sequence(rhs) => {
                    lhs.extend(rhs);
                    Valid::succeed(Value::Sequence(lhs))
                }
                other => {
                    lhs.push(other);
                    Valid::succeed(Value::Sequence(lhs))
                }
            },
            (Value::Mapping(mut lhs), other) => match other {
                Value::Mapping(rhs) => {
                    let mut errors = ValidationError::empty();
                    for (key, other_value) in rhs {
                        if let Some(lhs_value) = lhs.remove(&key) {
                            match lhs_value.merge_right(other_value).to_result() {
                                Ok(merged_value) => {
                                    lhs.insert(key, merged_value);
                                }
                                Err(err) => {
                                    errors = errors.combine(err);
                                }
                            }
                        } else {
                            lhs.insert(key, other_value);
                        }
                    }

                    if errors.is_empty() {
                        Valid::succeed(Value::Mapping(lhs))
                    } else {
                        Valid::from_validation_err(errors)
                    }
                }
                Value::Sequence(mut rhs) => {
                    rhs.push(Value::Mapping(lhs));
                    Valid::succeed(Value::Sequence(rhs))
                }
                other => Valid::succeed(other),
            },
            (Value::Tagged(mut lhs), other) => match other {
                Value::Tagged(rhs) => {
                    if lhs.tag == rhs.tag {
                        lhs.value = lhs.value.merge_right(rhs.value)?;
                        Valid::succeed(Value::Tagged(lhs))
                    } else {
                        Valid::succeed(Value::Tagged(rhs))
                    }
                }
                other => Valid::succeed(other),
            },
        }
    }
}

impl MergeRight for TypeDefinition {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        let mut errors = Vec::new();

        // Check for type kind compatibility
        if self.kind != other.kind {
            errors.push(format!(
                "Type kind conflict for '{}': {:?} vs {:?}",
                self.name, self.kind, other.kind
            ));
        }

        // Merge fields
        let merged_fields = self.fields.merge_right(other.fields);
        if let Valid::Failure(err) = &merged_fields {
            errors.push(err.clone());
        }

        // Merge directives
        let merged_directives = self.directives.merge_right(other.directives);
        if let Valid::Failure(err) = &merged_directives {
            errors.push(err.clone());
        }

        if errors.is_empty() {
            Valid::success(TypeDefinition {
                name: self.name,
                kind: self.kind,
                fields: merged_fields.unwrap_or_default(),
                directives: merged_directives.unwrap_or_default(),
            })
        } else {
            Valid::failure(errors.join("; "))
        }
    }
}

impl MergeRight for FieldDefinition {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        let mut errors = Vec::new();

        // Check if field types are identical
        if self.field_type != other.field_type {
            errors.push(format!(
                "Field type conflict for '{}': {:?} vs {:?}",
                self.name, self.field_type, other.field_type
            ));
        }

        // Merge arguments
        let merged_args = self.arguments.merge_right(other.arguments);
        if let Valid::Failure(err) = &merged_args {
            errors.push(err.clone());
        }

        // Merge directives
        let merged_directives = self.directives.merge_right(other.directives);
        if let Valid::Failure(err) = &merged_directives {
            errors.push(err.clone());
        }

        if errors.is_empty() {
            Valid::success(FieldDefinition {
                name: self.name,
                field_type: self.field_type,
                arguments: merged_args.unwrap_or_default(),
                directives: merged_directives.unwrap_or_default(),
            })
        } else {
            Valid::failure(errors.join("; "))
        }
    }
}

impl MergeRight for Directive {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        if self.name != other.name {
            return Valid::failure(format!(
                "Directive name conflict: '{}' vs '{}'",
                self.name, other.name
            ));
        }

        // Merge arguments
        let merged_arguments = self.arguments.merge_right(other.arguments);

        if let Valid::Success(arguments) = merged_arguments {
            Valid::success(Directive { name: self.name, arguments })
        } else {
            Valid::failure(format!(
                "Directive '{}' argument conflict: {}",
                self.name,
                merged_arguments.unwrap_failure()
            ))
        }
    }
}
