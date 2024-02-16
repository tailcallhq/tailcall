#[derive(Debug, thiserror::Error, Clone, PartialEq, PartialOrd)]
pub enum Error {
    #[error("Haven't found auth parameters")]
    Missing,
    #[error("Couldn't validate auth request")]
    // in case we haven't managed to actually validate the request
    // and have failed somewhere else, usually while executing request
    ValidationCheckFailed,
    #[error("Auth validation failed")]
    Invalid,
}
