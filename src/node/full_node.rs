use crate::config::Config;
use crate::node::base_node::BaseNode;
use std::ops::Deref;

/// Full-узел для основной рабочей нагрузки
pub struct FullNode {
    // Используем композицию вместо наследования
    pub base: BaseNode,
}

impl FullNode {
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Принудительно устанавливаем тип full в конфигурации
        config.node.node_type = "full".to_string();

        // Инициализируем базовый узел
        let base = BaseNode::new(config).await?;

        Ok(Self { base })
    }
}

/// Реализация Deref позволяет вызывать методы BaseNode прямо у FullNode
/// Например: full_node.start() вместо full_node.base.start()
impl Deref for FullNode {
    type Target = BaseNode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
