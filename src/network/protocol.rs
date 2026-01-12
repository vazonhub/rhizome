use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock, oneshot};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::dht::node::{Node, NodeID};
use crate::dht::protocol::NetworkProtocolTrait; // Трейт из предыдущего шага
use crate::dht::routing_table::RoutingTable;
use crate::exceptions::{NetworkError, RhizomeError};
use crate::network::transport::{Message, UDPTransport};
use crate::popularity::exchanger::PopularityExchanger;
use crate::security::rate_limiter::RateLimiter;
use crate::storage::main::Storage;

// Константы типов сообщений
pub const MSG_PING: u8 = 0x01;
pub const MSG_PONG: u8 = 0x02;
pub const MSG_FIND_NODE: u8 = 0x03;
pub const MSG_FIND_NODE_RESPONSE: u8 = 0x04;
pub const MSG_FIND_VALUE: u8 = 0x05;
pub const MSG_FIND_VALUE_RESPONSE: u8 = 0x06;
pub const MSG_STORE: u8 = 0x07;
pub const MSG_STORE_RESPONSE: u8 = 0x08;
pub const MSG_POPULARITY_EXCHANGE: u8 = 0x09;
pub const MSG_POPULARITY_EXCHANGE_RESPONSE: u8 = 0x0A;
pub const MSG_GLOBAL_RANKING_REQUEST: u8 = 0x0B;
pub const MSG_GLOBAL_RANKING_RESPONSE: u8 = 0x0C;

/// Структура сообщения в MessagePack (аналог словаря в Python)
#[derive(Serialize, Deserialize, Debug)]
struct ProtocolMessage {
    #[serde(rename = "type")]
    msg_type: u8,
    pub id: [u8; 16],
    pub node_id: [u8; 20],
    pub payload: serde_json::Value,
    pub timestamp: f64,
}

type ResponseSender = oneshot::Sender<(u8, serde_json::Value)>;

pub struct NetworkProtocol {
    transport: Arc<UDPTransport>,
    node_id: NodeID,
    local_address: SocketAddr,
    routing_table: Option<Arc<RwLock<RoutingTable>>>,
    storage: Option<Arc<Storage>>,
    pub popularity_exchanger: Arc<RwLock<Option<Arc<PopularityExchanger>>>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    pending_requests: Arc<Mutex<HashMap<[u8; 16], ResponseSender>>>,
    request_timeout: Duration,
}

