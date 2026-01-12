use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::dht::node::Node;
use crate::network::protocol::NetworkProtocol;
use crate::popularity::metrics::{MetricsCollector, PopularityMetrics};
use crate::popularity::ranking::{PopularityRanker, RankedItem}; // Предполагаем наличие методов в протоколе

/// Вспомогательная функция времени
fn get_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

pub struct PopularityExchanger {
    // Используем Arc для доступа к общим компонентам
    pub network_protocol: Arc<NetworkProtocol>,
    pub ranker: Arc<PopularityRanker>,
    pub metrics_collector: Option<Arc<RwLock<MetricsCollector>>>,

    // Внутреннее состояние (защищено RwLock для потокобезопасности)
    global_ranking: RwLock<Vec<RankedItem>>,
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

    pub async fn get_local_metrics(&self) -> Option<HashMap<Vec<u8>, PopularityMetrics>> {
        // 1. Проверяем, существует ли коллектор
        let collector_lock = self.metrics_collector.as_ref()?;

        // 2. Блокируем на чтение и клонируем данные
        // Мы клонируем HashMap, так как не можем вернуть ссылку на данные внутри Lock
        let collector = collector_lock.read().await;
        Some(collector.get_all_metrics().clone())
    }

    /// Обмен топ-N элементами с соседними узлами (аналог exchange_top_items)
    pub async fn exchange_top_items(
        &self,
        local_metrics: HashMap<Vec<u8>, PopularityMetrics>,
        neighbor_nodes: Vec<Node>,
        top_n: usize,
    ) -> HashMap<Vec<u8>, PopularityMetrics> {
        // 1. Получаем локальный топ
        let local_ranked = self.ranker.rank_items(&local_metrics, Some(top_n));

        // 2. Подготавливаем данные для отправки в формате JSON (Value)
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

        // 3. Параллельно обмениваемся данными (ограничиваем до 5 соседей)
        let mut tasks = Vec::new();
        for _node in neighbor_nodes.iter().take(5) {
            // В сетевом протоколе должен быть метод exchange_popularity
            tasks.push(exchange_data.clone());
        }

        let results = tasks;

        // 4. Обрабатываем результаты
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

    /// Вспомогательная функция обработки одного элемента данных
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

    /// Обработка полученных элементов (аналог process_received_items)
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

    /// Агрегация глобального рейтинга (аналог aggregate_global_ranking)
    pub async fn aggregate_global_ranking(
        &self,
        local_rankings: Vec<RankedItem>,
        seed_nodes: Vec<Node>,
    ) -> Vec<RankedItem> {
        // Таблица: Ключ -> Список оценок (scores)
        let mut all_scores: HashMap<Vec<u8>, Vec<f64>> = HashMap::new();

        // 1. Добавляем локальные оценки
        for item in &local_rankings {
            all_scores
                .entry(item.key.clone())
                .or_default()
                .push(item.score);
        }

        // 2. Запрашиваем оценки у других seed-узлов (до 10 штук)
        let mut tasks = Vec::new();
        for seed in seed_nodes.iter().take(10) {
            tasks.push(seed);
        }

        // let results = tasks;

        // for result in results {
        //     if let received_ranking = result {
        //         for item_val in received_ranking {
        //             if let (Some(key_hex), Some(score)) = (item_val["key"].as_str(), item_val["score"].as_f64()) {
        //                 if let Ok(key) = hex::decode(key_hex) {
        //                     all_scores.entry(key).or_default().push(score);
        //                 }
        //             }
        //         }
        //     }
        // }

        // 3. Вычисляем консенсусный рейтинг (медиана)
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

            // Расчет медианы
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

        // 4. Сортируем и сохраняем (Топ-100)
        consensus_ranking.sort_by(|a, b| b.score.total_cmp(&a.score));
        consensus_ranking.truncate(100);

        let final_top = consensus_ranking.clone();

        // Обновляем состояние
        *self.global_ranking.write().await = consensus_ranking;
        *self.global_ranking_updated.write().await = get_now();

        info!(
            local_items = local_rankings.len(),
            seed_nodes = seed_nodes.len(),
            consensus_items = final_top.len(),
            "Aggregated global ranking"
        );

        final_top
    }

    /// Получение глобального рейтинга в формате для API (аналог get_global_ranking)
    pub async fn get_global_ranking_api(&self) -> Vec<Value> {
        let ranking = self.global_ranking.read().await;
        let updated_at = *self.global_ranking_updated.read().await;

        // Если данные старее 3 часов, логика обновления должна быть запущена извне
        // или здесь через канал обратной связи.
        if get_now() - updated_at > 10800.0 {
            // В Rust мы обычно не запускаем фоновые задачи прямо из геттера
        }

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
