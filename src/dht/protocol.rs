use async_trait::async_trait;
use futures::future::join_all;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::dht::node::{Node, NodeID};
use crate::dht::routing_table::RoutingTable;
use crate::exceptions::{DHTError, RhizomeError};
use crate::storage::main::Storage;

/// Интерфейс для сетевого протокола, чтобы избежать циклической зависимости
#[async_trait]
pub trait NetworkProtocolTrait: Send + Sync {
    async fn ping(&self, node: &Node) -> bool;
    async fn find_node(
        &self,
        target_id: &NodeID,
        remote_node: &Node,
    ) -> Result<Vec<Node>, RhizomeError>;
    async fn find_value(
        &self,
        key: &[u8],
        remote_node: &Node,
    ) -> Result<Option<Vec<u8>>, RhizomeError>;
    async fn store(
        &self,
        key: &[u8],
        value: &[u8],
        ttl: i32,
        remote_node: &Node,
    ) -> Result<bool, RhizomeError>;
}

pub struct DHTProtocol {
    pub routing_table: Arc<RwLock<RoutingTable>>,
    pub storage: Arc<Storage>,
    pub network_protocol: Option<Arc<dyn NetworkProtocolTrait>>,
    pub alpha: usize,
}

impl DHTProtocol {
    pub fn new(
        routing_table: Arc<RwLock<RoutingTable>>,
        storage: Arc<Storage>,
        network_protocol: Option<Arc<dyn NetworkProtocolTrait>>,
    ) -> Self {
        Self {
            routing_table,
            storage,
            network_protocol,
            alpha: 3,
        }
    }

    /// Проверка доступности узла
    pub async fn ping(&self, node: &mut Node) -> bool {
        if let Some(net) = &self.network_protocol {
            let result = net.ping(node).await;
            if result {
                node.update_seen();
            } else {
                node.record_failed_ping();
            }
            result
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            node.update_seen();
            true
        }
    }

    /// Поиск узлов по идентификатору (Kademlia lookup)
    pub async fn find_node(&self, target_id: &NodeID) -> Result<Vec<Node>, RhizomeError> {
        let mut closest = {
            let rt = self.routing_table.read().await;
            rt.find_closest_nodes(target_id, self.alpha)
        };

        let net = match &self.network_protocol {
            Some(n) => n,
            None => return Ok(closest),
        };

        let mut seen_nodes: HashMap<NodeID, Node> =
            closest.iter().map(|n| (n.node_id, n.clone())).collect();
        let mut queried: HashSet<NodeID> = HashSet::new();

        loop {
            let candidates: Vec<Node> = closest
                .iter()
                .filter(|n| !queried.contains(&n.node_id))
                .take(self.alpha)
                .cloned()
                .collect();

            if candidates.is_empty() {
                break;
            }

            let mut tasks = Vec::new();
            for node in &candidates {
                tasks.push(net.find_node(target_id, node));
            }

            let results = join_all(tasks).await;
            let mut new_nodes_found = false;

            for found_nodes in results.into_iter().flatten() {
                for node in found_nodes {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        seen_nodes.entry(node.node_id)
                    {
                        e.insert(node.clone());
                        new_nodes_found = true;
                    }
                }
            }

            for node in candidates {
                queried.insert(node.node_id);
            }

            // Обновляем список ближайших
            let mut all_found: Vec<Node> = seen_nodes.values().cloned().collect();
            all_found.sort_by_key(|n| n.node_id.distance_to(target_id));
            closest = all_found.into_iter().take(self.alpha).collect();

            if !new_nodes_found {
                break;
            }
        }

        Ok(closest)
    }

    /// Поиск значения по ключу
    pub async fn find_value(&self, key: &[u8]) -> Result<Vec<u8>, RhizomeError> {
        // 1. Локальное хранилище
        if let Some(val) = self.storage.get(key.to_vec()).await? {
            return Ok(val);
        }

        let net = self
            .network_protocol
            .as_ref()
            .ok_or(RhizomeError::Dht(DHTError::ValueNotFound))?;

        // Создаем TargetID из ключа (первых 20 байт)
        let mut id_bytes = [0u8; 20];
        let len = key.len().min(20);
        id_bytes[..len].copy_from_slice(&key[..len]);
        let target_id = NodeID::new(id_bytes);

        let mut closest = {
            let rt = self.routing_table.read().await;
            rt.find_closest_nodes(&target_id, self.alpha)
        };

        let mut seen_nodes: HashMap<NodeID, Node> =
            closest.iter().map(|n| (n.node_id, n.clone())).collect();
        let mut queried: HashSet<NodeID> = HashSet::new();

        loop {
            let candidates: Vec<Node> = closest
                .iter()
                .filter(|n| !queried.contains(&n.node_id))
                .take(self.alpha)
                .cloned()
                .collect();

            if candidates.is_empty() {
                break;
            }

            // Параллельно запрашиваем значение
            let mut value_tasks = Vec::new();
            for node in &candidates {
                value_tasks.push(net.find_value(key, node));
            }
            let results = join_all(value_tasks).await;

            for result in results {
                if let Ok(Some(val)) = result {
                    return Ok(val); // Нашли значение!
                }
            }

            // Если не нашли, расширяем поиск узлов
            let mut node_tasks = Vec::new();
            for node in &candidates {
                node_tasks.push(net.find_node(&target_id, node));
            }
            let node_results = join_all(node_tasks).await;

            for nodes in node_results.into_iter().flatten() {
                for n in nodes {
                    seen_nodes.entry(n.node_id).or_insert(n);
                }
            }

            for node in candidates {
                queried.insert(node.node_id);
            }

            let mut all_found: Vec<Node> = seen_nodes.values().cloned().collect();
            all_found.sort_by_key(|n| n.node_id.distance_to(&target_id));
            closest = all_found.into_iter().take(self.alpha).collect();

            if queried.len() >= seen_nodes.len() {
                break;
            }
        }

        Err(RhizomeError::Dht(DHTError::ValueNotFound))
    }

    /// Сохранение данных (Kademlia STORE)
    pub async fn store(&self, key: &[u8], value: &[u8], ttl: i32) -> Result<bool, RhizomeError> {
        // 1. Сохраняем локально
        self.storage.put(key.to_vec(), value.to_vec(), ttl).await?;

        let net = match &self.network_protocol {
            Some(n) => n,
            None => return Ok(true),
        };

        let mut id_bytes = [0u8; 20];
        let len = key.len().min(20);
        id_bytes[..len].copy_from_slice(&key[..len]);
        let target_id = NodeID::new(id_bytes);

        // Находим ближайшие узлы
        let closest_nodes = self.find_node(&target_id).await?;

        if closest_nodes.is_empty() {
            return Ok(true);
        }

        let k = { self.routing_table.read().await.k };
        let mut store_tasks = Vec::new();

        for node in closest_nodes.iter().take(k) {
            store_tasks.push(net.store(key, value, ttl, node));
        }

        let results = join_all(store_tasks).await;
        let success_count = results
            .into_iter()
            .filter(|r| matches!(r, Ok(true)))
            .count();

        debug!(
            key = %hex::encode(&key[..key.len().min(8)]),
            success = success_count,
            attempted = k,
            "STORE completed"
        );

        Ok(success_count > 0)
    }
}
