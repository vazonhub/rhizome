use crate::dht::node::{Node, NodeID};
use std::time::{SystemTime, UNIX_EPOCH};

/// Вспомогательная функция для получения текущего времени
fn get_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// k-бакет для хранения узлов на определенном расстоянии
pub struct KBucket {
    pub k: usize,
    pub nodes: Vec<Node>,
    pub last_updated: f64,
}

impl KBucket {
    pub fn new(k: usize) -> Self {
        Self {
            k,
            nodes: Vec::with_capacity(k),
            last_updated: get_now(),
        }
    }

    /// Добавление узла в бакет (LRU логика)
    pub fn add_node(&mut self, node: Node) -> bool {
        // Если узел уже есть, перемещаем его в конец (LRU)
        if let Some(index) = self.nodes.iter().position(|n| n.node_id == node.node_id) {
            self.nodes.remove(index);
            self.nodes.push(node);
            self.last_updated = get_now();
            return true;
        }

        // Если есть место, добавляем в конец
        if self.nodes.len() < self.k {
            self.nodes.push(node);
            self.last_updated = get_now();
            return true;
        }

        // Бакет полон
        false
    }

    pub fn remove_node(&mut self, node_id: &NodeID) {
        if let Some(index) = self.nodes.iter().position(|n| &n.node_id == node_id) {
            self.nodes.remove(index);
            self.last_updated = get_now();
        }
    }

    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes.clone()
    }

    pub fn is_full(&self) -> bool {
        self.nodes.len() >= self.k
    }
}

/// Таблица маршрутизации Kademlia
pub struct RoutingTable {
    pub node_id: NodeID,
    pub k: usize,
    pub buckets: Vec<KBucket>,
}

impl RoutingTable {
    pub fn new(node_id: NodeID, k: usize, bucket_count: usize) -> Self {
        let mut buckets = Vec::with_capacity(bucket_count);
        for _ in 0..bucket_count {
            buckets.push(KBucket::new(k));
        }

        Self {
            node_id,
            k,
            buckets,
        }
    }

    /// Получение индекса бакета для целевого ID (на основе XOR расстояния)
    fn get_bucket_index(&self, target_id: &NodeID) -> usize {
        let distance = self.node_id.distance_to(target_id); // [u8; 20]

        for (i, &byte) in distance.iter().enumerate() {
            if byte != 0 {
                // leading_zeros() возвращает кол-во нулевых битов слева
                let leading_zeros = byte.leading_zeros() as usize;
                let index = i * 8 + leading_zeros;
                return index.min(self.buckets.len() - 1);
            }
        }

        // Если все байты 0, значит это наш собственный ID
        self.buckets.len() - 1
    }

    /// Добавление узла в таблицу маршрутизации
    pub fn add_node(&mut self, node: Node) -> bool {
        if node.node_id == self.node_id {
            return false;
        }

        let bucket_index = self.get_bucket_index(&node.node_id);

        // Проверяем, полон ли бакет
        if self.buckets[bucket_index].is_full() {
            // Проверяем наличие устаревших узлов (timeout берем 3600.0 как в классе Node)
            let stale_index = self.buckets[bucket_index]
                .nodes
                .iter()
                .position(|n| n.is_stale(3600.0));

            if let Some(idx) = stale_index {
                // Заменяем устаревший узел
                self.buckets[bucket_index].nodes.remove(idx);
                return self.buckets[bucket_index].add_node(node);
            }
            return false;
        }

        self.buckets[bucket_index].add_node(node)
    }

    pub fn remove_node(&mut self, node_id: &NodeID) {
        let bucket_index = self.get_bucket_index(node_id);
        self.buckets[bucket_index].remove_node(node_id);
    }

    /// Поиск ближайших узлов к целевому ID
    pub fn find_closest_nodes(&self, target_id: &NodeID, count: usize) -> Vec<Node> {
        let bucket_index = self.get_bucket_index(target_id);
        let mut closest_nodes: Vec<Node> = Vec::new();

        // Собираем узлы, начиная с целевого бакета и расширяя область
        for offset in 0..self.buckets.len() {
            let idx = (bucket_index + offset) % self.buckets.len();
            let nodes = self.buckets[idx].get_nodes();
            closest_nodes.extend(nodes);

            if closest_nodes.len() >= count * 2 {
                // Берем с запасом для точной сортировки
                break;
            }
        }

        // Сортируем по XOR расстоянию
        closest_nodes.sort_by_key(|n| n.node_id.distance_to(target_id));

        if closest_nodes.len() > count {
            closest_nodes.truncate(count);
        }

        closest_nodes
    }

    /// Получение всех узлов из таблицы
    pub fn get_all_nodes(&self) -> Vec<Node> {
        self.buckets
            .iter()
            .flat_map(|bucket| bucket.nodes.clone())
            .collect()
    }
}
