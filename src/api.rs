/// Module for work with network settings
pub mod config;
/// Module for work with exceptions
pub mod exceptions;
/// Module for work with logs
pub mod logger;

/// Kademlia DHT realization
pub mod dht;
pub mod network;
pub mod node;
pub mod popularity;
pub mod replication;
pub mod security;
pub mod storage;
pub mod utils;

use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, sleep};

// Импортируем все компоненты, созданные ранее
use crate::config::Config;
use crate::node::full_node::FullNode;
use crate::storage::data_types::{Message, ThreadMetadata};
use crate::storage::keys::KeyManager;
use crate::utils::crypto::hash_key;
use crate::utils::serialization::{deserialize, serialize};

pub struct RhizomeClient {
    pub config: Config,
    pub node: Option<Arc<FullNode>>,
    pub key_manager: KeyManager,
    is_running: bool,
}

impl RhizomeClient {
    /// Инициализация клиента
    pub fn new(config_path: Option<String>, config: Option<Config>) -> Self {
        let final_config = if let Some(c) = config {
            c
        } else if let Some(path) = config_path {
            Config::from_file(Some(PathBuf::from(path)))
        } else {
            let default_path = PathBuf::from("config.yaml");
            if default_path.exists() {
                Config::from_file(Some(default_path))
            } else {
                Config::from_file(None) // Значения по умолчанию
            }
        };

        Self {
            config: final_config,
            node: None,
            key_manager: KeyManager::new(),
            is_running: false,
        }
    }

    /// Запуск узла и подключение к сети
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_running {
            return Err("Node is already running".into());
        }

        let node = FullNode::new(self.config.clone()).await?;
        let node_arc = Arc::new(node);

        node_arc.start().await?;

        self.node = Some(node_arc);
        self.is_running = true;

        // Даем время на инициализацию (bootstrap)
        sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    /// Остановка узла
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(node) = self.node.take()
            && self.is_running
        {
            node.stop().await.expect("TODO: panic message");
            self.is_running = false;
        }

