"""
Full-узел (основная рабочая нагрузка)
"""

from rhizome.config import Config
from rhizome.node.base_node import BaseNode


class FullNode(BaseNode):
    """Full-узел для основной рабочей нагрузки"""
    
    def __init__(self, config: Config):
        # Принудительно устанавливаем тип full
        config.node.node_type = "full"
        super().__init__(config)

