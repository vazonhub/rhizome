use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Предполагаем, что PopularityMetrics находится в этом же крейте
use crate::popularity::metrics::PopularityMetrics;

/// Элемент с рейтингом популярности
#[derive(Debug, Clone)]
pub struct RankedItem {
    pub key: Vec<u8>,
    pub score: f64,
    pub metrics: PopularityMetrics,
}

// В Rust для f64 нет реализации Ord (из-за NaN),
// поэтому мы реализуем кастомную сортировку в методах ранжирования.

pub struct PopularityRanker {
    pub popularity_threshold: f64,
    pub active_threshold: f64,
}

impl PopularityRanker {
    pub fn new(popularity_threshold: f64, active_threshold: f64) -> Self {
        // Мы не сохраняем логгер как объект, так как в Rust
        // макросы tracing обращаются к глобальному подписчику.
        Self {
            popularity_threshold,
            active_threshold,
        }
    }

    /// Вспомогательный метод для получения текущего Unix timestamp в секундах
    fn get_now(&self) -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }

    /// Вычисление рейтинга популярности
    pub fn calculate_score(&self, metrics: &PopularityMetrics, adaptive_weights: bool) -> f64 {
        // Базовые веса
        let w_request_rate = 0.25;
        let w_replication_factor = 0.20;
        let mut w_freshness = 0.15;
        let w_audience_size = 0.10;
        let mut w_social_engagements = 0.20;
        let mut w_seed_coverage = 0.10;

        // Адаптивные веса в зависимости от возраста
        if adaptive_weights {
            let age_seconds = self.get_now() - metrics.first_seen;

            if age_seconds < 86400.0 {
                // Меньше 24 часов: приоритет свежести
                w_freshness = 0.30;
                w_social_engagements = 0.10;
                w_seed_coverage = 0.05;
            } else if age_seconds < 604800.0 {
                // Меньше 7 дней: приоритет соц. взаимодействий
                w_social_engagements = 0.30;
                w_freshness = 0.10;
                w_seed_coverage = 0.05;
            } else {
                // Больше 7 дней: приоритет устойчивости (seed coverage)
                w_seed_coverage = 0.25;
                w_social_engagements = 0.15;
                w_freshness = 0.05;
            }
        }

        // Нормализация метрик
        let norm = self.normalize_metrics(metrics);

        // Вычисление рейтинга
        let score = (norm.request_rate * w_request_rate
            + norm.replication_factor * w_replication_factor
            + norm.freshness * w_freshness
            + norm.audience_size * w_audience_size
            + norm.social_engagements * w_social_engagements
            + norm.seed_coverage * w_seed_coverage)
            * 10.0;

        // Ограничиваем диапазон 0.0 - 10.0 (аналог min(10.0, max(0.0, score)))
        score.clamp(0.0, 10.0)
    }

    /// Внутренняя структура для нормализованных данных (вместо Dict в Python)
    fn normalize_metrics(&self, metrics: &PopularityMetrics) -> NormalizedData {
        NormalizedData {
            request_rate: (metrics.request_rate / 100.0).min(1.0),
            replication_factor: (metrics.replication_count as f64 / 20.0).min(1.0),
            freshness: metrics.freshness_score,
            audience_size: (metrics.audience_size as f64 / 50.0).min(1.0),
            social_engagements: (metrics.social_engagements as f64 / 100.0).min(1.0),
            seed_coverage: metrics.seed_coverage,
        }
    }

    /// Ранжирование элементов
    pub fn rank_items(
        &self,
        metrics_dict: &HashMap<Vec<u8>, PopularityMetrics>,
        limit: Option<usize>,
    ) -> Vec<RankedItem> {
        let mut ranked_items: Vec<RankedItem> = metrics_dict
            .iter()
            .map(|(key, metrics)| {
                let score = self.calculate_score(metrics, true);
                RankedItem {
                    key: key.clone(),
                    score,
                    metrics: metrics.clone(),
                }
            })
            .collect();

        // Сортировка по убыванию score.
        // Используем total_cmp, чтобы корректно обрабатывать f64.
        ranked_items.sort_by(|a, b| b.score.total_cmp(&a.score));

        if let Some(l) = limit {
            ranked_items.truncate(l);
        }

        ranked_items
    }

    /// Получение популярных элементов (выше порога популярности)
    pub fn get_popular_items(
        &self,
        metrics_dict: &HashMap<Vec<u8>, PopularityMetrics>,
        limit: usize,
    ) -> Vec<RankedItem> {
        let ranked = self.rank_items(metrics_dict, None);
        let mut popular: Vec<RankedItem> = ranked
            .into_iter()
            .filter(|item| item.score >= self.popularity_threshold)
            .collect();

        popular.truncate(limit);
        popular
    }

    /// Получение активных элементов (выше порога активности)
    pub fn get_active_items(
        &self,
        metrics_dict: &HashMap<Vec<u8>, PopularityMetrics>,
        limit: usize,
    ) -> Vec<RankedItem> {
        let ranked = self.rank_items(metrics_dict, None);
        let mut active: Vec<RankedItem> = ranked
            .into_iter()
            .filter(|item| item.score >= self.active_threshold)
            .collect();

        active.truncate(limit);
        active
    }
}

/// Вспомогательная структура для нормализованных значений (без аллокаций в куче)
struct NormalizedData {
    request_rate: f64,
    replication_factor: f64,
    freshness: f64,
    audience_size: f64,
    social_engagements: f64,
    seed_coverage: f64,
}
