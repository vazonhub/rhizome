"""
Light-узел (ограниченные ресурсы)
"""

from rhizome.config import Config
from rhizome.node.base_node import BaseNode


class LightNode(BaseNode):
    """Light-узел с ограниченными ресурсами"""

    def __init__(self, config: Config):
        # Принудительно устанавливаем тип light
        config.node.node_type = "light"
        super().__init__(config)

        # Ограничиваем размер хранилища для light-узла
        self.config.storage.max_storage_size = min(
            self.config.storage.max_storage_size, 1024 * 1024 * 1024  # 1 GB максимум
        )
