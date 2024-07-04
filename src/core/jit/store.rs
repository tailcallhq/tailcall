use std::iter::repeat_with;

use crate::core::jit::model::FieldId;

/// Represents the size of multiple data (children of the resolved list)
/// and the index of currently processing element in that data
#[derive(Debug, Clone)]
struct MultipleDataPath {
    index: usize,
    len: usize,
}

/// Path to the data in the store with info
/// to resolve nested multiple data
#[derive(Debug, Clone)]
pub struct DataPath {
    /// List of paths, where every entry contains info about specific
    /// level ot multiple data
    multiple_path: Vec<MultipleDataPath>,
}

impl DataPath {
    /// Create default DataPath that resolved to single value
    pub fn single() -> Self {
        Self { multiple_path: Vec::new() }
    }

    /// Create new DataPath with specified additional entry
    pub fn with_path(&self, len: usize, index: usize) -> Self {
        let mut path = self.multiple_path.clone();

        path.push(MultipleDataPath { index, len });

        Self { multiple_path: path }
    }

    /// Iterator over indexes only for multiple paths.
    /// Helpful when collecting the data after it has been previously
    /// resolved
    pub fn multiple_indexes(&self) -> impl Iterator<Item = usize> + '_ {
        self.multiple_path.iter().map(|x| x.index)
    }
}

#[derive(Debug)]
pub struct Store<A> {
    data: Vec<Data<A>>,
}

#[derive(Clone, Default)]
pub enum Data<A> {
    /// Represents that the value was computed only once for the associated
    /// field
    Single(A),
    /// Represents that the value was computed multiple times for the associated
    /// field. The order is guaranteed by the executor to be the same as the
    /// other of invocation and not the other of completion.
    // TODO: there could be multiple inside multiple in case of nested resolvers that are resolved
    // to lists
    Multiple(Vec<Data<A>>),
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
            Data::Multiple(values) => {
                Data::Multiple(values.into_iter().map(|e| e.map(ab)).collect())
            }
            Data::Pending => Data::Pending,
        }
    }
}

impl<A> Store<A> {
    pub fn new(size: usize) -> Self {
        Store { data: (0..size).map(|_| Data::Pending).collect() }
    }

    pub fn set_data(&mut self, field_id: FieldId, data: Data<A>) {
        self.data[field_id.as_usize()] = data;
    }

    pub fn set(&mut self, field_id: &FieldId, path: &DataPath, data: A) {
        let mut current_data = &mut self.data[field_id.as_usize()];
        let path = &path.multiple_path;

        for path in path {
            if let Data::Multiple(v) = current_data {
                current_data = &mut v[path.index];
            } else {
                let v: Vec<_> = repeat_with(|| Data::Pending).take(path.len).collect();

                *current_data = Data::Multiple(v);
                if let Data::Multiple(v) = current_data {
                    current_data = &mut v[path.index];
                }
            };
        }

        *current_data = Data::Single(data)
    }

    pub fn get(&self, field_id: &FieldId) -> Option<&Data<A>> {
        self.data.get(field_id.as_usize())
    }
}
