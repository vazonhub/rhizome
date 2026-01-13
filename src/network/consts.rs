//! Consts for each type of message
//!
//! Need to be serialized for fast data transfer

/// Message for check node status
pub const MSG_PING: u8 = 0x01;

/// Answer after ping, which accept that node is live
pub const MSG_PONG: u8 = 0x02;

/// Request to find the closest node in network
pub const MSG_FIND_NODE: u8 = 0x03;

/// Answer with found nodes
pub const MSG_FIND_NODE_RESPONSE: u8 = 0x04;

/// Request message do get value from hash
pub const MSG_FIND_VALUE: u8 = 0x05;

/// Answer with value or with nodes with value
pub const MSG_FIND_VALUE_RESPONSE: u8 = 0x06;

/// Request to save pare key-value
pub const MSG_STORE: u8 = 0x07;

/// Approve or Reject to save data
pub const MSG_STORE_RESPONSE: u8 = 0x08;

/// Request to exchange popularity metrics
pub const MSG_POPULARITY_EXCHANGE: u8 = 0x09;

/// Answer on exchange request
pub const MSG_POPULARITY_EXCHANGE_RESPONSE: u8 = 0x0A;

/// Request to get global ranking
pub const MSG_GLOBAL_RANKING_REQUEST: u8 = 0x0B;

/// Answer with global ranking
pub const MSG_GLOBAL_RANKING_RESPONSE: u8 = 0x0C;
