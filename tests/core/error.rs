use std::path::Path;

use derive_more::From;
use markdown::mdast::{Heading, Node};
use tailcall::core::config::UnsupportedConfigFormat;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected code block with no specific language in {:?}", _0)]
    NoSpecificLanguage(Box<Path>),

    #[error("Unexpected number of {0} blocks in {:?} (only one is allowed)", _1)]
    UnexpectedNumberOfBlocks(String, Box<Path>),

    #[error(
        "Unexpected language in {0} block in {:?} (only JSON and YAML are supported)",
        _1
    )]
    #[from(ignore)]
    UnexpectedLanguage(String, Box<Path>),

    #[error("Serde JSON Error")]
    SerdeJson(serde_json::Error),

    #[error("Serde YAML Error")]
    SerdeYaml(serde_yaml::Error),

    #[error("Unexpected content of level {0} heading in {:?}: {:#?}", _1, _2)]
    UnexpectedHeadingContent(String, Box<Path>, Heading),

    #[error("Unexpected content of code in {:?}: {:#?}", _0, _1)]
    UnexpectedCodeContent(Box<Path>, Option<String>),

    #[error("Unexpected double-declaration of {0} in {:?}", _1)]
    #[from(ignore)]
    UnexpectedDoubleDeclaration(String, Box<Path>),

    #[error("Unexpected {0} annotation {:?} in {:?}", _1, _2)]
    UnexpectedAnnotation(String, String, Box<Path>),

    #[error("Unexpected level {0} heading in {:?}: {:#?}", _1, _2)]
    #[from(ignore)]
    UnexpectedHeadingLevel(String, Box<Path>, Heading),

    #[error("Unsupported Config Format {:?}", _0)]
    UnsupportedConfigFormat(UnsupportedConfigFormat),

    #[error("Unexpected component {:?} in {:?}: {:#?}", _0, _1, _2)]
    UnexpectedComponent(String, Box<Path>, Option<String>),

    #[error("Unexpected node in {:?}: {:#?}", _0, _1)]
    #[from(ignore)]
    UnexpectedNode(Box<Path>, Node),

    #[error(
        "Unexpected blocks in {:?}: You must define a GraphQL Config in an execution test.",
        _0
    )]
    #[from(ignore)]
    UnexpectedBlocks(Box<Path>),

    #[error("{0}")]
    TailcallPrettier(String),

    #[error("{0}")]
    #[from(ignore)]
    Execution(String),

    #[error("{0}")]
    #[from(ignore)]
    Validation(String),

    #[error("Std IO Error")]
    StdIO(std::io::Error),
}

pub type Result<A> = std::result::Result<A, Error>;
