#[derive(Clone, Debug)]
pub enum Error {}

pub type Result<A> = std::result::Result<A, Error>;
