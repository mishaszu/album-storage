use std::fmt::Display;

use tracing::error;

use crate::db::Error as DbError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ModalManagerNotInContext,
    ClientNotInContext,

    FailedToReadFile,
    FailedToReadDir,

    FailedToDeleteFile,

    BadImage,

    DbError(DbError),
    GraphQlError(async_graphql::Error),

    AuthError,
    AccessError(String),

    NotFound(String),

    InvalidID,

    EntityExists,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error!("{:<12} - graphql error - {self:?}", "GRAPHQL");
        match self {
            Error::AuthError => write!(f, "User not logged in"),
            Error::DbError(DbError::DbEntityNotFound)
            | Error::AccessError(_)
            | Error::NotFound(_) => write!(f, "Not found"),
            Error::GraphQlError(e) => write!(f, "{:?}", e),
            Error::DbError(_)
            | Error::ClientNotInContext
            | Error::FailedToReadDir
            | Error::ModalManagerNotInContext => write!(f, "Internal server error"),
            Error::InvalidID => write!(f, "Invalid ID"),
            Error::EntityExists => write!(f, "Entity exists"),
            Error::FailedToReadFile => write!(f, "Failed to read file"),
            Error::FailedToDeleteFile => write!(f, "Failed to delete file"),
            Error::BadImage => write!(f, "Bad image"),
        }
    }
}

impl From<DbError> for Error {
    fn from(e: DbError) -> Self {
        Error::DbError(e)
    }
}

impl From<async_graphql::Error> for Error {
    fn from(e: async_graphql::Error) -> Self {
        Error::GraphQlError(e)
    }
}