impl NetworkProtocol {
    pub fn new(
        transport: Arc<UDPTransport>,
        node_id: NodeID,
        local_address: SocketAddr,
        routing_table: Option<Arc<RwLock<RoutingTable>>>,
        storage: Option<Arc<Storage>>,
    ) -> Self {
        Self {
            transport,
            node_id,
            local_address,
            routing_table,
            storage,
            popularity_exchanger: Arc::new(RwLock::new(None)),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(100, 60, 20))),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            request_timeout: Duration::from_secs(10),
        }
    }

    pub async fn start(self: Arc<Self>) -> Result<(), RhizomeError> {
        let proto = self.clone();
        let transport = self.transport.clone();

        transport
            .start(move |msg| {
                let p = proto.clone();
                Box::pin(async move {
                    p.handle_incoming_message(msg).await;
                })
            })
            .await
            .map_err(|_| RhizomeError::Network(NetworkError::General))?;

        info!("Network protocol started");
        Ok(())
    }

    /// Остановка протокола (Аналог stop в Python)
    pub async fn stop(&mut self) {
        self.transport.stop().await;
        info!("Network protocol stopped");
    }

    async fn handle_incoming_message(&self, message: Message) {
        // 1. Десериализация для Rate Limit
        let raw_msg: Result<ProtocolMessage, _> = rmp_serde::from_slice(&message.data);

        if let Ok(m) = raw_msg {
            // Проверка Rate Limit
            let mut limiter = self.rate_limiter.lock().await;
            if limiter.check_rate_limit(Some(&m.node_id)).is_err() {
                warn!(address = %message.address, "Rate limit exceeded");
                return;
            }
            drop(limiter);

            // 2. Проверка, является ли это ответом на наш запрос
            let mut pending = self.pending_requests.lock().await;
            if let Some(sender) = pending.remove(&m.id) {
                let _ = sender.send((m.msg_type, m.payload));
                return;
            }
            drop(pending);

            // 3. Обработка входящего запроса
            if let Err(e) = self
                .handle_request(m.msg_type, m.id, m.payload, message.address)
                .await
            {
                error!(error = %e, "Error handling request");
            }
        }
    }

    async fn handle_request(
        &self,
        msg_type: u8,
        msg_id: [u8; 16],
        payload: serde_json::Value,
        address: SocketAddr,
    ) -> Result<(), RhizomeError> {
        match msg_type {
            MSG_PING => {
                if let Some(rt_link) = &self.routing_table
                    && let Some(id_val) = payload.get("node_id").and_then(|v| v.as_array()) {
                        // Обновляем таблицу маршрутизации
                        let mut id_bytes = [0u8; 20];
                        for (i, v) in id_val.iter().enumerate().take(20) {
                            id_bytes[i] = v.as_u64().unwrap_or(0) as u8;
                        }
                        let sender_node = Node::new(
                            NodeID::new(id_bytes),
                            address.ip().to_string(),
                            address.port(),
                        );
                        rt_link.write().await.add_node(sender_node);
                    }

                let response_payload = serde_json::json!({
                    "node_id": self.node_id.0,
                    "address": self.local_address.to_string()
                });
                self.send_response(MSG_PONG, msg_id, response_payload, address)
                    .await?;
            }

            MSG_FIND_NODE => {
                if let (Some(rt_link), Some(target_val)) =
                    (&self.routing_table, payload.get("target_id"))
                {
                    // Парсинг TargetID и поиск ближайших
                    let mut id_bytes = [0u8; 20];
                    if let Some(arr) = target_val.as_array() {
                        for (i, v) in arr.iter().enumerate().take(20) {
                            id_bytes[i] = v.as_u64().unwrap_or(0) as u8;
                        }
                    }

                    let rt = rt_link.read().await;
                    let closest = rt.find_closest_nodes(&NodeID::new(id_bytes), rt.k);

                    let nodes_data: Vec<serde_json::Value> = closest
                        .iter()
                        .map(|n| {
                            serde_json::json!({
                                "node_id": n.node_id.0,
                                "address": n.address,
                                "port": n.port
                            })
                        })
                        .collect();

                    self.send_response(
                        MSG_FIND_NODE_RESPONSE,
                        msg_id,
                        serde_json::json!({"nodes": nodes_data}),
                        address,
                    )
                    .await?;
                }
            }

            MSG_FIND_VALUE => {
                if let (Some(storage), Some(key_val)) = (&self.storage, payload.get("key")) {
                    let key_bytes: Vec<u8> =
                        serde_json::from_value(key_val.clone()).unwrap_or_default();
                    let value = storage.get(key_bytes.clone()).await?;

                    if let Some(v) = value {
                        self.send_response(
                            MSG_FIND_VALUE_RESPONSE,
                            msg_id,
                            serde_json::json!({"found": true, "value": v}),
                            address,
                        )
                        .await?;
                    } else if let Some(rt_link) = &self.routing_table {
                        // Возвращаем ближайшие узлы, если значение не найдено
                        let mut id_bytes = [0u8; 20];
                        let len = key_bytes.len().min(20);
                        id_bytes[..len].copy_from_slice(&key_bytes[..len]);

                        let rt = rt_link.read().await;
                        let closest = rt.find_closest_nodes(&NodeID::new(id_bytes), rt.k);
                        let nodes_data: Vec<serde_json::Value> = closest.iter().map(|n| {
                            serde_json::json!({"node_id": n.node_id.0, "address": n.address, "port": n.port})
                        }).collect();

                        self.send_response(
                            MSG_FIND_VALUE_RESPONSE,
                            msg_id,
                            serde_json::json!({"found": false, "nodes": nodes_data}),
                            address,
                        )
                        .await?;
                    }
                }
            }

            MSG_STORE => {
                if let (Some(storage), Some(key_val), Some(val_val)) =
                    (&self.storage, payload.get("key"), payload.get("value"))
                {
                    let key: Vec<u8> = serde_json::from_value(key_val.clone()).unwrap_or_default();
                    let value: Vec<u8> =
                        serde_json::from_value(val_val.clone()).unwrap_or_default();
                    let ttl = payload.get("ttl").and_then(|v| v.as_i64()).unwrap_or(86400) as i32;

                    storage.put(key, value, ttl).await?;
                    self.send_response(
                        MSG_STORE_RESPONSE,
                        msg_id,
                        serde_json::json!({"success": true}),
                        address,
                    )
                    .await?;
                }
            }
            MSG_POPULARITY_EXCHANGE => {
                let exchanger_lock = self.popularity_exchanger.read().await;
                if let Some(exchanger) = exchanger_lock.as_ref() {
                    if let Some(local_metrics) = exchanger.get_local_metrics().await {
                        // Ранжируем
                        let ranked = exchanger.ranker.rank_items(&local_metrics, Some(100));
                        let items: Vec<serde_json::Value> = ranked
                            .iter()
                            .map(|item| {
                                serde_json::json!({
                                    "key": hex::encode(&item.key),
                                    "score": item.score,
                                    "metrics": item.metrics.to_dict()
                                })
                            })
                            .collect();

                        self.send_response(
                            MSG_POPULARITY_EXCHANGE_RESPONSE,
                            msg_id,
                            serde_json::json!({"items": items}),
                            address,
                        )
                        .await?;
                    }

                    // Обрабатываем полученные данные
                    if let Some(received_items) = payload.get("items").and_then(|v| v.as_array()) {
                        exchanger
                            .process_received_items(received_items.clone())
                            .await;
                    }
                }
            }
            MSG_GLOBAL_RANKING_REQUEST => {
                let exchanger_lock = self.popularity_exchanger.read().await;
                if let Some(exchanger) = exchanger_lock.as_ref() {
                    let ranking = exchanger.get_global_ranking_api().await;
                    self.send_response(
                        MSG_GLOBAL_RANKING_RESPONSE,
                        msg_id,
                        serde_json::json!({"ranking": ranking}),
                        address,
                    )
                    .await?;
                }
            }
            _ => debug!("Unhandled message type: {}", msg_type),
        }
        Ok(())
    }

    async fn send_response(
        &self,
        msg_type: u8,
        msg_id: [u8; 16],
        payload: serde_json::Value,
        address: SocketAddr,
    ) -> Result<(), RhizomeError> {
        let data = self.pack_message(msg_type, msg_id, payload)?;
        self.transport.send(&data, address).await?;
        Ok(())
    }

    fn pack_message(
        &self,
        msg_type: u8,
        msg_id: [u8; 16],
        payload: serde_json::Value,
    ) -> Result<Vec<u8>, RhizomeError> {
        let msg = ProtocolMessage {
            msg_type,
            id: msg_id,
            node_id: self.node_id.0,
            payload,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        };
        rmp_serde::to_vec(&msg).map_err(|_| RhizomeError::Network(NetworkError::General))
    }

    fn generate_msg_id(&self) -> [u8; 16] {
        rand::thread_rng().r#gen()
    }
}

