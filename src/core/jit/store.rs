use crate::core::jit::model::FieldId;

#[derive(Debug)]
pub struct Store<A> {
    data: Vec<Data<A>>,
}

#[derive(Clone)]
pub enum Data<A> {
    /// Represents that the value was computed only once for the associated
    /// field
    Single(A),
    /// Represents that the value was computed multiple times for the associated
    /// field. The order is guaranteed by the executor to be the same as the
    /// other of invocation and not the other of completion.
    // TODO: there could be multiple inside multiple in case of nested resolvers that are resolved
    // to lists
    Multiple(Vec<A>),
    /// Represents that the value is yet to be computed
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
    pub fn map<B>(self, ab: impl Fn(A) -> B) -> Data<B> {
        match self {
            Data::Single(a) => Data::Single(ab(a)),
            Data::Multiple(values) => Data::Multiple(values.into_iter().map(ab).collect()),
            Data::Pending => Data::Pending,
        }
    }
}

impl<A> Store<A> {
    pub fn new(size: usize) -> Self {
        Store { data: (0..size).map(|_| Data::Pending).collect() }
    }

    pub fn set(&mut self, field_id: FieldId, data: Data<A>) {
        self.data[field_id.as_usize()] = data;
    }

    pub fn set_single(&mut self, field_id: &FieldId, data: A) {
        self.data[field_id.as_usize()] = Data::Single(data);
    }

    pub fn set_multiple(&mut self, field_id: &FieldId, data: Vec<A>) {
        self.data[field_id.as_usize()] = Data::Multiple(data);
    }

    pub fn get(&self, field_id: &FieldId) -> Option<&Data<A>> {
        self.data.get(field_id.as_usize())
    }
}
