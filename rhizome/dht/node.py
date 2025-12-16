"""
Представление узла в DHT
"""

from dataclasses import dataclass
from typing import Optional
import time

from rhizome.utils.crypto import compute_distance


@dataclass
class NodeID:
    """160-битный идентификатор узла"""
    id: bytes  # 20 байт (160 бит)
    
    def __post_init__(self):
        if len(self.id) != 20:
            raise ValueError("Node ID must be exactly 20 bytes (160 bits)")
    
    def distance_to(self, other: "NodeID") -> bytes:
        """Вычисление XOR-расстояния до другого узла"""
        return compute_distance(self.id, other.id)
    
    def __eq__(self, other):
        if not isinstance(other, NodeID):
            return False
        return self.id == other.id
    
    def __hash__(self):
        return hash(self.id)
    
    def __repr__(self):
        return f"NodeID({self.id.hex()[:16]}...)"


@dataclass
class Node:
    """Узел в сети"""
    node_id: NodeID
    address: str
    port: int
    last_seen: float = 0.0
    failed_pings: int = 0
    
    def __post_init__(self):
        if self.last_seen == 0.0:
            self.last_seen = time.time()
    
    def update_seen(self):
        """Обновление времени последнего контакта"""
        self.last_seen = time.time()
        self.failed_pings = 0
    
    def record_failed_ping(self):
        """Запись неудачного ping"""
        self.failed_pings += 1
    
    def is_stale(self, timeout: float = 3600.0) -> bool:
        """Проверка, является ли узел устаревшим"""
        return (time.time() - self.last_seen) > timeout
    
    def __eq__(self, other):
        if not isinstance(other, Node):
            return False
        return self.node_id == other.node_id
    
    def __hash__(self):
        return hash(self.node_id)
    
    def __repr__(self):
        return f"Node({self.node_id}, {self.address}:{self.port})"

