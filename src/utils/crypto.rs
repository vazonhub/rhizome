use rsa::{RsaPrivateKey, RsaPublicKey, pkcs8::EncodePublicKey};
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::Sha256;
use std::fs;
use std::io;
use std::path::Path;

/// Generation of a 160-bit Node ID
///
/// Returns:
/// - 20 bytes (160 bits) of the node ID
pub fn generate_node_id() -> [u8; 20] {
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    let public_key_der = public_key
        .to_public_key_der()
        .expect("failed to encode public key");

    // Используем SHA-1 для получения 160-битного ID (20 байт)
    let mut hasher = Sha1::new();
    hasher.update(public_key_der.as_bytes());
    let result = hasher.finalize();

    let mut node_id = [0u8; 20];
    node_id.copy_from_slice(&result);
    node_id
}

/// Calculating the XOR distance between two Node IDs
///
/// Args:
/// - node_id1: First Node ID (20 bytes)
/// - node_id2: Second Node ID (20 bytes)
pub fn compute_distance(node_id1: &[u8], node_id2: &[u8]) -> Vec<u8> {
    if node_id1.len() != node_id2.len() {
        panic!("Node IDs must have the same length");
    }

    node_id1
        .iter()
        .zip(node_id2.iter())
        .map(|(a, b)| a ^ b)
        .collect()
}

/// Hashing the key for DHT (SHA-256)
///
/// Args:
/// - key: The key for hashing
pub fn hash_key(key: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(key);
    let result = hasher.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Generating a key pair for cryptography
///
/// Returns:
/// - Tuple (private_key, public_key)
pub fn generate_keypair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

/// Save Node ID in file
pub fn save_node_id(node_id: &[u8], file_path: &Path) -> io::Result<()> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file_path, node_id)
}

/// Load Node ID from file
pub fn load_node_id(file_path: &Path) -> Option<Vec<u8>> {
    if !file_path.exists() {
        return None;
    }

    let node_id = fs::read(file_path).ok()?;

    if node_id.len() != 20 {
        panic!("Invalid node ID length: {}, expected 20", node_id.len());
    }

    Some(node_id)
}
