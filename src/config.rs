//! # Configuration Module
//!
//! This module manages all settings for the Rhizome node. It provides structured
//! access to DHT, Storage, Network, Popularity, and Security configurations.
//!
//! ## Features
//! - **Layered Loading**: Combines YAML file settings, environment variables (`.env`), and hardcoded defaults.
//! - **Serde Integration**: Uses `serde` for seamless serialization/deserialization to/from YAML.
//! - **Partial Configuration**: Supports loading incomplete YAML files by providing sensible defaults for missing fields.

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

// --- Default Value Providers ---
// These functions provide default values for Serde when a field is missing in the YAML file.

fn d_k() -> i32 {
    20
}
fn d_alpha() -> i32 {
    3
}
fn d_bits() -> i32 {
    160
}
fn d_refresh() -> i32 {
    3600
}
fn d_ping_to() -> f64 {
    5.0
}
fn d_req_to() -> f64 {
    10.0
}
fn d_data_dir() -> PathBuf {
    PathBuf::from("data")
}
fn d_max_storage() -> u64 {
    10 * 1024 * 1024 * 1024
}
fn d_ttl_def() -> i32 {
    86400
}
fn d_ttl_pop() -> i32 {
    2592000
}
fn d_ttl_act() -> i32 {
    604800
}
fn d_ttl_priv() -> i32 {
    10800
}
fn d_ttl_min() -> i32 {
    3600
}
fn d_host() -> String {
    "0.0.0.0".to_string()
}
fn d_port() -> i32 {
    8468
}
fn d_max_conn() -> i32 {
    100
}
fn d_conn_to() -> f64 {
    30.0
}
fn d_node_type() -> String {
    "full".to_string()
}
fn d_true() -> bool {
    true
}
fn d_false() -> bool {
    false
}
fn d_id_file() -> PathBuf {
    PathBuf::from("node_id.pem")
}
fn d_state_file() -> PathBuf {
    PathBuf::from("node_state.json")
}
fn d_upd_int() -> i32 {
    3600
}
fn d_exc_int() -> i32 {
    21600
}
fn d_glob_int() -> i32 {
    10800
}
fn d_pop_thr() -> f64 {
    7.0
}
fn d_act_thr() -> f64 {
    5.0
}
fn d_ring_size() -> i32 {
    8
}
fn d_rate_lim() -> i32 {
    100
}
fn d_rate_win() -> i32 {
    60
}
fn d_log_level() -> String {
    "INFO".to_string()
}
pub fn d_bucket_timeout() -> f64 {
    3600.0
}

/// Configuration for the Distributed Hash Table (DHT) and Kademlia parameters.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DHTConfig {
    /// Number of contacts stored in each bucket (k-value).
    #[serde(default = "d_k")]
    pub k: i32,
    /// Concurrency parameter for network lookups.
    #[serde(default = "d_alpha")]
    pub alpha: i32,
    /// Number of bits in the Node ID (usually 160).
    #[serde(default = "d_bits")]
    pub node_id_bits: i32,
    /// Total number of buckets in the routing table.
    #[serde(default = "d_bits")]
    pub bucket_count: i32,
    /// Interval in seconds for refreshing the routing table.
    #[serde(default = "d_refresh")]
    pub refresh_interval: i32,
    /// Timeout in seconds for PING requests.
    #[serde(default = "d_ping_to")]
    pub ping_timeout: f64,
    /// Timeout in seconds for standard DHT requests (FIND_NODE, etc).
    #[serde(default = "d_req_to")]
    pub request_timeout: f64,
}

impl Default for DHTConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

/// Settings related to local and replicated data storage.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    /// Directory path where data is persisted.
    #[serde(default = "d_data_dir")]
    pub data_dir: PathBuf,
    /// Maximum allowed size of the storage in bytes.
    #[serde(default = "d_max_storage")]
    pub max_storage_size: u64,
    /// Default Time-To-Live (TTL) for stored data.
    #[serde(default = "d_ttl_def")]
    pub default_ttl: i32,
    /// TTL for popular content.
    #[serde(default = "d_ttl_pop")]
    pub popular_ttl: i32,
    /// TTL for frequently active content.
    #[serde(default = "d_ttl_act")]
    pub active_ttl: i32,
    /// TTL for private or sensitive content.
    #[serde(default = "d_ttl_priv")]
    pub private_ttl: i32,
    /// Minimum guaranteed TTL regardless of popularity.
    #[serde(default = "d_ttl_min")]
    pub min_guaranteed_ttl: i32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

