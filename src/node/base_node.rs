use rand::Rng;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::dht::node::{Node, NodeID};
use crate::dht::protocol::{DHTProtocol, NetworkProtocolTrait};
use crate::dht::routing_table::RoutingTable;
use crate::exceptions::RhizomeError;
use crate::network::protocol::NetworkProtocol;
use crate::network::transport::UDPTransport;
use crate::popularity::exchanger::PopularityExchanger;
use crate::popularity::metrics::MetricsCollector;
use crate::popularity::ranking::PopularityRanker;
use crate::replication::replicator::Replicator;
use crate::storage::main::Storage;
use crate::utils::crypto::{generate_node_id, load_node_id, save_node_id};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NodeType {
    Seed,
    Full,
    Light,
    Mobile,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NodeType::Seed => "seed",
            NodeType::Full => "full",
            NodeType::Light => "light",
            NodeType::Mobile => "mobile",
        };
        write!(f, "{}", s)
    }
}

pub struct BaseNode {
    pub config: Config,
    pub node_id: NodeID,
    pub node_type: NodeType,

    // Компоненты, обернутые в Arc для совместного использования задачами
    pub routing_table: Arc<RwLock<RoutingTable>>,
    pub storage: Arc<Storage>,
    pub transport: Arc<UDPTransport>,
    pub metrics_collector: Arc<RwLock<MetricsCollector>>,
    pub popularity_ranker: Arc<PopularityRanker>,
    pub network_protocol: Arc<NetworkProtocol>,
    pub dht_protocol: Arc<DHTProtocol>,
    pub popularity_exchanger: Arc<PopularityExchanger>,
    pub replicator: Arc<Replicator>,

    // Состояние
    pub is_running: Arc<RwLock<bool>>,
    pub start_time: Arc<RwLock<Option<f64>>>,
}

#[allow(dead_code)]
impl BaseNode {
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Определение типа узла
        if config.node.auto_detect_type
            && let Some(detected) = Self::detect_node_type(&config) {
                config.node.node_type = detected.to_string(); // Обновляем строку в конфиге
            }

        let node_type = match config.node.node_type.as_str() {
            "seed" => NodeType::Seed,
            "full" => NodeType::Full,
            "light" => NodeType::Light,
            _ => NodeType::Mobile,
        };

        // 2. Загрузка или генерация Node ID
        let node_id_path = PathBuf::from(&config.node.node_id_file);
        let node_id_bytes = match load_node_id(&node_id_path) {
            Some(bytes) => {
                info!(path = ?node_id_path, "Node ID loaded from file");
                bytes
            }
            None => {
                info!("Generating new node ID");
                let bytes = generate_node_id().to_vec();
                save_node_id(&bytes, &node_id_path)?;
                bytes
            }
        };
        let mut id_fixed = [0u8; 20];
        id_fixed.copy_from_slice(&node_id_bytes[..20]);
        let node_id = NodeID::new(id_fixed);

        // 3. Инициализация базовых компонентов
        let routing_table = Arc::new(RwLock::new(RoutingTable::new(
            node_id,
            config.dht.k as usize,
            config.dht.bucket_count as usize,
        )));

        let storage = Arc::new(unsafe { Storage::new(config.storage.clone())? });

        let transport = Arc::new(UDPTransport::new(
            &config.network.listen_host,
            config.network.listen_port as u16,
        ));

        let metrics_collector = Arc::new(RwLock::new(MetricsCollector::new()));

        let popularity_ranker = Arc::new(PopularityRanker::new(
            config.popularity.popularity_threshold,
            config.popularity.active_threshold,
        ));

        // 4. Сетевой и DHT протоколы
        let listen_addr: std::net::SocketAddr = format!(
            "{}:{}",
            config.network.listen_host, config.network.listen_port
        )
        .parse()?;

        let network_protocol = Arc::new(NetworkProtocol::new(
            transport.clone(),
            node_id,
            listen_addr,
            Some(routing_table.clone()),
            Some(storage.clone()),
        ));

        let dht_protocol = Arc::new(DHTProtocol::new(
            routing_table.clone(),
            storage.clone(),
            Some(network_protocol.clone()),
        ));

