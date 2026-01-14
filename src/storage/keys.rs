use crate::utils::crypto::hash_key;

/// DHT key builder
pub struct DHTKeyBuilder;

#[allow(dead_code)]
impl DHTKeyBuilder {
    /// Key for list of all threads
    pub fn global_threads() -> [u8; 32] {
        hash_key("global:threads".as_bytes())
    }

    /// Key for N-top popular themes
    pub fn global_popular() -> [u8; 32] {
        hash_key("global:popular".as_bytes())
    }

    /// Key for last N messages
    pub fn global_recent() -> [u8; 32] {
        hash_key("global:recent".as_bytes())
    }

    /// Key for list active seed-nodes
    pub fn global_seeds() -> [u8; 32] {
        hash_key("global:seeds".as_bytes())
    }

    /// Key for thread metadata
    pub fn thread_meta(thread_id: &str) -> [u8; 32] {
        hash_key(format!("thread:{}:meta", thread_id).as_bytes())
    }

    /// Key for chronological list of thread messages
    pub fn thread_index(thread_id: &str) -> [u8; 32] {
        hash_key(format!("thread:{}:index", thread_id).as_bytes())
    }

    /// Key for popular messages in thread
    pub fn thread_popular(thread_id: &str) -> [u8; 32] {
        hash_key(format!("thread:{}:popular", thread_id).as_bytes())
    }

    /// Key for thread statistic
    pub fn thread_stats(thread_id: &str) -> [u8; 32] {
        hash_key(format!("thread:{}:stats", thread_id).as_bytes())
    }

    /// Key for message
    pub fn message(message_hash: &str) -> [u8; 32] {
        hash_key(format!("msg:{}", message_hash).as_bytes())
    }

    /// Key for links ot the reply on message
    pub fn message_refs(message_hash: &str) -> [u8; 32] {
        hash_key(format!("msg:{}:refs", message_hash).as_bytes())
    }

    /// Key for reactions on message
    pub fn message_votes(message_hash: &str) -> [u8; 32] {
        hash_key(format!("msg:{}:votes", message_hash).as_bytes())
    }

    /// Key for user profile
    pub fn user_profile(pubkey: &str) -> [u8; 32] {
        hash_key(format!("user:{}:profile", pubkey).as_bytes())
    }

    /// Key for user thread
    pub fn user_threads(pubkey: &str) -> [u8; 32] {
        hash_key(format!("user:{}:threads", pubkey).as_bytes())
    }

    /// Key for user reputation
    pub fn user_reputation(pubkey: &str) -> [u8; 32] {
        hash_key(format!("user:{}:reputation", pubkey).as_bytes())
    }

    /// Parsing of the key for finding type
    pub fn parse_key(_key: &[u8]) -> Option<std::collections::HashMap<String, String>> {
        // В оригинале упрощенная версия возвращает None
        None
    }
}

/// Manager for work with keys
///
/// It is template for work with builder
pub struct KeyManager {
    // In Rust, we don't need to save builder instance
    // if methods are static
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Get key for thread metadata
    pub fn get_thread_meta_key(&self, thread_id: &str) -> [u8; 32] {
        DHTKeyBuilder::thread_meta(thread_id)
    }

    /// Get Key for message
    pub fn get_message_key(&self, message_hash: &str) -> [u8; 32] {
        DHTKeyBuilder::message(message_hash)
    }

    /// Get key for global list of threads
    pub fn get_global_threads_key(&self) -> [u8; 32] {
        DHTKeyBuilder::global_threads()
    }

    /// Get key for popular threads
    pub fn get_global_popular_key(&self) -> [u8; 32] {
        DHTKeyBuilder::global_popular()
    }
}
