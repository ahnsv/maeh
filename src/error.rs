use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum MaehError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("toml decode: {0}")]
    TomlDecode(#[from] toml::de::Error),
    #[error("toml encode: {0}")]
    TomlEncode(#[from] toml::ser::Error),
    #[error("cache miss: {0}")]
    CacheMiss(String),
    #[error("capsule too large: {actual} chars > {max} chars")]
    CapsuleTooLarge { actual: usize, max: usize },
    #[error("backend: {0}")]
    Backend(#[from] maeh::backend::BackendError),
    #[error("usage: {0}")]
    Usage(String),
    #[error(transparent)]
    Clap(#[from] clap::Error),
}

pub(crate) type Result<T> = std::result::Result<T, MaehError>;
