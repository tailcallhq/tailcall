#[derive(Default, Clone, Debug)]
pub struct Stats {
  pub min_ttl: Option<u64>,
}

impl Stats {
  pub fn new(min_ttl: Option<u64>) -> Self {
    Self { min_ttl }
  }
}
