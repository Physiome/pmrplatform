use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

#[derive(Debug, PartialEq, Error, Deserialize, Serialize)]
pub enum ArgumentError {
    #[error("unexpected value provided for argument id: {0}")]
    UnexpectedValue(i64),
    #[error("value expected but missing for argument id: {0}")]
    ValueExpected(i64),
}

#[derive(Debug, PartialEq, Error, Deserialize, Serialize)]
pub enum LookupError {
    #[error("registry `{1}` missing for argument id: {0}")]
    RegistryMissing(i64, String),
    #[error("invalid choice `{1}` for argument id: {0}")]
    InvalidChoice(i64, String),
    #[error("default value missing for argument id: {0}")]
    TaskTemplateArgNoDefault(i64),
}

#[derive(Debug, PartialEq, Error, Deserialize, Serialize)]
pub enum BuildArgError {
    #[error(transparent)]
    ArgumentError(#[from] ArgumentError),
    #[error(transparent)]
    LookupError(#[from] LookupError),
}

#[derive(Debug, PartialEq, Error, Deserialize, Serialize)]
pub struct BuildArgErrors(pub(crate) Vec<BuildArgError>);

mod display;
