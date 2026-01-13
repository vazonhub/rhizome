/// Consts for each type of message
///
/// Need for serialization in network.
pub mod consts;
/// Network protocol
///
/// Module for sending data and receive data from internet.
/// They will transfer abstract command like (Ping, Store) in real bytes and send it by UDP.
/// Protocol work with answers and responsibility for safety.
pub mod protocol;
/// Module with realization of UDP
pub mod transport;
