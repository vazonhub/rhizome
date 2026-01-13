//! # Rhizome Exceptions Module
//!
//! This module contains all custom error types used throughout the Rhizome library.
//! It is designed to be compatible with the `?` operator by providing a root
//! `RhizomeError` enum that encapsulates more specific error types.

use thiserror::Error;

/// The root error type for the Rhizome project.
///
/// This enum wraps specialized errors from different subsystems (DHT, Storage, etc.).
/// It allows for high-level error handling while preserving specific error details.
#[derive(Error, Debug)]
pub enum RhizomeError {
    /// Errors occurring during Distributed Hash Table (DHT) operations.
    #[error("DHT error: {0}")]
    Dht(#[from] DHTError),

    /// Errors occurring in the local or replicated storage systems.
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Errors related to network transport, bootstrapping, or traffic control.
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// Errors related to cryptographic verification and protocol security.
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    /// Indicates that an operation was attempted on an unsupported or unknown node type.
    #[error("Invalid node type")]
    InvalidNodeType,
}

/// Errors specific to DHT (Kademlia) operations.
#[derive(Error, Debug)]
pub enum DHTError {
    /// The requested node could not be found in the routing table.
    #[error("Node not found")]
    NodeNotFound,

    /// The key-value pair requested does not exist in the network.
    #[error("Value not found in DHT")]
    ValueNotFound,

    /// An unspecified error occurred within the DHT logic.
    #[error("General DHT error")]
    General,
}

/// Errors specific to local data storage and replication.
#[derive(Error, Debug)]
pub enum StorageError {
    /// The local storage limit has been reached.
    #[error("Storage full")]
    StorageFull,

    /// Data could not be successfully synchronized across replicas.
    #[error("Replication error")]
    ReplicationError,

    /// An unspecified error occurred within the storage engine.
    #[error("General storage error")]
    General,
}

/// Errors specific to network transport and connectivity.
#[derive(Error, Debug)]
pub enum NetworkError {
    /// The node failed to join the network through the provided bootstrap addresses.
    #[error("Bootstrap process failed")]
    BootstrapError,

    /// The remote node or local rate limiter blocked the request due to frequency.
    #[error("Rate limit exceeded")]
    RateLimitError,

    /// An unspecified error occurred at the network transport level.
    #[error("General network error")]
    General,
}

/// Errors specific to security, authentication, and integrity checks.
#[derive(Error, Debug)]
pub enum SecurityError {
    /// The cryptographic signature of a message or block is invalid.
    #[error("Invalid signature")]
    InvalidSignature,

    /// An unspecified error occurred during security processing.
    #[error("General security error")]
    General,
}

/// A convenience type alias for `std::result::Result` with [`RhizomeError`].
///
/// Use this alias to simplify function signatures across the project.
///
/// # Example
///
/// ```rust
/// use crate::exceptions::Result;
///
/// fn do_something() -> Result<()> {
///     // Your logic here
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, RhizomeError>;
