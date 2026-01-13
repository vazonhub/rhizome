use crate::config::Config;
use crate::node::base_node::BaseNode;
use std::ops::Deref;

/// Full-node for main work load
pub struct FullNode {
    // Используем композицию вместо наследования
    pub base: BaseNode,
}

impl FullNode {
    /// Constructor for node of full type
    ///
    /// Guarantied that node type is full and has all node conditions without any restrictions
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        config.node.node_type = "full".to_string();

        let base = BaseNode::new(config).await?;

        Ok(Self { base })
    }
}

/// Realization of Deref gives opportunity to call BaseNode methods in FullNode
/// Example: full_node.start() <-- full_node.base.start()
impl Deref for FullNode {
    type Target = BaseNode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
