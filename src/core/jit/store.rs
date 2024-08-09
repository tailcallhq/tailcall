use std::collections::HashMap;

use crate::core::jit::model::FieldId;

/// Path to the data in the store with info
/// to resolve nested multiple data
#[derive(Debug, Clone)]
pub struct DataPath(Vec<usize>);

impl Default for DataPath {
    fn default() -> Self {
        Self::new()
    }
}

impl DataPath {
    /// Create default DataPath that resolved to single value
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create new DataPath with specified additional entry
    pub fn with_index(mut self, index: usize) -> Self {
        self.0.push(index);

        Self(self.0)
    }

    /// Iterator over indexes only for multiple paths.
    /// Helpful when collecting the data after it has been previously
    /// resolved
    pub fn as_slice(&self) -> &[usize] {
        &self.0
    }
}

#[derive(Debug)]
pub struct Store<Data> {
    data: HashMap<usize, Data>,
}

impl<Data> Default for Store<Data> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Data> Store<Data> {
    pub fn new() -> Self {
        Store { data: HashMap::new() }
    }

    pub fn set_data(&mut self, field_id: FieldId, data: Data) {
        self.data.insert(field_id.as_usize(), data);
    }

    pub fn set(&mut self, field_id: &FieldId, data: Data) {
        self.data.insert(field_id.as_usize(), data);
    }

    pub fn get(&self, field_id: &FieldId) -> Option<&Data> {
        self.data.get(&field_id.as_usize())
    }
}
