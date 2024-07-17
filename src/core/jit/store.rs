use std::collections::HashMap;

use crate::core::{ir::TypeName, jit::model::FieldId};

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
pub struct Store<A> {
    data: HashMap<usize, Data<A>>,
}

#[derive(Clone, Default)]
pub enum Data<A> {
    /// Represents that the value was computed only once for the associated
    /// field
    Single {
        value: A,
        type_name: Option<TypeName>,
    },
    /// Represents that the value was computed multiple times for the associated
    /// field. The order is guaranteed by the executor to be the same as the
    /// other of invocation and not the other of completion.
    Multiple(HashMap<usize, Data<A>>),
    /// Represents that the value is yet to be computed
    #[default]
    Pending,
}

impl<A> std::fmt::Debug for Data<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single { .. } => f.debug_tuple("Single").finish(),
            Self::Multiple(arg0) => f.debug_tuple("Multiple").field(&arg0.len()).finish(),
            Self::Pending => write!(f, "Pending"),
        }
    }
}

impl<A> Data<A> {
    pub fn single(value: A) -> Self {
        Self::Single { value, type_name: None }
    }

    pub fn map<B>(self, ab: impl Fn(A) -> B + Copy) -> Data<B> {
        match self {
            Data::Single { value, type_name } => Data::Single { value: ab(value), type_name },
            Data::Multiple(values) => Data::Multiple(
                values
                    .into_iter()
                    .map(|(index, e)| (index, e.map(ab)))
                    .collect(),
            ),
            Data::Pending => Data::Pending,
        }
    }
}

impl<A> Default for Store<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Store<A> {
    pub fn new() -> Self {
        Store { data: HashMap::new() }
    }

    pub fn set_data(&mut self, field_id: FieldId, data: Data<A>) {
        self.data.insert(field_id.as_usize(), data);
    }

    pub fn entry(&mut self, field_id: &FieldId, path: &DataPath) -> &mut Data<A> {
        let path = path.as_slice();
        let mut current_entry = self.data.entry(field_id.as_usize());

        for index in path {
            let entry = current_entry
                .and_modify(|e| match e {
                    Data::Multiple(_) => {}
                    // force replacing to multiple data in case store has something else
                    _ => *e = Data::Multiple(HashMap::new()),
                })
                .or_insert(Data::Multiple(HashMap::new()));

            if let Data::Multiple(map) = entry {
                current_entry = map.entry(*index);
            } else {
                unreachable!("Map should contain only Data::Multiple at this point");
            }
        }

        current_entry.or_insert(Data::Pending)
    }

    pub fn get(&self, field_id: &FieldId) -> Option<&Data<A>> {
        self.data.get(&field_id.as_usize())
    }
}
