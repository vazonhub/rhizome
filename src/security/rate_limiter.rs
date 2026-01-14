use std::collections::{HashMap, VecDeque};
use tracing::warn;

use crate::exceptions::{NetworkError, RhizomeError};
use crate::utils::time::get_now_f64;

/// Structure for limit messages peer some period of time
///
/// Use algo of Sliding Window
pub struct RateLimiter {
    /// Global limit of requests
    max_requests: usize,
    /// Size of sliding window
    window_seconds: u64,
    /// Limit for one node
    per_node_limit: usize,

    /// History of all requests (timestamps)
    request_history: VecDeque<f64>,

    /// Requests by node: NodeID -> deque of timestamps
    node_requests: HashMap<Vec<u8>, VecDeque<f64>>,
}

impl RateLimiter {
    /// Initialize rate limiter
    pub fn new(max_requests: usize, window_seconds: u64, per_node_limit: usize) -> Self {
        Self {
            max_requests,
            window_seconds,
            per_node_limit,
            request_history: VecDeque::with_capacity(max_requests * 2),
            node_requests: HashMap::new(),
        }
    }

    /// Check rate limit
    ///
    /// Main function which work with all requests and can block some requests if they do not fit
    pub fn check_rate_limit(&mut self, node_id: Option<&[u8]>) -> Result<bool, RhizomeError> {
        let current_time = get_now_f64();

        self.cleanup_old_requests(current_time);

        let recent_requests = self.request_history.len();

        if recent_requests >= self.max_requests {
            warn!(
                requests = recent_requests,
                limit = self.max_requests,
                "Rate limit exceeded"
            );
            return Err(RhizomeError::Network(NetworkError::RateLimitError));
        }

        if let Some(id) = node_id {
            let node_id_vec = id.to_vec();
            let node_history = self
                .node_requests
                .entry(node_id_vec.clone())
                .or_insert_with(|| VecDeque::with_capacity(self.per_node_limit * 2));

            let node_recent = node_history.len();

            if node_recent >= self.per_node_limit {
                let hex_id = hex::encode(&id[..id.len().min(8)]);
                warn!(
                    node_id = %hex_id,
                    requests = node_recent,
                    limit = self.per_node_limit,
                    "Per-node rate limit exceeded"
                );
                return Err(RhizomeError::Network(NetworkError::RateLimitError));
            }

            node_history.push_back(current_time);
        }

        self.request_history.push_back(current_time);

        Ok(true)
    }

    /// Cleanup old requests by window size in sliding window
    fn cleanup_old_requests(&mut self, current_time: f64) {
        let window = self.window_seconds as f64;

        while let Some(&first_ts) = self.request_history.front() {
            if current_time - first_ts > window {
                self.request_history.pop_front();
            } else {
                break;
            }
        }

        self.node_requests.retain(|_, history| {
            while let Some(&first_ts) = history.front() {
                if current_time - first_ts > window {
                    history.pop_front();
                } else {
                    break;
                }
            }

            !history.is_empty()
        });
    }

    /// Getting requests statistics for analyze
    pub fn get_stats(&mut self) -> HashMap<String, f64> {
        let current_time = get_now_f64();
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
