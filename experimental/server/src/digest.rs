#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Digest(pub usize);

impl Digest {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        // TODO:
        Digest(0)
    }
}
