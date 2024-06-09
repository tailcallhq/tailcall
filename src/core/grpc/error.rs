use derive_more::From;
use prost_reflect::DescriptorError;
use serde_json;

use crate::core::blueprint::GrpcMethod;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Serde Json Error")]
    SerdeJsonError(serde_json::Error),

    // #[error("Arc Error")]
    // ArcError(Arc<anyhow::Error>),
    #[error("Prost Encode Error")]
    ProstEncodeError(prost::EncodeError),

    #[error("Prost Decode Error")]
    ProstDecodeError(prost::DecodeError),

    #[error("Empty Response")]
    EmptyResponse,

    #[error("Couldn't resolve message")]
    MessageNotResolved,

    #[error("Descriptor pool error")]
    DescriptorPoolError(DescriptorError),

    #[error("Protox Parse Error")]
    ProtoxParseError(protox_parse::ParseError),

    #[error("Couldn't find method {}", ._0.name)]
    MissingMethod(GrpcMethod),

    #[error("Unable to find list field on type")]
    MissingListField,

    #[error("Field not found {0}")]
    #[from(ignore)]
    MissingField(String),

    #[error("Service not found {0}")]
    #[from(ignore)]
    MissingService(String),
}

pub type Result<A> = std::result::Result<A, Error>;