#[async_trait]
impl NetworkProtocolTrait for NetworkProtocol {
    async fn ping(&self, node: &Node) -> bool {
        let msg_id = self.generate_msg_id();
        let (tx, rx) = oneshot::channel();

        self.pending_requests.lock().await.insert(msg_id, tx);

        let addr: SocketAddr = format!("{}:{}", node.address, node.port).parse().unwrap();
        let payload = serde_json::json!({"node_id": self.node_id.0});

        if let Ok(data) = self.pack_message(MSG_PING, msg_id, payload) {
            let _ = self.transport.send(&data, addr).await;

            if let Ok(Ok((msg_type, _))) = timeout(self.request_timeout, rx).await {
                return msg_type == MSG_PONG;
            }
        }

        self.pending_requests.lock().await.remove(&msg_id);
        false
    }

    async fn find_node(
        &self,
        target_id: &NodeID,
        remote_node: &Node,
    ) -> Result<Vec<Node>, RhizomeError> {
        let msg_id = self.generate_msg_id();
        let (tx, rx) = oneshot::channel();

        self.pending_requests.lock().await.insert(msg_id, tx);

        let addr: SocketAddr = format!("{}:{}", remote_node.address, remote_node.port)
            .parse()
            .unwrap();
        let payload = serde_json::json!({"target_id": target_id.0});

        let data = self.pack_message(MSG_FIND_NODE, msg_id, payload)?;
        self.transport.send(&data, addr).await?;

        match timeout(self.request_timeout, rx).await {
            Ok(Ok((msg_type, payload))) if msg_type == MSG_FIND_NODE_RESPONSE => {
                let mut nodes = Vec::new();
                if let Some(nodes_arr) = payload.get("nodes").and_then(|v| v.as_array()) {
                    for n_val in nodes_arr {
                        if let (Some(id_arr), Some(addr), Some(port)) = (
                            n_val.get("node_id").and_then(|v| v.as_array()),
                            n_val.get("address").and_then(|v| v.as_str()),
                            n_val.get("port").and_then(|v| v.as_u64()),
                        ) {
                            let mut id_bytes = [0u8; 20];
                            for (i, v) in id_arr.iter().enumerate().take(20) {
                                id_bytes[i] = v.as_u64().unwrap_or(0) as u8;
                            }
                            nodes.push(Node::new(
                                NodeID::new(id_bytes),
                                addr.to_string(),
                                port as u16,
                            ));
                        }
                    }
                }
                Ok(nodes)
            }
            _ => {
                self.pending_requests.lock().await.remove(&msg_id);
                Err(RhizomeError::Network(NetworkError::General))
            }
        }
    }