        Ok(())
    }

    fn get_now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// Создание нового треда
    pub async fn create_thread(
        &self,
        thread_id: &str,
        title: &str,
        category: Option<String>,
        tags: Option<Vec<String>>,
        creator_pubkey: Option<String>,
        ttl: i32,
    ) -> Result<ThreadMetadata, Box<dyn std::error::Error>> {
        let node = self.node.as_ref().ok_or("Node not running")?;

        let creator = creator_pubkey
            .unwrap_or_else(|| format!("0x{}", hex::encode(&hash_key(thread_id.as_bytes())[..8])));

        let thread_meta = ThreadMetadata {
            id: thread_id.to_string(),
            title: title.to_string(),
            created_at: Self::get_now(),
            creator_pubkey: creator,
            category,
            tags: tags.unwrap_or_default(),
            message_count: 0,
            last_activity: Self::get_now(),
            popularity_score: 0.0,
            encryption_type: "public".to_string(),
            access_control: None,
        };

        let meta_key = self.key_manager.get_thread_meta_key(thread_id);
        let meta_data = serialize(&thread_meta, "msgpack")?;

        let success = node.store(&meta_key, &meta_data, ttl).await?;

        if !success {
            return Err(format!("Failed to create thread: {}", thread_id).into());
        }

        Ok(thread_meta)
    }

    /// Поиск треда по ID
    pub async fn find_thread(
        &self,
        thread_id: &str,
    ) -> Result<Option<ThreadMetadata>, Box<dyn std::error::Error>> {
        let node = self.node.as_ref().ok_or("Node not running")?;
        let meta_key = self.key_manager.get_thread_meta_key(thread_id);

        match node.find_value(&meta_key).await {
            Ok(data) => {
                let thread_meta: ThreadMetadata = deserialize(&data, "msgpack")?;
                Ok(Some(thread_meta))
            }
            Err(_) => Ok(None),
        }
    }

    /// Обновление метаданных треда
    pub async fn update_thread(
        &self,
        thread_id: &str,
        updates: Value, // Используем JSON для динамических обновлений
    ) -> Result<Option<ThreadMetadata>, Box<dyn std::error::Error>> {
        let mut thread_meta = match self.find_thread(thread_id).await? {
            Some(m) => m,
            None => return Ok(None),
        };

        // Применяем обновления из JSON (аналог hasattr/setattr)
        if let Some(count) = updates.get("message_count").and_then(|v| v.as_i64()) {
            thread_meta.message_count = count as i32;
        }
        if let Some(score) = updates.get("popularity_score").and_then(|v| v.as_f64()) {
            thread_meta.popularity_score = score;
        }

        thread_meta.last_activity = updates
            .get("last_activity")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(Self::get_now);

        let node = self.node.as_ref().unwrap();
        let meta_key = self.key_manager.get_thread_meta_key(thread_id);
        let meta_data = serialize(&thread_meta, "msgpack")?;

        node.store(&meta_key, &meta_data, 86400).await?;

        Ok(Some(thread_meta))
    }

    /// Добавление сообщения в тред
    pub async fn add_message(
        &self,
        thread_id: &str,
        content: &str,
        author_signature: Option<String>,
        parent_id: Option<String>,
        content_type: &str,
        ttl: i32,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let node = self.node.as_ref().ok_or("Node not running")?;

        let thread_meta = self
            .find_thread(thread_id)
            .await?
            .ok_or_else(|| format!("Thread not found: {}", thread_id))?;

        let timestamp = Self::get_now();
        let message_id = format!("msg_{}_{}", thread_id, timestamp);

        let signature = author_signature.unwrap_or_else(|| {
            format!("sig_{}", hex::encode(&hash_key(message_id.as_bytes())[..8]))
        });

        let message = Message {
            id: message_id.clone(),
            thread_id: thread_id.to_string(),
            parent_id,
            content: content.to_string(),
            author_signature: Some(signature),
            timestamp,
            content_type: content_type.to_string(),
            attachments: vec![],
            metadata: serde_json::json!({}),
        };

        let message_hash = hex::encode(&hash_key(message_id.as_bytes())[..8]);
        let message_key = self.key_manager.get_message_key(&message_hash);
        let message_data = serialize(&message, "msgpack")?;

        let success = node.store(&message_key, &message_data, ttl).await?;

        if !success {
            return Err("Failed to store message".into());
        }

        // Обновляем метаданные треда
        let updates = serde_json::json!({
            "message_count": thread_meta.message_count + 1,
            "last_activity": timestamp
        });
        self.update_thread(thread_id, updates).await?;

        Ok(message)
    }

    /// Получение списка популярных тредов
    pub async fn get_popular_threads(
        &self,
        limit: usize,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let node = self.node.as_ref().ok_or("Node not running")?;

        let all_metrics = node
            .metrics_collector
            .read()
            .await
            .get_all_metrics()
            .clone();
        if all_metrics.is_empty() {
            return Ok(vec![]);
        }

        let ranked = node.popularity_ranker.rank_items(&all_metrics, Some(limit));

        Ok(ranked
            .iter()
            .map(|item| {
                serde_json::json!({
                    "key": hex::encode(&item.key),
                    "score": item.score
                })
            })
            .collect())
    }

    /// Поиск тредов (фильтрация)
    pub async fn search_threads(
        &self,
        _query: Option<&str>,
        category: Option<&str>,
        tags: Option<Vec<&str>>,
    ) -> Result<Vec<ThreadMetadata>, Box<dyn std::error::Error>> {
        // В оригинале: берем список ID из глобального индекса
        // Здесь мы имитируем эту логику
        let threads_key = self.key_manager.get_global_threads_key();
        let node = self.node.as_ref().ok_or("Node not running")?;

        let data = match node.find_value(&threads_key).await {
            Ok(d) => d,
            Err(_) => return Ok(vec![]),
        };

        let thread_ids: Vec<String> = deserialize(&data, "msgpack")?;
        let mut results = Vec::new();

        for id in thread_ids {
            if let Some(meta) = self.find_thread(&id).await? {
                if let Some(cat) = category
                    && meta.category.as_deref() != Some(cat)
                {
                    continue;
                }

                if let Some(ref t_filter) = tags
                    && !t_filter
                        .iter()
                        .any(|&tag| meta.tags.contains(&tag.to_string()))
                {
                    continue;
                }

                results.push(meta);
            }
        }

        Ok(results)
    }

    pub fn get_node_info(&self) -> Value {
        match &self.node {
            Some(node) => {
                serde_json::json!({
                    "node_id": hex::encode(node.node_id.0),
                    "node_type": format!("{:?}", node.node_type),
                    "is_running": self.is_running,
                    "address": format!("{}:{}", self.config.network.listen_host, self.config.network.listen_port),
                })
            }
            None => serde_json::json!({"status": "not_initialized"}),
        }
    }
}