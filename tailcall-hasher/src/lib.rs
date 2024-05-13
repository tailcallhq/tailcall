use std::hash::Hasher;

use fxhash::FxHasher;

#[derive(Default)]
pub struct TCHasher {
    hasher: FxHasher,
}

impl Hasher for TCHasher {
    fn finish(&self) -> u64 {
        self.hasher.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes)
    }
}
