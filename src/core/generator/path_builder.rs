pub struct PathBuilder {
    base_path: Vec<i32>,
}

impl PathBuilder {
    pub fn new(base_path: &[i32]) -> Self {
        Self { base_path: base_path.to_vec() }
    }

    pub fn extend(&self, extension: &[i32]) -> Vec<i32> {
        [self.base_path.as_slice(), extension].concat()
    }
}
