//! Rhizome P2P
//!
//! Rhizome is a high—performance, decentralized P2P messaging library implemented on Rust.
//! It is based on the Kademlia DHT protocol with custom data replication and
//! content ranking mechanisms.
//!
//! ## Features
//! - 🦀 Rust Core: Maximum performance and memory security without GC.
//! - 🔒 Anonymity: DHT-based routing hides direct connections between network participants.
//! - ⚡ Async First: A fully asynchronous stack based on tokio and futures.
//! - 🔄 Smart replication: Automatic distribution of data to k-nearest nodes.
//! - 📈 Popularity system: Content in demand gets storage priority and a higher TTL.
//! - 📦 Modularity: You can use it as a ready-made CLI node, or connect it as a library (cargo lib) to your project.

uniffi::setup_scaffolding!("rhizome_p2p");

/// Configuration Module
pub mod config;
/// Rhizome Exceptions Module
pub mod exceptions;
/// Module for logging and registration of events
pub mod logger;

/// Kademlia DHT realization
pub mod dht;
/// Realization of network working on more low transport level
pub mod network;
/// Module for work with nodes: types of nodes and their main functions
pub mod node;
/// Module for work with exchange of popular data and analyze metrics for this data
pub mod popularity;
/// Need for data copying to other nodes in network
pub mod replication;
/// Security module for create network more stable
pub mod security;
/// Local storage in node for fast data choosing
pub mod storage;
mod uniffi_bindgen;
/// Some help functional for work with serialization and crypto
pub mod utils;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};

use crate::config::Config;
use crate::exceptions::{DHTError, NetworkError, RhizomeError};
use crate::node::full_node::FullNode;
use crate::storage::keys::KeyManager;
use crate::utils::crypto::hash_key;
use crate::utils::serialization::{deserialize, serialize};
use crate::utils::time::get_now_i64;

#[derive(uniffi::Record, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ThreadMetadataBridge {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub creator_pubkey: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub message_count: i32,
    pub last_activity: i64,
    pub popularity_score: f64,
}

#[derive(uniffi::Record, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MessageBridge {
    pub id: String,
    pub thread_id: String,
    pub parent_id: Option<String>,
    pub content: String,
    pub author_signature: Option<String>,
    pub timestamp: i64,
    pub content_type: String,
    pub attachments: Vec<String>,
}

#[derive(uniffi::Object)]
pub struct RhizomeClient {
    // Оборачиваем внутреннее состояние для возможности работы через &self
    inner: Arc<RwLock<ClientInner>>,
}

struct ClientInner {
    pub config: Config,
    pub node: Option<Arc<FullNode>>,
    pub key_manager: KeyManager,
    pub is_running: bool,
}

/// API client for work with protocol
#[uniffi::export]
impl RhizomeClient {
    #[uniffi::constructor]
    pub fn new(config_path: Option<String>) -> Arc<Self> {
        let final_config = if let Some(path) = config_path {
            Config::from_file(Some(PathBuf::from(path)))
        } else {
            let default_path = PathBuf::from("config.yaml");
            if default_path.exists() {
                Config::from_file(Some(default_path))
            } else {
                Config::from_file(None)
            }
        };

        Arc::new(Self {
            inner: Arc::new(RwLock::new(ClientInner {
                config: final_config,
                node: None,
                key_manager: KeyManager::new(),
                is_running: false,
            })),
        })
    }

    pub async fn start(&self) -> Result<(), RhizomeError> {
        let mut inner = self.inner.write().await;
        if inner.is_running {
            return Err(RhizomeError::Network(NetworkError::General));
        }

        let node = FullNode::new(inner.config.clone())
            .await
            .map_err(|_| RhizomeError::Dht(DHTError::General))?;

        let node_arc = Arc::new(node);
        node_arc
            .start()
            .await
            .map_err(|_| RhizomeError::Network(NetworkError::General))?;

        inner.node = Some(node_arc);
        inner.is_running = true;

        sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), RhizomeError> {
        let mut inner = self.inner.write().await;
        if let Some(node) = inner.node.take()
            && inner.is_running
        {
            node.stop()
                .await
                .map_err(|_| RhizomeError::Network(NetworkError::General))?;
            inner.is_running = false;
        }
        Ok(())
    }

