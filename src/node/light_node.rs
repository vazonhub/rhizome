use crate::config::Config;
use crate::node::base_node::BaseNode;
use std::ops::Deref;

/// Light-узел для систем с ограниченными ресурсами
pub struct LightNode {
    // Композиция вместо наследования
    pub base: BaseNode,
}

#[allow(dead_code)]
impl LightNode {
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Принудительно устанавливаем тип light
        config.node.node_type = "light".to_string();

        // 2. Ограничиваем размер хранилища (минимум между текущим и 1 ГБ)
        let max_light_bytes: u64 = 1024 * 1024 * 1024; // 1 GB
        if config.storage.max_storage_size > max_light_bytes {
            config.storage.max_storage_size = max_light_bytes;
        }

        // 3. Инициализируем базовый узел с обновленным конфигом
        let base = BaseNode::new(config).await?;

        Ok(Self { base })
    }
}

/// Позволяет прозрачно обращаться к методам BaseNode (start, stop, find_value и т.д.)
impl Deref for LightNode {
    type Target = BaseNode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
