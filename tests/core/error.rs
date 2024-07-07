use std::path::Path;
use std::fmt::Display;

use derive_more::{From, DebugCustom};
use markdown::mdast::{Heading, Node};
use tailcall::core::config::UnsupportedConfigFormat;

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Unexpected code block with no specific language in {:?}", _0)]
    NoSpecificLanguage(Box<Path>),

    #[debug(fmt = "Unexpected number of {0} blocks in {:?} (only one is allowed)", _1)]
    UnexpectedNumberOfBlocks(String, Box<Path>),

    #[debug(fmt = 
        "Unexpected language in {0} block in {:?} (only JSON and YAML are supported)",
        _1
    )]
    #[from(ignore)]
    UnexpectedLanguage(String, Box<Path>),

    #[debug(fmt = "Serde JSON Error")]
    SerdeJson(serde_json::Error),

    #[debug(fmt = "Serde YAML Error")]
    SerdeYaml(serde_yaml::Error),

    #[debug(fmt = "Unexpected content of level {0} heading in {:?}: {:#?}", _1, _2)]
    UnexpectedHeadingContent(String, Box<Path>, Heading),

    #[debug(fmt = "Unexpected content of code in {:?}: {:#?}", _0, _1)]
    UnexpectedCodeContent(Box<Path>, Option<String>),

    #[debug(fmt = "Unexpected double-declaration of {0} in {:?}", _1)]
    #[from(ignore)]
    UnexpectedDoubleDeclaration(String, Box<Path>),

    #[debug(fmt = "Unexpected {0} annotation {:?} in {:?}", _1, _2)]
    UnexpectedAnnotation(String, String, Box<Path>),

    #[debug(fmt = "Unexpected level {0} heading in {:?}: {:#?}", _1, _2)]
    #[from(ignore)]
    UnexpectedHeadingLevel(String, Box<Path>, Heading),

    #[debug(fmt = "Unsupported Config Format {:?}", _0)]
    UnsupportedConfigFormat(UnsupportedConfigFormat),

    #[debug(fmt = "Unexpected component {:?} in {:?}: {:#?}", _0, _1, _2)]
    UnexpectedComponent(String, Box<Path>, Option<String>),

    #[debug(fmt = "Unexpected node in {:?}: {:#?}", _0, _1)]
    #[from(ignore)]
    UnexpectedNode(Box<Path>, Node),

    #[debug(fmt = 
        "Unexpected blocks in {:?}: You must define a GraphQL Config in an execution test.",
        _0
    )]
    #[from(ignore)]
    UnexpectedBlocks(Box<Path>),

    #[debug(fmt = "{0}")]
    TailcallPrettier(String),

    #[debug(fmt = "{0}")]
    #[from(ignore)]
    Execution(String),

    #[debug(fmt = "{0}")]
    #[from(ignore)]
    Validation(String),

    #[debug(fmt = "Std IO Error")]
    StdIO(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<A> = std::result::Result<A, Error>;
