use std::hash::Hasher;

use fxhash::FxHasher;

/// A hasher that uses the FxHash algorithm. Currently it's a dumb wrapper
/// around `fxhash::FxHasher`. We could potentially add some custom logic here
/// in the future.
#[derive(Default)]
pub struct TailcallHasher {
    hasher: FxHasher,
}

impl Hasher for TailcallHasher {
    fn finish(&self) -> u64 {
        self.hasher.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes)
    }
}