    async fn find_value(
        &self,
        key: &[u8],
        remote_node: &Node,
    ) -> Result<Option<Vec<u8>>, RhizomeError> {
        let msg_id = self.generate_msg_id();
        let (tx, rx) = oneshot::channel();

        self.pending_requests.lock().await.insert(msg_id, tx);
        let addr: SocketAddr = format!("{}:{}", remote_node.address, remote_node.port)
            .parse()
            .unwrap();

        let data = self.pack_message(MSG_FIND_VALUE, msg_id, serde_json::json!({"key": key}))?;
        self.transport.send(&data, addr).await?;

        match timeout(self.request_timeout, rx).await {
            Ok(Ok((msg_type, payload))) if msg_type == MSG_FIND_VALUE_RESPONSE => {
                if payload
                    .get("found")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let val: Vec<u8> =
                        serde_json::from_value(payload.get("value").cloned().unwrap_or_default())
                            .unwrap_or_default();
                    Ok(Some(val))
                } else {
                    Ok(None)
                }
            }
            _ => {
                self.pending_requests.lock().await.remove(&msg_id);
                Ok(None)
            }
        }
    }

    async fn store(
        &self,
        key: &[u8],
        value: &[u8],
        ttl: i32,
        remote_node: &Node,
    ) -> Result<bool, RhizomeError> {
        let msg_id = self.generate_msg_id();
        let (tx, rx) = oneshot::channel();

        self.pending_requests.lock().await.insert(msg_id, tx);
        let addr: SocketAddr = format!("{}:{}", remote_node.address, remote_node.port)
            .parse()
            .unwrap();

        let payload = serde_json::json!({"key": key, "value": value, "ttl": ttl});
        let data = self.pack_message(MSG_STORE, msg_id, payload)?;
        self.transport.send(&data, addr).await?;

        match timeout(self.request_timeout, rx).await {
            Ok(Ok((msg_type, payload))) if msg_type == MSG_STORE_RESPONSE => Ok(payload
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)),
            _ => {
                self.pending_requests.lock().await.remove(&msg_id);
                Ok(false)
            }
        }
    }
}
