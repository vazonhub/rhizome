use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Вспомогательная функция для получения текущего времени (Unix timestamp)
fn get_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularityMetrics {
    pub key: Vec<u8>,

    // Базовые метрики
    pub request_count: u64,
    pub request_rate: f64,
    pub replication_count: u32,
    pub freshness_score: f64,
    pub audience_size: usize,

    // Расширенные метрики
    pub social_engagements: u64,
    pub view_time: f64,
    pub seed_coverage: f64,

    // Временные метки
    pub first_seen: f64,
    pub last_request: f64,
    pub created_at: Option<f64>,

    // История запросов (не сериализуется напрямую в dict в Python,
    // но нужна для расчетов. В Rust помечаем skip для serde, если нужно)
    #[serde(skip)]
    pub request_timestamps: VecDeque<f64>,
    #[serde(skip)]
    pub requesting_nodes: HashSet<Vec<u8>>,
}

impl PopularityMetrics {
    pub fn new(key: Vec<u8>) -> Self {
        let now = get_now();
        Self {
            key,
            request_count: 0,
            request_rate: 0.0,
            replication_count: 1,
            freshness_score: 1.0,
            audience_size: 1,
            social_engagements: 0,
            view_time: 0.0,
            seed_coverage: 0.0,
            first_seen: now,
            last_request: now,
            created_at: None,
            request_timestamps: VecDeque::with_capacity(1000),
            requesting_nodes: HashSet::new(),
        }
    }

    /// Обновление метрик при запросе
    pub fn update_request(&mut self, node_id: Option<Vec<u8>>) {
        let now = get_now();
        self.request_count += 1;
        self.last_request = now;

        // Эмуляция deque(maxlen=1000)
        if self.request_timestamps.len() >= 1000 {
            self.request_timestamps.pop_front();
        }
        self.request_timestamps.push_back(now);

        if let Some(id) = node_id {
            self.requesting_nodes.insert(id);
            self.audience_size = self.requesting_nodes.len();
        }

        // Пересчет request_rate (запросов в час)
        let ts_len = self.request_timestamps.len();
        if ts_len > 1 {
            let first = *self.request_timestamps.front().unwrap();
            let last = *self.request_timestamps.back().unwrap();
            let time_span = last - first;

            if time_span > 0.0 {
                self.request_rate = (ts_len as f64 / time_span) * 3600.0;
            } else {
                self.request_rate = ts_len as f64 * 3600.0;
            }
        } else {
            self.request_rate = if self.request_count > 0 { 1.0 } else { 0.0 };
        }
    }

    /// Обновление метрики свежести
    pub fn update_freshness(&mut self, age_seconds: Option<f64>) {
        let age = match age_seconds {
            Some(a) => a,
            None => {
                let start_time = self.created_at.unwrap_or(self.first_seen);
                get_now() - start_time
            }
        };

        if age < 3600.0 {
            self.freshness_score = 1.0;
        } else if age < 86400.0 {
            self.freshness_score = 1.0 - (age / 86400.0) * 0.5;
        } else {
            let days = age / 86400.0;
            // Math: 0.5 * (0.5 ^ (days / 7))
            let score = 0.5 * (0.5f64).powf(days / 7.0);
            self.freshness_score = score.max(0.1);
        }
    }

    pub fn update_replication(&mut self, count: u32) {
        self.replication_count = self.replication_count.max(count);
    }

    pub fn update_social_engagement(&mut self, count: u64) {
        self.social_engagements += count;
    }

    /// Аналог to_dict (использует serde_json::Value для гибкости)
    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Аналог from_dict
    pub fn from_dict(data: serde_json::Value) -> Result<Self, serde_json::Error> {
        let mut metrics: Self = serde_json::from_value(data)?;
        // Инициализируем пустые коллекции, так как они не сериализованы
        metrics.request_timestamps = VecDeque::with_capacity(1000);
        metrics.requesting_nodes = HashSet::new();
        Ok(metrics)
    }
}

pub struct MetricsCollector {
    pub metrics: HashMap<Vec<u8>, PopularityMetrics>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    pub fn record_find_value(&mut self, key: Vec<u8>, node_id: Option<Vec<u8>>) {
        let m = self
            .metrics
            .entry(key.clone())
            .or_insert_with(|| PopularityMetrics::new(key.clone()));
        m.update_request(node_id);
        m.update_freshness(None);

        debug!(
            "Recorded FIND_VALUE for key: {}",
            hex::encode(&key[..key.len().min(8)])
        );
    }

    pub fn record_store(&mut self, key: Vec<u8>, replication_count: u32) {
        let m = self
            .metrics
            .entry(key.clone())
            .or_insert_with(|| PopularityMetrics::new(key.clone()));
        m.update_replication(replication_count);
        m.update_freshness(None);

        debug!(
            "Recorded STORE for key: {}, replication: {}",
            hex::encode(&key[..key.len().min(8)]),
            replication_count
        );
    }

    pub fn record_social_engagement(&mut self, key: Vec<u8>, count: u64) {
        let m = self
            .metrics
            .entry(key.clone())
            .or_insert_with(|| PopularityMetrics::new(key.clone()));
        m.update_social_engagement(count);

        debug!(
            "Recorded social engagement for key: {}, count: {}",
            hex::encode(&key[..key.len().min(8)]),
            count
        );
    }

    pub fn get_metrics(&self, key: &[u8]) -> Option<&PopularityMetrics> {
        self.metrics.get(key)
    }

    pub fn get_all_metrics(&self) -> &HashMap<Vec<u8>, PopularityMetrics> {
        &self.metrics
    }

    pub fn update_all_freshness(&mut self) {
        for m in self.metrics.values_mut() {
            m.update_freshness(None);
        }
    }

    pub fn cleanup_old_metrics(&mut self, max_age_days: u64) {
        let now = get_now();
        let max_age = max_age_days as f64 * 86400.0;

        let initial_len = self.metrics.len();
        self.metrics
            .retain(|_, v| (now - v.last_request) <= max_age);

        let removed = initial_len - self.metrics.len();
        if removed > 0 {
            info!("Cleaned up old metrics, removed count: {}", removed);
        }
    }
}
