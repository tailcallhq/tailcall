use std::collections::HashMap;

use crate::core::jit::model::FieldId;

/// Path to the data in the store with info
/// to resolve nested multiple data
#[derive(Debug, Clone)]
pub struct DataPath {
    /// List of paths, where every entry contains info about specific
    /// level ot multiple data
    multiple_path: Vec<usize>,
}

impl DataPath {
    /// Create default DataPath that resolved to single value
    pub fn single() -> Self {
        Self { multiple_path: Vec::new() }
    }

    /// Create new DataPath with specified additional entry
    pub fn with_index(&self, index: usize) -> Self {
        let mut multiple_path = self.multiple_path.clone();

        multiple_path.push(index);

        Self { multiple_path }
    }

    /// Iterator over indexes only for multiple paths.
    /// Helpful when collecting the data after it has been previously
    /// resolved
    pub fn multiple_indexes(&self) -> impl Iterator<Item = &usize> + '_ {
        self.multiple_path.iter()
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
    Single(A),
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
            Self::Single(_) => f.debug_tuple("Single").finish(),
            Self::Multiple(arg0) => f.debug_tuple("Multiple").field(&arg0.len()).finish(),
            Self::Pending => write!(f, "Pending"),
        }
    }
}

impl<A> Data<A> {
    pub fn map<B>(self, ab: impl Fn(A) -> B + Copy) -> Data<B> {
        match self {
            Data::Single(a) => Data::Single(ab(a)),
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

    pub fn set(&mut self, field_id: &FieldId, path: &DataPath, data: A) {
        let path = &path.multiple_path;
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

        *current_entry.or_insert(Data::Pending) = Data::Single(data);
    }

    pub fn get(&self, field_id: &FieldId) -> Option<&Data<A>> {
        self.data.get(&field_id.as_usize())
    }
}
