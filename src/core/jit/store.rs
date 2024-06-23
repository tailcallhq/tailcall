use crate::core::jit::model::FieldId;

#[allow(unused)]
pub struct Store<A> {
    map: Vec<Data<A>>,
}
#[allow(unused)]
#[derive(Clone)]
pub enum Data<A> {
    /// Represents that the value was computed only once for the associated
    /// field
    Single(A),
    /// Represents that the value was computed multiple times for the associated
    /// field. The order is guaranteed by the executor to be the same as the
    /// other of invocation and not the other of completion.
    Multiple(Vec<A>),
    /// Represents that the value is yet to be computed
    Pending,
}

impl<A> Store<A> {
    pub fn new(size: usize) -> Self {
        Store { map: (0..size).map(|_| Data::Pending).collect() }
    }

    pub fn set(&mut self, field_id: FieldId, data: Data<A>) {
        self.map.insert(field_id.as_usize(), data);
    }

    pub fn get(&self, field_id: &FieldId) -> Option<&Data<A>> {
        self.map.get(field_id.as_usize())
    }
}
