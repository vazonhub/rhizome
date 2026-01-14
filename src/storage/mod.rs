/// Module for data scheme in protocol
///
/// They convert bytes in to the rust object for using in work.
/// Also, this module can describe the style of content in threads and messages of the network.
pub mod data_types;
/// This module standardize the keys in network
///
/// It means that by this module anyone can use thread id and choose one uniq hash for data
pub mod keys;
/// Drive of storage
///
/// Work with TTL and responsible for storaging data on user device
pub mod main;