/// Network-specific settings including listening addresses and connection limits.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    /// The host IP address to bind the node's transport.
    #[serde(default = "d_host")]
    pub listen_host: String,
    /// The port number to listen on.
    #[serde(default = "d_port")]
    pub listen_port: i32,
    /// A list of bootstrap node addresses (e.g., "1.2.3.4:8468").
    #[serde(default)]
    pub bootstrap_nodes: Vec<String>,
    /// Maximum number of concurrent network connections.
    #[serde(default = "d_max_conn")]
    pub max_connections: i32,
    /// Timeout in seconds for establishing a connection.
    #[serde(default = "d_conn_to")]
    pub connection_timeout: f64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

/// General identity and state settings for the Rhizome node.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfig {
    /// Type of the node ("full", "light", "seed", etc).
    #[serde(default = "d_node_type")]
    pub node_type: String,
    /// Enable automatic detection of node capabilities.
    #[serde(default = "d_true")]
    pub auto_detect_type: bool,
    /// Path to the file containing the Node's identity (PEM).
    #[serde(default = "d_id_file")]
    pub node_id_file: PathBuf,
    /// Path to the JSON file where node state is persisted across reboots.
    #[serde(default = "d_state_file")]
    pub state_file: PathBuf,
}

impl Default for NodeConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

/// Parameters for content popularity ranking and metrics exchange.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PopularityConfig {
    /// How often to update local popularity metrics.
    #[serde(default = "d_upd_int")]
    pub update_interval: i32,
    /// Interval for exchanging popularity data with neighbors.
    #[serde(default = "d_exc_int")]
    pub exchange_interval: i32,
    /// Interval for recalculating global ranking data.
    #[serde(default = "d_glob_int")]
    pub global_update_interval: i32,
    /// Score threshold to consider an item "popular".
    #[serde(default = "d_pop_thr")]
    pub popularity_threshold: f64,
    /// Score threshold for "active" status.
    #[serde(default = "d_act_thr")]
    pub active_threshold: f64,
}

impl Default for PopularityConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

/// Security, privacy, and traffic control settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    /// Enable Ring Signatures for sender anonymity.
    #[serde(default = "d_true")]
    pub enable_ring_signatures: bool,
    /// Size of the anonymity set for ring signatures.
    #[serde(default = "d_ring_size")]
    pub ring_size: i32,
    /// Use stealth addresses for transaction/message privacy.
    #[serde(default = "d_true")]
    pub enable_stealth_addresses: bool,
    /// Route traffic through the Tor network.
    #[serde(default = "d_false")]
    pub enable_tor: bool,
    /// Route traffic through the I2P network.
    #[serde(default = "d_false")]
    pub enable_i2p: bool,
    /// Maximum allowed requests per window from a single peer.
    #[serde(default = "d_rate_lim")]
    pub rate_limit_requests: i32,
    /// Window size in seconds for the rate limiter.
    #[serde(default = "d_rate_win")]
    pub rate_limit_window: i32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

/// The master configuration object for the entire Rhizome system.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub dht: DHTConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub node: NodeConfig,
    #[serde(default)]
    pub popularity: PopularityConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    /// Global logging level ("DEBUG", "INFO", "WARN", "ERROR").
    #[serde(default = "d_log_level")]
    pub log_level: String,
    /// Optional path to the log file. If None, logs to stdout.
    pub log_file: Option<PathBuf>,
}

impl Config {
    /// Loads the configuration from a YAML file and environment variables.
    ///
    /// It first attempts to load `.env` variables, then reads the specified YAML file.
    /// Environment variables (like `LOG_LEVEL`) override settings found in the file.
    /// If no file is found, it uses internal defaults for all parameters.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to the YAML file. Defaults to `config.yaml`.
    pub fn from_file(config_path: Option<PathBuf>) -> Self {
        let _ = dotenvy::dotenv();

        let path = config_path.unwrap_or_else(|| PathBuf::from("config.yaml"));

        let mut config: Config = if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_yaml::from_str(&content).unwrap_or_else(|_| serde_yaml::from_str("{}").unwrap())
        } else {
            serde_yaml::from_str("{}").unwrap()
        };

        if let Ok(env_level) = env::var("LOG_LEVEL") {
            config.log_level = env_level;
        }

        config
    }

    /// Persists the current configuration state to a YAML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the serialization fails or if the file cannot be written.
    pub fn to_file(&self, config_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_content = serde_yaml::to_string(self)?;
        fs::write(config_path, yaml_content)?;
        Ok(())
    }
}
