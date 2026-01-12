use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

// Функции-помощники для значений по умолчанию (нужны для serde)
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DHTConfig {
    #[serde(default = "d_k")]
    pub k: i32,
    #[serde(default = "d_alpha")]
    pub alpha: i32,
    #[serde(default = "d_bits")]
    pub node_id_bits: i32,
    #[serde(default = "d_bits")]
    pub bucket_count: i32,
    #[serde(default = "d_refresh")]
    pub refresh_interval: i32,
    #[serde(default = "d_ping_to")]
    pub ping_timeout: f64,
    #[serde(default = "d_req_to")]
    pub request_timeout: f64,
}

impl Default for DHTConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    #[serde(default = "d_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "d_max_storage")]
    pub max_storage_size: u64,
    #[serde(default = "d_ttl_def")]
    pub default_ttl: i32,
    #[serde(default = "d_ttl_pop")]
    pub popular_ttl: i32,
    #[serde(default = "d_ttl_act")]
    pub active_ttl: i32,
    #[serde(default = "d_ttl_priv")]
    pub private_ttl: i32,
    #[serde(default = "d_ttl_min")]
    pub min_guaranteed_ttl: i32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    #[serde(default = "d_host")]
    pub listen_host: String,
    #[serde(default = "d_port")]
    pub listen_port: i32,
    #[serde(default)]
    pub bootstrap_nodes: Vec<String>,
    #[serde(default = "d_max_conn")]
    pub max_connections: i32,
    #[serde(default = "d_conn_to")]
    pub connection_timeout: f64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfig {
    #[serde(default = "d_node_type")]
    pub node_type: String,
    #[serde(default = "d_true")]
    pub auto_detect_type: bool,
    #[serde(default = "d_id_file")]
    pub node_id_file: PathBuf,
    #[serde(default = "d_state_file")]
    pub state_file: PathBuf,
}

impl Default for NodeConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PopularityConfig {
    #[serde(default = "d_upd_int")]
    pub update_interval: i32,
    #[serde(default = "d_exc_int")]
    pub exchange_interval: i32,
    #[serde(default = "d_glob_int")]
    pub global_update_interval: i32,
    #[serde(default = "d_pop_thr")]
    pub popularity_threshold: f64,
    #[serde(default = "d_act_thr")]
    pub active_threshold: f64,
}

impl Default for PopularityConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    #[serde(default = "d_true")]
    pub enable_ring_signatures: bool,
    #[serde(default = "d_ring_size")]
    pub ring_size: i32,
    #[serde(default = "d_true")]
    pub enable_stealth_addresses: bool,
    #[serde(default = "d_false")]
    pub enable_tor: bool,
    #[serde(default = "d_false")]
    pub enable_i2p: bool,
    #[serde(default = "d_rate_lim")]
    pub rate_limit_requests: i32,
    #[serde(default = "d_rate_win")]
    pub rate_limit_window: i32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

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
    #[serde(default = "d_log_level")]
    pub log_level: String,
    pub log_file: Option<PathBuf>,
}

impl Config {
    /// Загрузка конфигурации из файла (аналог from_file в Python)
    pub fn from_file(config_path: Option<PathBuf>) -> Self {
        // Загрузка .env переменных
        let _ = dotenvy::dotenv();

        let path = config_path.unwrap_or_else(|| PathBuf::from("config.yaml"));

        let mut config: Config = if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_yaml::from_str(&content).unwrap_or_else(|_| serde_yaml::from_str("{}").unwrap())
        } else {
            serde_yaml::from_str("{}").unwrap()
        };

        // Приоритет переменной окружения для log_level
        if let Ok(env_level) = env::var("LOG_LEVEL") {
            config.log_level = env_level;
        }

        config
    }

    /// Сохранение конфигурации в файл (аналог to_file в Python)
    pub fn to_file(&self, config_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_content = serde_yaml::to_string(self)?;
        fs::write(config_path, yaml_content)?;
        Ok(())
    }
}
