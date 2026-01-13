use crate::config::Config;
use crate::node::base_node::BaseNode;
use std::ops::Deref;

/// Light-node systems with limited resources
pub struct LightNode {
    // Композиция вместо наследования
    pub base: BaseNode,
}

#[allow(dead_code)]
impl LightNode {
    /// Constructor for node of light type
    ///
    /// Guarantied that node type is light and max_storage_bytes is 1GB.
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        config.node.node_type = "light".to_string();

        let max_light_bytes: u64 = 1024 * 1024 * 1024; // 1 GB
        if config.storage.max_storage_size > max_light_bytes {
            config.storage.max_storage_size = max_light_bytes;
        }

        let base = BaseNode::new(config).await?;

        Ok(Self { base })
    }
}

/// Realization of Deref gives opportunity to call BaseNode methods in LightNode
/// Example: light_node.start() <-- light_node.base.start()
impl Deref for LightNode {
    type Target = BaseNode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
