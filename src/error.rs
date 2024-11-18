use derive_more::From;

#[derive(From, thiserror::Error, Debug)]
pub enum TcError {
    #[error("Worker Error")]
    Worker(crate::core::worker::Error),

    #[error("Unsupported Config Format")]
    UnsupportedConfigFormat(crate::core::config::UnsupportedConfigFormat),

    #[cfg(feature = "cli")]
    #[error("Unsupported File Format")]
    UnsupportedFileFormat(crate::cli::generator::source::UnsupportedFileFormat),

    #[cfg(feature = "cli")]
    #[error("LLM Error")]
    LLM(crate::cli::llm::Error),

    #[error("Tokio Task JoinError Error")]
    Join(tokio::task::JoinError),

    #[error("Serde JSON Error")]
    SerdeJson(serde_json::Error),

    #[error("From Utf8 Error")]
    FromUtf8(std::string::FromUtf8Error),

    #[error("Inquire Error")]
    Inquire(inquire::InquireError),

    #[error("Javascript Runtime Error")]
    Javascript(rquickjs::Error),

    #[error("Protox Error")]
    Protox(protox::Error),

    #[error("Runtime Error")]
    Report(miette::Report),

    #[error("Diagnostic Runtime Error")]
    Diagnostic(miette::MietteDiagnostic),

    #[error("Validation Error")]
    Validation(crate::core::valid::ValidationError<miette::MietteDiagnostic>)
}

pub type TcResult<A> = std::result::Result<A, TcError>;
