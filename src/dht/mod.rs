/// Basic description of node in Kademlia DHT
///
/// Describe `Who`
///
/// Contain two modules:
/// - `NodeId` - uniq identifier in Kademlia DHT network
/// - `Node` - implementation of Kademlia DHT node with state, last seen and TTL
pub mod node;
/// Realization of Kademlia Work
///
/// Describe `How`
pub mod protocol;
pub mod routing_table;
