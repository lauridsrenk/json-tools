use thiserror::Error;

#[derive(Debug, Error)]
#[error("could not find {at_path}")]
pub struct PathNotFound {
    pub at_path: String,
}

#[derive(Debug, Error)]
#[error("value at path {at_path} is not a Dict")]
pub struct WrongValueAtPath {
    pub at_path: String,
}
