use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

use crate::config::Config;
use crate::node::base_node::{BaseNode, BaseNodePtrs};
use crate::utils::time::get_now_f64;

/// Seed-node for work with popularity
pub struct SeedNode {
    pub base: BaseNode,
}

#[allow(dead_code)]
impl SeedNode {
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        config.node.node_type = "seed".to_string();

        let base = BaseNode::new(config).await?;

        Ok(Self { base })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.base.start().await?;

        let base_ptrs = Arc::new(self.base.clone_ptrs());

        tokio::spawn(async move {
            Self::seed_loop(base_ptrs).await;
        });

        info!("Seed-specific tasks started");
        Ok(())
    }

    async fn seed_loop(node: Arc<BaseNodePtrs>) {
        let global_update_interval = node.config.popularity.global_update_interval as f64;
        let mut last_global_update = 0.0;

        while *node.is_running.read().await {
            let current_time = get_now_f64();

            if current_time - last_global_update >= global_update_interval {
                if let Err(e) = Self::update_global_ranking(&node).await {
                    error!(error = %e, "Error updating global ranking in seed task");
                }
                last_global_update = current_time;
            }

            sleep(Duration::from_secs(300)).await;
        }
    }

    async fn update_global_ranking(node: &BaseNodePtrs) -> Result<(), Box<dyn std::error::Error>> {
        let all_metrics = node
            .metrics_collector
            .read()
            .await
            .get_all_metrics()
            .clone();
        if all_metrics.is_empty() {
            return Ok(());
        }

        let local_ranked = node.popularity_ranker.rank_items(&all_metrics, Some(100));

        let mut seed_nodes = Vec::new();
        let all_nodes = node.routing_table.read().await.get_all_nodes();

        // TODO: фильтруем тех, кто похож на seed
        // (Например, по метаданным или отдельному бакету)
        for n in all_nodes {
            seed_nodes.push(n);
        }

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

/// Realization of Deref gives opportunity to call BaseNode methods in SeedNode
/// Example: seed_node.start() <-- seed_node.base.start()
impl Deref for SeedNode {
    type Target = BaseNode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
