/// Basic description of node in Kademlia DHT
/// 
/// Contain two modules:
/// - `NodeId` - uniq identifier in Kademlia DHT network
/// - `Node` - implementation of Kademlia DHT node with state, last seen and TTL
pub mod node;
pub mod protocol;
pub mod routing_table;