use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Msgpack error: {0}")]
    MsgpackError(#[from] rmp_serde::encode::Error),

    #[error("Msgpack decode error: {0}")]
    MsgpackDecodeError(#[from] rmp_serde::decode::Error),
}

/// Data serialization
///
/// Args:
/// - data: Data for serialization (any type with the Serialize attribute)
/// - format: Serialization format ("msgpack" or "json")
pub fn serialize<T: Serialize>(data: &T, format: &str) -> Result<Vec<u8>, SerializationError> {
    match format {
        "msgpack" => {
            let mut buf = Vec::new();
            data.serialize(&mut rmp_serde::Serializer::new(&mut buf))?;
            Ok(buf)
        }
        "json" => {
            let json_string = serde_json::to_string(data)?;
            Ok(json_string.into_bytes())
        }
        _ => Err(SerializationError::UnsupportedFormat(format.to_string())),
    }
}

/// Deserialization of data
///
/// Args:
/// - data: Serialized data (bytes)
/// - format: Serialization format ("msgpack" or "json")
pub fn deserialize<T: DeserializeOwned>(
    data: &[u8],
    format: &str,
) -> Result<T, SerializationError> {
    match format {
        "msgpack" => {
            let val = rmp_serde::from_slice(data)?;
            Ok(val)
        }
        "json" => {
            let val = serde_json::from_slice(data)?;
            Ok(val)
        }
        _ => Err(SerializationError::UnsupportedFormat(format.to_string())),
    }
}
