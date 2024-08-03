use std::rc::Rc;

///
/// A special data structure with a O(1) complexity for append and concat operations
#[derive(Clone)]
pub enum Chunk<A> {
    Nil,
    Cons(A, Rc<Chunk<A>>),
    Concat(Rc<Chunk<A>>, Rc<Chunk<A>>),
}

impl<A> Chunk<A> {
    pub fn new() -> Self {
        Self::Nil
    }

    pub fn append(self, a: A) -> Self {
        Chunk::Cons(a, Rc::new(self))
    }

    pub fn concat(self, other: Chunk<A>) -> Self {
        Self::Concat(Rc::new(self), Rc::new(other))
    }

    pub fn as_vec(&self) -> Vec<&A> {
        let mut vec = Vec::new();
        self.as_vec_mut(&mut vec);
        vec
    }

    pub fn as_vec_mut<'a>(&'a self, buf: &mut Vec<&'a A>) {
        match self {
            Chunk::Nil => {}
            Chunk::Cons(a, rest) => {
                buf.push(a);
                rest.as_vec_mut(buf);
            }
            Chunk::Concat(a, b) => {
                a.as_vec_mut(buf);
                b.as_vec_mut(buf);
            }
        }
    }
}
