use crate::config::Config;
use crate::node::base_node::BaseNode;
use std::ops::Deref;

/// Mobile-node for mobile device
pub struct MobileNode {
    // Используем композицию
    pub base: BaseNode,
}

#[allow(dead_code)]
impl MobileNode {
    /// Constructor for node of full type
    ///
    /// Guarantied that node type is mobile, max storage is 100mb and max buckets count is 10.
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        config.node.node_type = "mobile".to_string();

        let max_mobile_bytes: u64 = 100 * 1024 * 1024;
        config.storage.max_storage_size = config.storage.max_storage_size.min(max_mobile_bytes);

        config.dht.k = 10;

        let base = BaseNode::new(config).await?;

        Ok(Self { base })
    }
}

/// Realization of Deref gives opportunity to call BaseNode methods in MobileNode
/// Example: mobile_node.start() <-- mobile_node.base.start()
impl Deref for MobileNode {
    type Target = BaseNode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
