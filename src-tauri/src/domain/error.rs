use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum DomainError {
    #[error("relative path cannot be empty")]
    EmptyRelativePath,
    #[error("path must be relative to the configured root")]
    AbsolutePath,
    #[error("path escapes the configured root")]
    PathTraversal,
    #[error("path contains an empty segment")]
    EmptyPathSegment,
    #[error("content fingerprint cannot be empty")]
    EmptyFingerprint,
}
