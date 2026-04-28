use creamy_utils::version::VersionError;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ManifestError {
    #[error("Field '{0}' cannot be empty")]
    EmptyValue(&'static str),

    #[error("{0}")]
    Version(#[from] VersionError),

    #[error("{0}")]
    Toml(#[from] toml::de::Error),
}
