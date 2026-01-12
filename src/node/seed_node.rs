use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{error, info};

use crate::config::Config;
use crate::node::base_node::{BaseNode, BaseNodePtrs};

/// Seed-узел с высокой доступностью и большим объемом хранилища
pub struct SeedNode {
    pub base: BaseNode,
}

#[allow(dead_code)]
impl SeedNode {
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Принудительно устанавливаем тип seed
        config.node.node_type = "seed".to_string();

        // Базовый узел сам настроит параметры на основе этого типа
        let base = BaseNode::new(config).await?;

        Ok(Self { base })
    }

    /// Переопределенный запуск узла
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Запускаем базовую логику (сеть, DHT, стандартные фоновые задачи)
        self.base.start().await?;

        // Клонируем указатели на компоненты для фоновой задачи Seed-узла
        // (Используем структуру BaseNodePtrs, которую мы определили в base_node.rs)
        let base_ptrs = Arc::new(self.base.clone_ptrs());

        // Запускаем специфичные для Seed-узла задачи
        tokio::spawn(async move {
            Self::seed_loop(base_ptrs).await;
        });

        info!("Seed-specific tasks started");
        Ok(())
    }

    /// Фоновый цикл для seed-узла
    async fn seed_loop(node: Arc<BaseNodePtrs>) {
        let global_update_interval = node.config.popularity.global_update_interval as f64;
        let mut last_global_update = 0.0;

        while *node.is_running.read().await {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();

            // Глобальное ранжирование каждые N часов (из конфига)
            if current_time - last_global_update >= global_update_interval {
                if let Err(e) = Self::update_global_ranking(&node).await {
                    error!(error = %e, "Error updating global ranking in seed task");
                }
                last_global_update = current_time;
            }

            // Проверяем каждые 5 минут
            sleep(Duration::from_secs(300)).await;
        }
    }

    /// Обновление глобального рейтинга
    async fn update_global_ranking(node: &BaseNodePtrs) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Получаем локальные метрики
        let all_metrics = node
            .metrics_collector
            .read()
            .await
            .get_all_metrics()
            .clone();
        if all_metrics.is_empty() {
            return Ok(());
        }

        // 2. Ранжируем локальные элементы
        let local_ranked = node.popularity_ranker.rank_items(&all_metrics, Some(100));

        // 3. Получаем список других seed-узлов из таблицы маршрутизации
        // В реальной системе здесь может быть фильтрация по типу узла
        let mut seed_nodes = Vec::new();
        let all_nodes = node.routing_table.read().await.get_all_nodes();

        // Временная логика (TODO из Python): фильтруем тех, кто похож на seed
        // (Например, по метаданным или отдельному бакету)
        for n in all_nodes {
            seed_nodes.push(n);
        }

        // 4. Агрегируем глобальный рейтинг через Exchanger
        let global_ranking = node
            .popularity_exchanger
            .aggregate_global_ranking(local_ranked, seed_nodes)
            .await;

        info!(
            items = global_ranking.len(),
            "Updated global ranking on seed node"
        );
        Ok(())
    }
}

/// Реализация Deref для доступа к методам BaseNode
impl Deref for SeedNode {
    type Target = BaseNode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
