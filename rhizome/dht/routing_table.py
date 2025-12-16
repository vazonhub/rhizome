"""
Таблица маршрутизации Kademlia (k-бакеты)
"""

import time
from typing import List, Optional

from rhizome.dht.node import Node, NodeID
from rhizome.utils.crypto import compute_distance


class KBucket:
    """k-бакет для хранения узлов на определенном расстоянии"""

    def __init__(self, k: int = 20):
        self.k = k
        self.nodes: List[Node] = []
        self.last_updated = time.time()

    def add_node(self, node: Node) -> bool:
        """
        Добавление узла в бакет

        Returns:
            True если узел добавлен, False если бакет полон
        """
        # Если узел уже есть, перемещаем его в конец (LRU)
        if node in self.nodes:
            self.nodes.remove(node)
            self.nodes.append(node)
            self.last_updated = time.time()
            return True

        # Если есть место, добавляем
        if len(self.nodes) < self.k:
            self.nodes.append(node)
            self.last_updated = time.time()
            return True

        # Бакет полон
        return False

    def remove_node(self, node: Node):
        """Удаление узла из бакета"""
        if node in self.nodes:
            self.nodes.remove(node)
            self.last_updated = time.time()

    def get_nodes(self, limit: Optional[int] = None) -> List[Node]:
        """Получение узлов из бакета"""
        nodes = self.nodes
        if limit:
            nodes = nodes[:limit]
        return nodes

    def is_full(self) -> bool:
        """Проверка, полон ли бакет"""
        return len(self.nodes) >= self.k


class RoutingTable:
    """Таблица маршрутизации Kademlia"""

    def __init__(self, node_id: NodeID, k: int = 20, bucket_count: int = 160):
        self.node_id = node_id
        self.k = k
        self.buckets: List[KBucket] = [KBucket(k) for _ in range(bucket_count)]

    def _get_bucket_index(self, target_id: NodeID) -> int:
        """Получение индекса бакета для целевого ID"""
        distance = compute_distance(self.node_id.id, target_id.id)

        # Находим первый ненулевой бит (слева направо)
        for i, byte in enumerate(distance):
            if byte != 0:
                # Находим позицию первого ненулевого бита в байте
                for bit in range(8):
                    if byte & (0x80 >> bit):
                        return i * 8 + bit

        # Если расстояние равно 0, это сам узел
        return len(self.buckets) - 1

    def add_node(self, node: Node) -> bool:
        """Добавление узла в таблицу маршрутизации"""
        if node.node_id == self.node_id:
            return False  # Не добавляем себя

        bucket_index = self._get_bucket_index(node.node_id)
        bucket = self.buckets[bucket_index]

        if bucket.is_full():
            # Если бакет полон, проверяем, есть ли устаревшие узлы
            stale_nodes = [n for n in bucket.nodes if n.is_stale()]
            if stale_nodes:
                # Заменяем устаревший узел
                bucket.remove_node(stale_nodes[0])
                return bucket.add_node(node)
            return False

        return bucket.add_node(node)

    def remove_node(self, node: Node):
        """Удаление узла из таблицы маршрутизации"""
        bucket_index = self._get_bucket_index(node.node_id)
        self.buckets[bucket_index].remove_node(node)

    def find_closest_nodes(self, target_id: NodeID, count: int) -> List[Node]:
        """Поиск ближайших узлов к целевому ID"""
        bucket_index = self._get_bucket_index(target_id)
        closest_nodes: List[Node] = []

        # Собираем узлы из текущего бакета и соседних
        for offset in range(len(self.buckets)):
            idx = (bucket_index + offset) % len(self.buckets)
            nodes = self.buckets[idx].get_nodes()
            closest_nodes.extend(nodes)

            if len(closest_nodes) >= count:
                break

        # Сортируем по расстоянию
        closest_nodes.sort(key=lambda n: compute_distance(n.node_id.id, target_id.id))

        return closest_nodes[:count]

    def get_all_nodes(self) -> List[Node]:
        """Получение всех узлов из таблицы"""
        all_nodes = []
        for bucket in self.buckets:
            all_nodes.extend(bucket.nodes)
        return all_nodes
