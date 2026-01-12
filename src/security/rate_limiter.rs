use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;

// Предполагаем, что ошибки импортируются из вашего модуля exceptions
use crate::exceptions::{NetworkError, RhizomeError};

pub struct RateLimiter {
    max_requests: usize,
    window_seconds: u64,
    per_node_limit: usize,

    // История общих запросов (timestamps)
    request_history: VecDeque<f64>,

    // Запросы по узлам: NodeID -> deque of timestamps
    node_requests: HashMap<Vec<u8>, VecDeque<f64>>,
}

impl RateLimiter {
    /// Инициализация rate limiter
    pub fn new(max_requests: usize, window_seconds: u64, per_node_limit: usize) -> Self {
        Self {
            max_requests,
            window_seconds,
            per_node_limit,
            // Резервируем место для оптимизации (аналог maxlen в Python)
            request_history: VecDeque::with_capacity(max_requests * 2),
            node_requests: HashMap::new(),
        }
    }

    /// Получение текущего времени в секундах (аналог time.time())
    fn get_current_time(&self) -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64()
    }

    /// Проверка rate limit
    pub fn check_rate_limit(&mut self, node_id: Option<&[u8]>) -> Result<bool, RhizomeError> {
        let current_time = self.get_current_time();

        // 1. Очищаем старые запросы
        self.cleanup_old_requests(current_time);

        // 2. Проверяем общий лимит
        let recent_requests = self.request_history.len();

        if recent_requests >= self.max_requests {
            warn!(
                requests = recent_requests,
                limit = self.max_requests,
                "Rate limit exceeded"
            );
            return Err(RhizomeError::Network(NetworkError::RateLimitError));
        }

        // 3. Проверяем per-node лимит
        if let Some(id) = node_id {
            let node_id_vec = id.to_vec();
            let node_history = self
                .node_requests
                .entry(node_id_vec.clone())
                .or_insert_with(|| VecDeque::with_capacity(self.per_node_limit * 2));

            let node_recent = node_history.len();

            if node_recent >= self.per_node_limit {
                let hex_id = hex::encode(&id[..id.len().min(8)]); // Первые 16 символов hex (8 байт)
                warn!(
                    node_id = %hex_id,
                    requests = node_recent,
                    limit = self.per_node_limit,
                    "Per-node rate limit exceeded"
                );
                return Err(RhizomeError::Network(NetworkError::RateLimitError));
            }

            // Добавляем в историю узла
            node_history.push_back(current_time);
        }

        // Запрос разрешен, добавляем в общую историю
        self.request_history.push_back(current_time);

        Ok(true)
    }

    /// Очистка старых запросов (аналог _cleanup_old_requests)
    fn cleanup_old_requests(&mut self, current_time: f64) {
        let window = self.window_seconds as f64;

        // Очищаем общую историю
        while let Some(&first_ts) = self.request_history.front() {
            if current_time - first_ts > window {
                self.request_history.pop_front();
            } else {
                break;
            }
        }

        // Очищаем историю по узлам
        // В Rust итерация по HashMap с удалением делается через retain
        self.node_requests.retain(|_, history| {
            while let Some(&first_ts) = history.front() {
                if current_time - first_ts > window {
                    history.pop_front();
                } else {
                    break;
                }
            }
            // Удаляем пустые истории (аналог del self.node_requests[node_id])
            !history.is_empty()
        });
    }

    #[allow(dead_code)]
    /// Получение статистики (аналог get_stats)
    pub fn get_stats(&mut self) -> HashMap<String, f64> {
        let current_time = self.get_current_time();
        self.cleanup_old_requests(current_time);

        let mut stats = HashMap::new();
        stats.insert(
            "recent_requests".to_string(),
            self.request_history.len() as f64,
        );
        stats.insert("max_requests".to_string(), self.max_requests as f64);
        stats.insert("window_seconds".to_string(), self.window_seconds as f64);
        stats.insert("active_nodes".to_string(), self.node_requests.len() as f64);

        stats
    }
}
