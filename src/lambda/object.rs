use std::fmt::{Display, Formatter};
#[derive(Clone, Debug)]
pub enum Object {
    Path(Vec<String>),
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Path(s) => f.write_str(format!("{:?}", s).as_str()), // TODO remove debug
        }
    }
}
