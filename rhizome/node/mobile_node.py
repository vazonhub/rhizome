"""
Mobile-узел (максимально легкий клиент)
"""

from rhizome.config import Config
from rhizome.node.base_node import BaseNode


class MobileNode(BaseNode):
    """Mobile-узел для мобильных устройств"""
    
    def __init__(self, config: Config):
        # Принудительно устанавливаем тип mobile
        config.node.node_type = "mobile"
        super().__init__(config)
        
        # Сильно ограничиваем ресурсы для mobile-узла
        self.config.storage.max_storage_size = min(
            self.config.storage.max_storage_size,
            100 * 1024 * 1024  # 100 MB максимум
        )
        
        # Упрощенный протокол
        self.config.dht.k = 10  # Меньше узлов в бакетах