    pub async fn create_thread(
        &self,
        thread_id: String,
        title: String,
        category: Option<String>,
        tags: Option<Vec<String>>,
        creator_pubkey: Option<String>,
        ttl: i32,
    ) -> Result<ThreadMetadataBridge, RhizomeError> {
        let inner = self.inner.read().await;
        let node = inner
            .node
            .as_ref()
            .ok_or(RhizomeError::Dht(DHTError::NodeNotFound))?;

        let creator = creator_pubkey
            .unwrap_or_else(|| format!("0x{}", hex::encode(&hash_key(thread_id.as_bytes())[..8])));

        let thread_meta = ThreadMetadataBridge {
            id: thread_id.clone(),
            title,
            created_at: get_now_i64(),
            creator_pubkey: creator,
            category,
            tags: tags.unwrap_or_default(),
            message_count: 0,
            last_activity: get_now_i64(),
            popularity_score: 0.0,
        };

        let meta_key = inner.key_manager.get_thread_meta_key(&thread_id);
        let meta_data =
            serialize(&thread_meta, "msgpack").map_err(|_| RhizomeError::Dht(DHTError::General))?;
        node.store(&meta_key, &meta_data, ttl).await?;

        // Обновление индекса
        let threads_key = inner.key_manager.get_global_threads_key();
        let mut thread_list: Vec<String> = match node.find_value(&threads_key).await {
            Ok(data) => deserialize(&data, "msgpack").unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        if !thread_list.contains(&thread_id) {
            thread_list.push(thread_id);
            let list_data = serialize(&thread_list, "msgpack")
                .map_err(|_| RhizomeError::Dht(DHTError::General))?;
            node.store(&threads_key, &list_data, 86400).await?;
        }

        Ok(thread_meta)
    }

    pub async fn add_message(
        &self,
        thread_id: String,
        content: String,
        author_signature: Option<String>,
        parent_id: Option<String>,
        content_type: String,
        ttl: i32,
    ) -> Result<MessageBridge, RhizomeError> {
        let inner = self.inner.read().await;
        let node = inner
            .node
            .as_ref()
            .ok_or(RhizomeError::Dht(DHTError::NodeNotFound))?;

        let timestamp = get_now_i64();
        let message_id = format!("msg_{}_{}", thread_id, timestamp);

        let signature = author_signature.unwrap_or_else(|| {
            format!("sig_{}", hex::encode(&hash_key(message_id.as_bytes())[..8]))
        });

        let message = MessageBridge {
            id: message_id.clone(),
            thread_id: thread_id.clone(),
            parent_id,
            content,
            author_signature: Some(signature),
            timestamp,
            content_type,
            attachments: vec![],
        };

        let message_hash = hex::encode(&hash_key(message_id.as_bytes())[..8]);
        let message_key = inner.key_manager.get_message_key(&message_hash);
        let message_data =
            serialize(&message, "msgpack").map_err(|_| RhizomeError::Dht(DHTError::General))?;

        node.store(&message_key, &message_data, ttl).await?;

        // Здесь мы бы вызвали update_thread, но для краткости опустим (логика аналогична)
        Ok(message)
    }

    // Для API используем String (JSON), так как UniFFI не поддерживает динамический Value
    pub async fn get_popular_threads_json(&self, limit: u32) -> Result<String, RhizomeError> {
        let inner = self.inner.read().await;
        let node = inner
            .node
            .as_ref()
            .ok_or(RhizomeError::Dht(DHTError::NodeNotFound))?;

        let all_metrics = node
            .metrics_collector
            .read()
            .await
            .get_all_metrics()
            .clone();
        let ranked = node
            .popularity_ranker
            .rank_items(&all_metrics, Some(limit as usize));

        let result = serde_json::json!(
            ranked
                .iter()
                .map(|item| {
                    serde_json::json!({
                        "key": hex::encode(&item.key),
                        "score": item.score
                    })
                })
                .collect::<Vec<_>>()
        );

        Ok(result.to_string())
    }

    pub async fn get_node_info_json(&self) -> String {
        let inner = self.inner.read().await;
        match &inner.node {
            Some(node) => {
                serde_json::json!({
                    "node_id": hex::encode(node.node_id.0),
                    "node_type": format!("{:?}", node.node_type),
                    "is_running": inner.is_running,
                    "address": format!("{}:{}", inner.config.network.listen_host, inner.config.network.listen_port),
                }).to_string()
            }
            None => serde_json::json!({"status": "not_initialized"}).to_string(),
        }
    }
}
