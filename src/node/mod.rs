/// Base node it is facade which hide difficulties of realization and gets simple interface
pub mod base_node;

/// Guarantied that node type is full and has all node conditions without any restrictions
pub mod full_node;
/// Light node saves data in storage no more than 1GB
pub mod light_node;
/// Mobile node saves data in storage no more than 100mb and max buckets 10
pub mod mobile_node;
/// For work with popularity
pub mod seed_node;
