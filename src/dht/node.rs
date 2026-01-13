use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::crypto::compute_distance;

/// 160-bits node identifier for Kademlia DHT Network
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeID(pub [u8; 20]);

impl NodeID {
    /// Creation of uniq node identifier
    pub fn new(id: [u8; 20]) -> Self {
        Self(id)
    }

    /// Calculate XOR-distance between nodes
    pub fn distance_to(&self, other: &NodeID) -> [u8; 20] {
        let dist_vec = compute_distance(&self.0, &other.0);
        let mut res = [0u8; 20];
        res.copy_from_slice(&dist_vec[..20]);
        res
    }
}

/// Create beautiful output on Debug mode
/// Convert from `[12, 14, 10, ...]` to string like: `NodeID(a1b2c3...)`
impl fmt::Debug for NodeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_id = hex::encode(self.0);
        write!(f, "NodeID({}...)", &hex_id[..16])
    }
}

/// Node in the network
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// Uniq `NodeId` identifier
    pub node_id: NodeID,
    /// IP address _(exm. '127.0.0.1')_
    pub address: String,
    /// Address port _(exm. 8080)_
    pub port: u16,
    /// Time of last answer from node
    pub last_seen: f64,
    /// Counter of bad requests to the node _(work like TTL in ipv4)_
    pub failed_pings: u32,
}

impl Node {
    /// Create new node
    ///
    /// Last seen of node is now after node create
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

    /// Update node time
    ///
    /// Call if we have some pings from node
    pub fn update_seen(&mut self) {
        self.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        self.failed_pings = 0;
    }

    /// Fail ping state change
    ///
    /// Call if we find bad ping to the node
    pub fn record_failed_ping(&mut self) {
        self.failed_pings += 1;
    }

    /// Check is node valid
    ///
    /// Function compare current time with time of last seen of the node
    pub fn is_stale(&self, timeout: f64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        (now - self.last_seen) > timeout
    }
}

/// Implementation of Hash for Node
///
/// Need to be work with Collections
impl Hash for Node {
    /// Get Node hash code
    ///
    /// Return node hash by using hash function on node_id of the node
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
    }
}

/// Implementation node output style
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node({:?}, {}:{})",
            self.node_id, self.address, self.port
        )
    }
}
