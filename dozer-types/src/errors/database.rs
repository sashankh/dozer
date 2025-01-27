#![allow(clippy::enum_variant_names)]
use crate::errors::internal::BoxedError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Unable to open or create database at location: {0}")]
    OpenOrCreateError(String),
    #[error("Unable to deserialize type: {} - Reason: {}", typ, reason.to_string())]
    DeserializationError { typ: String, reason: BoxedError },
    #[error("Unable to serialize type: {} - Reason: {}", typ, reason.to_string())]
    SerializationError { typ: String, reason: BoxedError },
    #[error("Invalid dataset: {0}")]
    InvalidDatasetIdentifier(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    // Error forwarding
    #[error(transparent)]
    InternalError(#[from] BoxedError),
}
