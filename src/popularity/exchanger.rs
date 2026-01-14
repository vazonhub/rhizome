use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::dht::node::Node;
use crate::network::protocol::NetworkProtocol;
use crate::popularity::metrics::{MetricsCollector, PopularityMetrics};
use crate::popularity::ranking::{PopularityRanker, RankedItem};
use crate::utils::time::get_now_f64;

/// Structure for exchange popularity nodes
pub struct PopularityExchanger {
    /// Protocol for Access to the UDP
    pub network_protocol: Arc<NetworkProtocol>,
    /// For track popular threads
    pub ranker: Arc<PopularityRanker>,
    /// Collector of metrics about threads
    pub metrics_collector: Option<Arc<RwLock<MetricsCollector>>>,
    /// Cache of best of the best threads
    global_ranking: RwLock<Vec<RankedItem>>,
    /// Last update of the global ranking
    global_ranking_updated: RwLock<f64>,
}

impl PopularityExchanger {
    pub fn new(
        network_protocol: Arc<NetworkProtocol>,
        ranker: Arc<PopularityRanker>,
        metrics_collector: Option<Arc<RwLock<MetricsCollector>>>,
    ) -> Self {
        Self {
            network_protocol,
            ranker,
            metrics_collector,
            global_ranking: RwLock::new(Vec::new()),
            global_ranking_updated: RwLock::new(0.0),
        }
    }

    /// Collect local metrics
    pub async fn get_local_metrics(&self) -> Option<HashMap<Vec<u8>, PopularityMetrics>> {
        let collector_lock = self.metrics_collector.as_ref()?;

        let collector = collector_lock.read().await;
        Some(collector.get_all_metrics().clone())
    }

    /// Exchange top-N elements with neighbor nodes
    pub async fn exchange_top_items(
        &self,
        local_metrics: HashMap<Vec<u8>, PopularityMetrics>,
        neighbor_nodes: Vec<Node>,
        top_n: usize,
    ) -> HashMap<Vec<u8>, PopularityMetrics> {
        let local_ranked = self.ranker.rank_items(&local_metrics, Some(top_n));

        let exchange_data: Vec<Value> = local_ranked
            .iter()
            .map(|item| {
                json!({
                    "key": hex::encode(&item.key),
                    "score": item.score,
                    "metrics": item.metrics.to_dict()
                })
            })
            .collect();

        if neighbor_nodes.is_empty() {
            return local_metrics;
        }

        let mut tasks = Vec::new();
        for _node in neighbor_nodes.iter().take(5) {
            tasks.push(exchange_data.clone());
        }

        let results = tasks;

        let mut updated_metrics = local_metrics;
        let mut received_count = 0;

        for result in results {
            let received_items = result;
            {
                received_count += received_items.len();
                for item_val in received_items {
                    if let Err(e) = self.process_single_item(&mut updated_metrics, item_val) {
                        warn!(error = %e, "Error processing received item during exchange");
                    }
                }
            }
        }

        info!(
            local_items = local_ranked.len(),
            neighbors = neighbor_nodes.len(),
            received_items = received_count,
            "Exchanged popularity data"
        );

        updated_metrics
    }

    /// Function for render one item
    fn process_single_item(
        &self,
        metrics_map: &mut HashMap<Vec<u8>, PopularityMetrics>,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key_hex = data["key"].as_str().ok_or("Missing key")?;
        let key = hex::decode(key_hex)?;
        let received_metrics_val = data.get("metrics").cloned().unwrap_or(Value::Null);

        if let Some(existing_metrics) = metrics_map.get_mut(&key) {
            let received_replication = received_metrics_val["replication_count"]
                .as_u64()
                .unwrap_or(1) as u32;
            existing_metrics.update_replication(received_replication);
        } else {
            let new_metrics = PopularityMetrics::from_dict(received_metrics_val)?;
            metrics_map.insert(key, new_metrics);
        }
        Ok(())
    }

    /// Press received items
    pub async fn process_received_items(&self, items: Vec<Value>) {
        let collector_lock = match &self.metrics_collector {
            Some(c) => c,
            None => return,
        };

        let mut collector = collector_lock.write().await;
        for item_data in items {
            if let Some(key_hex) = item_data["key"].as_str()
                && let Ok(key) = hex::decode(key_hex)
                && let Some(metrics) = collector.metrics.get_mut(&key)
            {
                let rep = item_data["metrics"]["replication_count"]
                    .as_u64()
                    .unwrap_or(1) as u32;
                metrics.update_replication(rep);
            }
        }
    }

    /// Aggregate Global Ranking
    pub async fn aggregate_global_ranking(
        &self,
        local_rankings: Vec<RankedItem>,
        seed_nodes: Vec<Node>,
    ) -> Vec<RankedItem> {
        let mut all_scores: HashMap<Vec<u8>, Vec<f64>> = HashMap::new();

        for item in &local_rankings {
            all_scores
                .entry(item.key.clone())
                .or_default()
                .push(item.score);
        }

        let mut tasks = Vec::new();

        for seed in seed_nodes.iter().take(10) {
            let net = self.network_protocol.clone();
            let seed_node = seed.clone();

            tasks.push(async move { net.get_global_ranking_remote(&seed_node).await });
        }

        let results = futures::future::join_all(tasks).await;

        for received_ranking in results.into_iter().flatten() {
            for item_val in received_ranking {
                if let (Some(key_hex), Some(score)) =
                    (item_val["key"].as_str(), item_val["score"].as_f64())
                    && let Ok(key) = hex::decode(key_hex)
                {
                    all_scores.entry(key).or_default().push(score);
                }
            }
        }

        let mut consensus_ranking = Vec::new();
        let collector = if let Some(c) = &self.metrics_collector {
            c.read().await
        } else {
            return Vec::new();
        };

        for (key, mut scores) in all_scores {
            if scores.is_empty() {
                continue;
            }

            scores.sort_by(|a, b| a.total_cmp(b));
            let median_score = scores[scores.len() / 2];

            if let Some(metrics) = collector.get_metrics(&key) {
                consensus_ranking.push(RankedItem {
                    key,
                    score: median_score,
                    metrics: metrics.clone(),
                });
            }
        }

        consensus_ranking.sort_by(|a, b| b.score.total_cmp(&a.score));
        consensus_ranking.truncate(100);

        let final_top = consensus_ranking.clone();

        *self.global_ranking.write().await = consensus_ranking;
        *self.global_ranking_updated.write().await = get_now_f64();

        info!(
            local_items = local_rankings.len(),
            seed_nodes = seed_nodes.len(),
            consensus_items = final_top.len(),
            "Aggregated global ranking"
        );

        final_top
    }

    /// Get global ranking in API format
    pub async fn get_global_ranking_api(&self) -> Vec<Value> {
        let ranking = self.global_ranking.read().await;

        ranking
            .iter()
            .map(|item| {
                json!({
                    "key": hex::encode(&item.key),
                    "score": item.score,
                    "metrics": item.metrics.to_dict()
                })
            })
            .collect()
    }
}
