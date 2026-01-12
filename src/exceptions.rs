use thiserror::Error;

/// Базовая коренная ошибка для Rhizome
#[derive(Error, Debug)]
pub enum RhizomeError {
    #[error("DHT error: {0}")]
    Dht(#[from] DHTError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    #[error("Invalid node type")]
    InvalidNodeType,
}

/// Ошибки DHT операций
#[derive(Error, Debug)]
pub enum DHTError {
    #[error("Node not found")]
    NodeNotFound,

    #[error("Value not found in DHT")]
    ValueNotFound,

    #[error("General DHT error")]
    General,
}

/// Ошибки хранилища
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage full")]
    StorageFull,

    #[error("Replication error")]
    ReplicationError,

    #[error("General storage error")]
    General,
}

/// Ошибки сетевых операций
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Bootstrap process failed")]
    BootstrapError,

    #[error("Rate limit exceeded")]
    RateLimitError,

    #[error("General network error")]
    General,
}

/// Ошибки безопасности
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("General security error")]
    General,
}

/// Удобный тип Result для проекта Rhizome
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, RhizomeError>;
