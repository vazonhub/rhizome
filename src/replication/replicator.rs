use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::dht::protocol::DHTProtocol;
use crate::popularity::ranking::RankedItem;
use crate::storage::main::Storage;

pub struct Replicator {
    dht_protocol: Arc<DHTProtocol>,
    storage: Arc<Storage>,
    min_replication_factor: usize,
    popular_replication_factor: usize,
}

impl Replicator {
    pub fn new(
        dht_protocol: Arc<DHTProtocol>,
        storage: Arc<Storage>,
        min_replication_factor: usize,
        popular_replication_factor: usize,
    ) -> Self {
        Self {
            dht_protocol,
            storage,
            min_replication_factor,
            popular_replication_factor,
        }
    }

    /// Репликация популярных элементов
    pub async fn replicate_popular_items(
        &self,
        ranked_items: Vec<RankedItem>,
        popularity_threshold: f64,
    ) -> HashMap<Vec<u8>, bool> {
        let mut results = HashMap::new();

        // Фильтруем популярные элементы
        let popular_items: Vec<&RankedItem> = ranked_items
            .iter()
            .filter(|item| item.score >= popularity_threshold)
            .collect();

        info!(
            total_items = ranked_items.len(),
            popular_items = popular_items.len(),
            "Starting replication"
        );

        for item in popular_items {
            let key = &item.key;
            let key_hex = hex::encode(&key[..key.len().min(8)]);

            // Получаем значение из хранилища
            let value_result = self.storage.get(key.clone()).await;

            match value_result {
                Ok(Some(value)) => {
                    // Проверяем текущий коэффициент репликации из метрик
                    let current_replication = item.metrics.replication_count as usize;
                    let target_replication = self.popular_replication_factor;

                    if current_replication >= target_replication {
                        // Уже достаточно репликаций
                        results.insert(key.clone(), true);
                        continue;
                    }

                    // Реплицируем через STORE (TTL 30 дней)
                    let ttl = 2592000;
                    match self.dht_protocol.store(key, &value, ttl).await {
                        Ok(success) => {
                            results.insert(key.clone(), success);
                            if success {
                                debug!(
                                    key = %key_hex,
                                    score = item.score,
                                    target_replication = target_replication,
                                    "Replicated popular item"
                                );
                            } else {
                                warn!(key = %key_hex, "Replication failed");
                            }
                        }
                        Err(e) => {
                            error!(key = %key_hex, error = %e, "Error during STORE in replication");
                            results.insert(key.clone(), false);
                        }
                    }
                }
                Ok(None) => {
                    warn!(key = %key_hex, "Value not found for replication");
                    results.insert(key.clone(), false);
                }
                Err(e) => {
                    error!(key = %key_hex, error = %e, "Error accessing storage for replication");
                    results.insert(key.clone(), false);
                }
            }
        }

        let successful = results.values().filter(|&&v| v).count();
        info!(
            total = results.len(),
            successful = successful,
            failed = results.len() - successful,
            "Replication completed"
        );

        results
    }

    /// Обеспечение минимальной репликации для списка ключей
    pub async fn ensure_minimal_replication(
        &self,
        keys: Vec<Vec<u8>>,
        min_factor: Option<usize>,
    ) -> HashMap<Vec<u8>, bool> {
        let _target_factor = min_factor.unwrap_or(self.min_replication_factor);
        let mut results = HashMap::new();

        for key in keys {
            let _key_hex = hex::encode(&key[..key.len().min(8)]);

            match self.storage.get(key.clone()).await {
                Ok(Some(value)) => {
                    // Выполняем STORE для обеспечения наличия данных (TTL 1 день)
                    match self.dht_protocol.store(&key, &value, 86400).await {
                        Ok(success) => results.insert(key, success),
                        Err(_) => results.insert(key, false),
                    };
                }
                _ => {
                    results.insert(key, false);
                }
            }
        }

        results
    }

    /// Экстренная репликация при обнаружении потери узла
    pub async fn emergency_replication(&self, key: Vec<u8>, value: Vec<u8>) -> bool {
        let key_hex = hex::encode(&key[..key.len().min(8)]);
        warn!(key = %key_hex, "Emergency replication triggered");

        // Реплицируем с высоким приоритетом и TTL 30 дней
        let ttl = 2592000;
        match self.dht_protocol.store(&key, &value, ttl).await {
            Ok(true) => {
                info!(key = %key_hex, "Emergency replication successful");
                true
            }
            Ok(false) => {
                error!(key = %key_hex, "Emergency replication failed (store returned false)");
                false
            }
            Err(e) => {
                error!(key = %key_hex, error = %e, "Error in emergency replication");
                false
            }
        }
    }
}
