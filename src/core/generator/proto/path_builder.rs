use super::path_field::PathField;

pub struct PathBuilder {
    base_path: Vec<i32>,
}

impl PathBuilder {
    pub fn new(base_path: &[i32]) -> Self {
        Self { base_path: base_path.to_vec() }
    }

    pub fn extend(&self, field: PathField, index: i32) -> Vec<i32> {
        let mut extended_path = self.base_path.clone();
        extended_path.push(field.value());
        extended_path.push(index);
        extended_path
    }
}