        // 5. Популярность и Репликация
        let popularity_exchanger = Arc::new(PopularityExchanger::new(
            network_protocol.clone(),
            popularity_ranker.clone(),
            Some(metrics_collector.clone()),
        ));

        let replicator = Arc::new(Replicator::new(
            dht_protocol.clone(),
            storage.clone(),
            5,
            10,
        ));

        Ok(Self {
            config,
            node_id,
            node_type,
            routing_table,
            storage,
            transport,
            metrics_collector,
            popularity_ranker,
            network_protocol,
            dht_protocol,
            popularity_exchanger,
            replicator,
            is_running: Arc::new(RwLock::new(false)),
            start_time: Arc::new(RwLock::new(None)),
        })
    }

    /// Автоматическое определение типа узла на основе свободного места
    fn detect_node_type(config: &Config) -> Option<NodeType> {
        let path = &config.storage.data_dir;
        let _ = std::fs::create_dir_all(path);

        if let Ok(stat) = fs2::free_space(path) {
            let gb = 1024 * 1024 * 1024;
            if stat >= 100 * gb {
                return Some(NodeType::Seed);
            }
            if stat >= 10 * gb {
                return Some(NodeType::Full);
            }
            if stat >= gb {
                return Some(NodeType::Light);
            }
        }
        Some(NodeType::Mobile)
    }

    /// Запуск узла
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.is_running.write().await;
        if *running {
            return Ok(());
        }

        info!(node_id = %hex::encode(&self.node_id.0[..8]), "Starting node");

        *running = true;
        *self.start_time.write().await = Some(Self::get_now());

        // 1. Запуск сетевого уровня
        let net = self.network_protocol.clone();
        net.start().await?;

        // 2. Bootstrap (подключение к сети)
        self.bootstrap().await;

        // 3. Запуск фоновых задач (рефрешинг и очистка)
        let node_ref = Arc::new(self.clone_ptrs());
        tokio::spawn(async move {
            Self::background_loop(node_ref).await;
        });

        // 4. Запуск задач популярности
        let node_ref_pop = Arc::new(self.clone_ptrs());
        tokio::spawn(async move {
            Self::popularity_loop(node_ref_pop).await;
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.is_running.write().await;
        if !*running {
            return Ok(());
        }

        info!("Stopping node");
        *running = false; // Это заставит циклы background_loop и popularity_loop завершиться

        // Остановка сетевого протокола
        // self.network_protocol.stop().await;

        // Сохранение состояния в файл
        if let Err(e) = self.save_state().await {
            error!(error = %e, "Failed to save node state during stop");
        }

        // В Rust хранилище (Heed/LMDB) закроется автоматически, когда Arc<Storage>
        // выйдет из области видимости (Drop), но если нужно явно — можно добавить метод в Storage.

        info!("Node stopped");
        Ok(())
    }

    /// Сохранение состояния (Аналог _save_state в Python)
    async fn save_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state_file = PathBuf::from(&self.config.node.state_file);

        // Создаем директории если их нет
        if let Some(parent) = state_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let rt = self.routing_table.read().await;
        let total_nodes: usize = rt.buckets.iter().map(|b| b.nodes.len()).sum();
        let buckets_with_nodes = rt.buckets.iter().filter(|b| !b.nodes.is_empty()).count();

        // Формируем JSON структуру
        let state = serde_json::json!({
            "node_id": hex::encode(self.node_id.0),
            "node_type": self.node_type.to_string(),
            "start_time": *self.start_time.read().await,
            "is_running": false,
            "routing_table_stats": {
                "total_nodes": total_nodes,
                "buckets_with_nodes": buckets_with_nodes,
            },
        });

        // Записываем в файл
        let file = std::fs::File::create(state_file)?;
        serde_json::to_writer_pretty(file, &state)?;

        debug!("Node state saved");
        Ok(())
    }

    /// Загрузка состояния (Аналог _load_state в Python)
    pub async fn load_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state_file = PathBuf::from(&self.config.node.state_file);
        if !state_file.exists() {
            return Ok(());
        }

        let file = std::fs::File::open(state_file)?;
        let state: serde_json::Value = serde_json::from_reader(file)?;

        if let Some(saved_id_hex) = state.get("node_id").and_then(|v| v.as_str()) {
            let current_id_hex = hex::encode(self.node_id.0);
            if saved_id_hex != current_id_hex {
                warn!(
                    saved = %&saved_id_hex[..16],
                    current = %&current_id_hex[..16],
                    "Node ID mismatch in saved state"
                );
            }
        }

        debug!("Node state loaded");
        Ok(())
    }

    /// Процесс подключения к начальным узлам
    async fn bootstrap(&self) {
        let bootstrap_nodes = &self.config.network.bootstrap_nodes;
        if bootstrap_nodes.is_empty() {
            warn!("No bootstrap nodes configured");
            return;
        }

        for addr_str in bootstrap_nodes {
            if let Ok(addr) = addr_str.parse::<std::net::SocketAddr>() {
                // Временный узел (ID узнаем после PING)
                let boot_node =
                    Node::new(NodeID::new([0u8; 20]), addr.ip().to_string(), addr.port());

                if self.network_protocol.ping(&boot_node).await {
                    info!(address = %addr_str, "Bootstrap node connected");
                    self.routing_table.write().await.add_node(boot_node);

                    // Итеративный поиск себя для заполнения таблицы
                    let _ = self.dht_protocol.find_node(&self.node_id).await;
                }
            }
        }
    }

    /// Обмен данными о популярности с соседними узлами (Аналог _exchange_popularity)
    pub async fn exchange_popularity(&self) -> Result<(), RhizomeError> {
        let all_metrics = self
            .metrics_collector
            .read()
            .await
            .get_all_metrics()
            .clone();
        if all_metrics.is_empty() {
            return Ok(());
        }

        // Получаем соседние узлы из routing table (первые 10)
        let neighbor_nodes = {
            let rt = self.routing_table.read().await;
            let mut nodes = rt.get_all_nodes();
            nodes.truncate(10);
            nodes
        };

        if neighbor_nodes.is_empty() {
            return Ok(());
        }

        // Обмениваемся данными
        self.popularity_exchanger
            .exchange_top_items(all_metrics, neighbor_nodes.clone(), 100)
            .await;

        info!(
            neighbors = neighbor_nodes.len(),
            "Exchanged popularity data"
        );
        Ok(())
    }

    /// Основной цикл фоновых задач (рефрешинг бакетов)
    async fn background_loop(node: Arc<BaseNodePtrs>) {
        while *node.is_running.read().await {
            // Очистка старых данных в хранилище
            if let Ok(deleted) = node.storage.cleanup_expired().await
                && deleted > 0 {
                    debug!(count = deleted, "Cleaned up expired data");
                }

            // Рефрешинг бакетов
            let refresh_interval = node.config.dht.refresh_interval as f64;
            let mut buckets_to_refresh = Vec::new();

            {
                let rt = node.routing_table.read().await;
                let now = Self::get_now();
                for (i, bucket) in rt.buckets.iter().enumerate() {
                    if !bucket.nodes.is_empty() && (now - bucket.last_updated) > refresh_interval {
                        buckets_to_refresh.push(i);
                    }
                }
            }

            for idx in buckets_to_refresh {
                let random_id = node.generate_random_id_for_bucket(idx);
                let _ = node.dht_protocol.find_node(&random_id).await;
                debug!(index = idx, "Bucket refreshed");
            }

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    /// Цикл задач популярности (Ранжирование, Репликация, Обмен)
    async fn popularity_loop(node: Arc<BaseNodePtrs>) {
        let mut last_update = 0.0;
        let mut last_exchange = 0.0;

        while *node.is_running.read().await {
            let now = Self::get_now();

            // 1. Обновление рейтингов и Репликация (каждый час)
            if now - last_update >= node.config.popularity.update_interval as f64 {
                let metrics = node
                    .metrics_collector
                    .read()
                    .await
                    .get_all_metrics()
                    .clone();
                let ranked = node.popularity_ranker.rank_items(&metrics, Some(100));

                // Продление TTL популярных данных
                for item in &ranked {
                    if item.score >= node.config.popularity.popularity_threshold {
                        let _ = node.storage.extend_ttl(item.key.clone(), 1.0).await;
                    }
                }

                // Репликация
                node.replicator
                    .replicate_popular_items(ranked, node.config.popularity.popularity_threshold)
                    .await;

                last_update = now;
            }

            // 2. Обмен данными (каждые 6 часов)
            if now - last_exchange >= node.config.popularity.exchange_interval as f64 {
                let metrics = node
                    .metrics_collector
                    .read()
                    .await
                    .get_all_metrics()
                    .clone();
                let neighbors = node.routing_table.read().await.get_all_nodes();

                node.popularity_exchanger
                    .exchange_top_items(metrics, neighbors, 100)
                    .await;
                last_exchange = now;
            }

            // 3. Обновление свежести
            node.metrics_collector.write().await.update_all_freshness();

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    /// Генерация случайного ID для Kademlia бакета
    fn generate_random_id_for_bucket(&self, bucket_index: usize) -> NodeID {
        let mut rng = rand::thread_rng();
        let mut random_id = self.node_id.0;

        let byte_idx = bucket_index / 8;
        let bit_idx = bucket_index % 8;

        if byte_idx < 20 {
            let flip_bit = 0x80 >> bit_idx;
            random_id[byte_idx] ^= flip_bit;

            // Заполняем остальные биты случайным образом
            for (i, byte) in random_id.iter_mut().enumerate().skip(byte_idx).take(20 - byte_idx) {
                let mask = if i == byte_idx {
                    (1 << (7 - bit_idx)) - 1
                } else {
                    0xFF
                };
                *byte ^= rng.r#gen::<u8>() & mask;
            }
        }
        NodeID::new(random_id)
    }

    pub async fn find_value(&self, key: &[u8]) -> Result<Vec<u8>, RhizomeError> {
        self.metrics_collector
            .write()
            .await
            .record_find_value(key.to_vec(), Some(self.node_id.0.to_vec()));
        self.dht_protocol.find_value(key).await
    }

    pub async fn store(&self, key: &[u8], value: &[u8], ttl: i32) -> Result<bool, RhizomeError> {
        let success = self.dht_protocol.store(key, value, ttl).await?;
        let replication_count = if success { self.config.dht.k as u32 } else { 1 };
        self.metrics_collector
            .write()
            .await
            .record_store(key.to_vec(), replication_count);
        Ok(success)
    }

    fn get_now() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }

    /// Вспомогательный метод для клонирования указателей для потоков
    pub(crate) fn clone_ptrs(&self) -> BaseNodePtrs {
        BaseNodePtrs {
            config: self.config.clone(),
            routing_table: self.routing_table.clone(),
            storage: self.storage.clone(),
            metrics_collector: self.metrics_collector.clone(),
            popularity_ranker: self.popularity_ranker.clone(),
            dht_protocol: self.dht_protocol.clone(),
            popularity_exchanger: self.popularity_exchanger.clone(),
            replicator: self.replicator.clone(),
            is_running: self.is_running.clone(),
        }
    }
}

/// Структура только с Arc-указателями для передачи в фоновые задачи
pub(crate) struct BaseNodePtrs {
    pub(crate) config: Config,
    pub(crate) routing_table: Arc<RwLock<RoutingTable>>,
    storage: Arc<Storage>,
    pub(crate) metrics_collector: Arc<RwLock<MetricsCollector>>,
    pub(crate) popularity_ranker: Arc<PopularityRanker>,
    dht_protocol: Arc<DHTProtocol>,
    pub(crate) popularity_exchanger: Arc<PopularityExchanger>,
    replicator: Arc<Replicator>,
    pub(crate) is_running: Arc<RwLock<bool>>,
}

impl BaseNodePtrs {
    fn generate_random_id_for_bucket(&self, _bucket_index: usize) -> NodeID {
        // (Логика идентична методу выше)
        NodeID::new([0u8; 20]) // Заглушка
    }
}
