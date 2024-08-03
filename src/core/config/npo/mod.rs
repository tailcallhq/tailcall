mod identifier;
mod queries;

use std::{collections::HashSet, rc::Rc};

pub use identifier::*;
pub use queries::*;

use super::Config;

struct NPOIdentifier<'a> {
    config: &'a Config,
}

impl<'a> NPOIdentifier<'a> {
    pub fn new(config: &'a Config) -> NPOIdentifier {
        NPOIdentifier { config }
    }

    fn iter(
        &self,
        path: Chunk<FieldName<'a>>,
        type_name: TypeName<'a>,
        is_list: bool,
        visited: &mut HashSet<TypeName<'a>>,
    ) -> Chunk<Chunk<FieldName<'a>>> {
        if visited.contains(&type_name) {
            return Chunk::new();
        } else {
            visited.insert(type_name);
        }
        let mut chunks = Chunk::new();
        if let Some(type_of) = self.config.find_type(&type_name.as_str()) {
            for (name, field) in type_of.fields.iter() {
                let path = path.clone().append(FieldName::new(name));
                let is_batch = field.has_batched_resolver();

                if field.has_resolver() {
                    if !is_batch && is_list {
                        chunks = chunks.append(path.clone());
                    }
                }

                let is_list = is_list | field.list;

                chunks = chunks.concat(self.iter(
                    path,
                    TypeName::new(field.type_of.as_str()),
                    is_list,
                    visited,
                ))
            }
        }

        chunks
    }

    fn find_chunks(&self) -> Chunk<Chunk<FieldName<'a>>> {
        match &self.config.schema.query {
            None => Chunk::new(),
            Some(query) => self.iter(
                Chunk::new(),
                TypeName::new(query.as_str()),
                false,
                &mut HashSet::new(),
            ),
        }
    }

    fn find(&self) -> Vec<Vec<&'a str>> {
        self.find_chunks()
            .as_vec()
            .iter()
            .map(|chunk| {
                chunk
                    .as_vec()
                    .iter()
                    .map(|field_name| field_name.as_str())
                    .collect()
            })
            .collect()
    }
}

#[derive(Clone)]
enum Chunk<A> {
    Nil,
    Cons(A, Rc<Chunk<A>>),
    Concat(Rc<Chunk<A>>, Rc<Chunk<A>>),
}

impl<A> Chunk<A> {
    fn new() -> Self {
        Self::Nil
    }

    fn from_vec(vec: Vec<A>) -> Chunk<A> {
        let mut path = Chunk::new();
        for a in vec.into_iter().rev() {
            path = Chunk::Cons(a, Rc::new(path));
        }
        path
    }

    fn append(self, a: A) -> Self {
        Chunk::Cons(a, Rc::new(self))
    }

    fn concat(self, other: Chunk<A>) -> Self {
        Self::Concat(Rc::new(self), Rc::new(other))
    }

    fn as_vec(&self) -> Vec<&A> {
        let mut vec = Vec::new();
        self.as_vec_mut(&mut vec);
        vec
    }

    fn as_vec_mut<'a>(&'a self, buf: &mut Vec<&'a A>) {
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
