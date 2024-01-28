use std::ops::Deref;
use nom::AsBytes;

#[derive(Default)]
pub struct Body {
    data: Vec<u8>,
}

impl Body {
    pub fn empty() -> Self {
        Default::default()
    }
}

impl<T: AsRef<[u8]>> From<T> for Body {
    fn from(value: T) -> Self {
        Self {
            data: value.as_ref().to_vec(),
        }
    }
}

impl Deref for Body {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data.as_bytes()
    }
}