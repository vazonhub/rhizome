use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

// Предполагаем, что функция XOR-расстояния находится в модуле crypto
use crate::utils::crypto::compute_distance;

/// 160-битный идентификатор узла
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeID(pub [u8; 20]);

impl NodeID {
    /// Создание нового NodeID (в Rust размер фиксирован массивом, валидация длины не нужна)
    pub fn new(id: [u8; 20]) -> Self {
        Self(id)
    }

    /// Вычисление XOR-расстояния до другого узла
    pub fn distance_to(&self, other: &NodeID) -> [u8; 20] {
        // Вызываем функцию из нашего модуля криптографии
        // Если там она возвращает Vec<u8>, приводим к массиву
        let dist_vec = compute_distance(&self.0, &other.0);
        let mut res = [0u8; 20];
        res.copy_from_slice(&dist_vec[..20]);
        res
    }
}

/// Реализация Display для красивого вывода (аналог __repr__)
impl fmt::Debug for NodeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_id = hex::encode(self.0);
        write!(f, "NodeID({}...)", &hex_id[..16])
    }
}

/// Узел в сети
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub node_id: NodeID,
    pub address: String,
    pub port: u16,
    pub last_seen: f64,
    pub failed_pings: u32,
}

impl Node {
    /// Создание нового узла (аналог __post_init__ с установкой времени)
    pub fn new(node_id: NodeID, address: String, port: u16) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        Self {
            node_id,
            address,
            port,
            last_seen: now,
            failed_pings: 0,
        }
    }

    /// Обновление времени последнего контакта
    pub fn update_seen(&mut self) {
        self.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        self.failed_pings = 0;
    }

    /// Запись неудачного ping
    pub fn record_failed_ping(&mut self) {
        self.failed_pings += 1;
    }

    /// Проверка, является ли узел устаревшим
    pub fn is_stale(&self, timeout: f64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        (now - self.last_seen) > timeout
    }
}

/// Реализация Hash для Node (на основе его NodeID)
impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
    }
}

/// Реализация Display для Node
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node({:?}, {}:{})",
            self.node_id, self.address, self.port
        )
    }
}
