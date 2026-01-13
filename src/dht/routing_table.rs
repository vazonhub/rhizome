use crate::config::d_bucket_timeout;
use crate::dht::node::{Node, NodeID};
use std::time::{SystemTime, UNIX_EPOCH};

/// Return current time in seconds
fn get_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// K-Buckets for saving nodes with their distance
pub struct KBucket {
    /// Volume of the bucket _(usually 20)_
    pub k: usize,
    /// List of nodes in this bucket
    pub nodes: Vec<Node>,
    /// Time of last update
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

    /// Node add with LRU logic
    pub fn add_node(&mut self, node: Node) -> bool {
        if let Some(index) = self.nodes.iter().position(|n| n.node_id == node.node_id) {
            self.nodes.remove(index);
            self.nodes.push(node);
            self.last_updated = get_now();
            return true;
        }

        if self.nodes.len() < self.k {
            self.nodes.push(node);
            self.last_updated = get_now();
            return true;
        }

        false
    }

    /// Remove node from bucket
    pub fn remove_node(&mut self, node_id: &NodeID) {
        if let Some(index) = self.nodes.iter().position(|n| &n.node_id == node_id) {
            self.nodes.remove(index);
            self.last_updated = get_now();
        }
    }

    /// Get nodes from bucket
    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes.clone()
    }

    /// Check bucket state
    pub fn is_full(&self) -> bool {
        self.nodes.len() >= self.k
    }
}

/// Routing table
pub struct RoutingTable {
    /// Our node id
    pub node_id: NodeID,
    /// K volume of buckets
    pub k: usize,
    /// 160-counted buckets for 160-bits NodeId
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

    /// Find bucket index for node id by XOR distance algo
    fn get_bucket_index(&self, target_id: &NodeID) -> usize {
        let distance = self.node_id.distance_to(target_id);

        for (i, &byte) in distance.iter().enumerate() {
            if byte != 0 {
                let leading_zeros = byte.leading_zeros() as usize;
                let index = i * 8 + leading_zeros;
                return index.min(self.buckets.len() - 1);
            }
        }

        self.buckets.len() - 1
    }

    /// Add node in routing table
    pub fn add_node(&mut self, node: Node) -> bool {
        if node.node_id == self.node_id {
            return false;
        }

        let bucket_index = self.get_bucket_index(&node.node_id);

        // Проверяем, полон ли бакет
        if self.buckets[bucket_index].is_full() {
            let stale_index = self.buckets[bucket_index]
                .nodes
                .iter()
                .position(|n| n.is_stale(d_bucket_timeout()));

            if let Some(idx) = stale_index {
                self.buckets[bucket_index].nodes.remove(idx);
                return self.buckets[bucket_index].add_node(node);
            }
            return false;
        }

        self.buckets[bucket_index].add_node(node)
    }

    /// Remove node
    pub fn remove_node(&mut self, node_id: &NodeID) {
        let bucket_index = self.get_bucket_index(node_id);
        self.buckets[bucket_index].remove_node(node_id);
    }

    /// Find closest nodes
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

    /// Getting all table nodes
    pub fn get_all_nodes(&self) -> Vec<Node> {
        self.buckets
            .iter()
            .flat_map(|bucket| bucket.nodes.clone())
            .collect()
    }
}
