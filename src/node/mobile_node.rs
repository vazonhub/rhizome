use crate::config::Config;
use crate::node::base_node::BaseNode;
use std::ops::Deref;

/// Mobile-узел для мобильных устройств (максимально легкий клиент)
pub struct MobileNode {
    // Используем композицию
    pub base: BaseNode,
}

#[allow(dead_code)]
impl MobileNode {
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Принудительно устанавливаем тип mobile
        config.node.node_type = "mobile".to_string();

        // 2. Сильно ограничиваем ресурсы хранилища
        // 100 MB максимум
        let max_mobile_bytes: u64 = 100 * 1024 * 1024;
        config.storage.max_storage_size = config.storage.max_storage_size.min(max_mobile_bytes);

        // 3. Упрощаем параметры DHT (снижаем нагрузку на CPU и сеть)
        // Меньше узлов в бакетах (k=10)
        config.dht.k = 10;

        // 4. Инициализируем базовый узел
        let base = BaseNode::new(config).await?;

        Ok(Self { base })
    }
}

/// Реализация Deref для доступа к методам базового узла
impl Deref for MobileNode {
    type Target = BaseNode;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
