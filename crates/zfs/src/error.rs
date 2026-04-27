use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("zfs command failed: {code:?}\nstdout: {stdout}\nstderr: {stderr}")]
    CommandFailed {
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },

    #[error("missing required key: {0}")]
    MissingKey(String),

    #[error("parse error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("parse error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error("invalid property value for '{property}': {src}")]
    InvalidProperty {
        property: &'static str,
        src: &'static str,
    },

    #[error("unknown {what}: '{value}'")]
    UnknownValue { what: &'static str, value: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("event stream closed unexpectedly")]
    EventStreamClosed,

    #[error("ZFS pool not found: {0}")]
    PoolNotFound(String),

    #[error("ZFS dataset not found: {0}")]
    DatasetNotFound(String),

    #[error("zpool events parsing failed: {0}")]
    EventParse(String),

    #[error("property conversion failed: {0}")]
    PropertyConversion(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
