use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Not found")]
    NotFound,
    #[error("Conflict: entity already exists")]
    Conflict,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
}

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
}
