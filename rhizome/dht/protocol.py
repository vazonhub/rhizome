"""
Протокол DHT операций
"""

import asyncio
from typing import TYPE_CHECKING, Any, Dict, List, Optional

from rhizome.dht.node import Node, NodeID
from rhizome.dht.routing_table import RoutingTable
from rhizome.exceptions import DHTError, NodeNotFoundError, ValueNotFoundError
from rhizome.logger import get_logger
from rhizome.utils.crypto import compute_distance

if TYPE_CHECKING:
    from rhizome.network.protocol import NetworkProtocol
    from rhizome.storage.storage import Storage


class DHTProtocol:
    """Протокол для операций DHT"""

    def __init__(
        self,
        routing_table: RoutingTable,
        storage: "Storage",
        network_protocol: Optional["NetworkProtocol"] = None,
    ):
        self.routing_table = routing_table
        self.storage = storage
        self.network_protocol = network_protocol
        self.alpha = 3  # Количество параллельных запросов
        self.logger = get_logger("dht.protocol")

    async def ping(self, node: Node) -> bool:
        """
        Проверка доступности узла

        Args:
            node: Узел для проверки

        Returns:
            True если узел доступен, False иначе
        """
        if self.network_protocol:
            # Используем сетевой протокол для реального ping
            result = await self.network_protocol.ping(node)
            if result:
                node.update_seen()
            else:
                node.record_failed_ping()
            return result
        else:
            # Заглушка для тестирования
            await asyncio.sleep(0.1)
            node.update_seen()
            return True

    async def find_node(self, target_id: NodeID) -> List[Node]:
        """
        Поиск узлов по идентификатору (Kademlia lookup алгоритм)

        Args:
            target_id: Целевой Node ID

        Returns:
            Список ближайших узлов
        """
        # Начинаем с ближайших узлов из routing table
        closest = self.routing_table.find_closest_nodes(target_id, self.alpha)
        seen_nodes = {node.node_id: node for node in closest}

        if not self.network_protocol:
            return closest

        # Итеративный поиск: запрашиваем у найденных узлов еще более близкие
        queried = set()
        while True:
            # Выбираем alpha ближайших не опрошенных узлов
            candidates = [node for node in closest if node.node_id not in queried][: self.alpha]

            if not candidates:
                break

            # Параллельно запрашиваем у выбранных узлов
            tasks = [self.network_protocol.find_node(target_id, node) for node in candidates]

            results = await asyncio.gather(*tasks, return_exceptions=True)

            # Добавляем найденные узлы
            new_nodes_found = False
            for result in results:
                if isinstance(result, list):
                    for node in result:
                        if node.node_id not in seen_nodes:
                            seen_nodes[node.node_id] = node
                            new_nodes_found = True

            # Отмечаем узлы как опрошенные
            for node in candidates:
                queried.add(node.node_id)

            # Обновляем список ближайших
            closest = sorted(
                seen_nodes.values(), key=lambda n: compute_distance(target_id.id, n.node_id.id)
            )[: self.alpha]

            # Если не нашли новых узлов, завершаем
            if not new_nodes_found:
                break

        return closest[: self.alpha]

    async def find_value(self, key: bytes) -> Optional[bytes]:
        """
        Поиск значения по ключу (Kademlia алгоритм)

        Args:
            key: Ключ для поиска

        Returns:
            Значение или None если не найдено
        """
        # Сначала проверяем локальное хранилище
        value = await self.storage.get(key)
        if value is not None:
            return value

        if not self.network_protocol:
            raise ValueNotFoundError(f"Value not found for key: {key.hex()}")

        # Ищем в DHT
        key_hash = key[:20] if len(key) >= 20 else key + b"\x00" * (20 - len(key))
        target_id = NodeID(id=key_hash[:20])

        # Начинаем с ближайших узлов из routing table
        closest = self.routing_table.find_closest_nodes(target_id, self.alpha)
        seen_nodes = {node.node_id: node for node in closest}
        queried = set()

        # Итеративный поиск
        while True:
            # Выбираем alpha ближайших не опрошенных узлов
            candidates = [node for node in closest if node.node_id not in queried][: self.alpha]

            if not candidates:
                break

            # Параллельно запрашиваем у выбранных узлов
            tasks = [self.network_protocol.find_value(key, node) for node in candidates]

            results = await asyncio.gather(*tasks, return_exceptions=True)

            # Проверяем результаты
            found_value = None
            for i, result in enumerate(results):
                if isinstance(result, bytes):
                    # Нашли значение!
                    found_value = result
                    break
                elif isinstance(result, Exception):
                    # Ошибка при запросе, пропускаем этот узел
                    continue

            if found_value is not None:
                return found_value

            # Если значение не найдено, запрашиваем узлы через find_node
            # для расширения поиска
            for node in candidates:
                try:
                    found_nodes = await self.network_protocol.find_node(target_id, node)
                    for found_node in found_nodes:
                        if found_node.node_id not in seen_nodes:
                            seen_nodes[found_node.node_id] = found_node
                except Exception:
                    # Игнорируем ошибки при поиске узлов
                    pass

            # Отмечаем узлы как опрошенные
            for node in candidates:
                queried.add(node.node_id)

            # Обновляем список ближайших
            closest = sorted(
                seen_nodes.values(), key=lambda n: compute_distance(target_id.id, n.node_id.id)
            )[: self.alpha]

            # Если не нашли новых узлов, завершаем
            if len(queried) >= len(seen_nodes):
                break

        # Значение не найдено
        raise ValueNotFoundError(f"Value not found for key: {key.hex()}")

    async def store(self, key: bytes, value: bytes, ttl: int = 86400) -> bool:
        """
        Сохранение данных на k ближайших узлах (Kademlia STORE)

        Args:
            key: Ключ
            value: Значение
            ttl: Time to live в секундах

        Returns:
            True если успешно сохранено хотя бы на одном узле
        """
        # Сохраняем локально
        await self.storage.put(key, value, ttl)

        if not self.network_protocol:
            return True

        # Находим k ближайших узлов
        key_hash = key[:20] if len(key) >= 20 else key + b"\x00" * (20 - len(key))
        target_id = NodeID(id=key_hash[:20])

        # Используем find_node для получения ближайших узлов
        closest_nodes = await self.find_node(target_id)

        # Если не нашли узлов через find_node, используем routing table
        if not closest_nodes:
            closest_nodes = self.routing_table.find_closest_nodes(target_id, self.routing_table.k)

        # Сохраняем на найденных узлах параллельно
        if closest_nodes:
            tasks = [
                self.network_protocol.store(key, value, ttl, node)
                for node in closest_nodes[: self.routing_table.k]
            ]

            results = await asyncio.gather(*tasks, return_exceptions=True)

            # Подсчитываем успешные сохранения
            success_count = sum(1 for r in results if r is True)
            self.logger.debug(
                "STORE completed",
                key=key.hex()[:16],
                nodes_attempted=len(tasks),
                nodes_success=success_count,
            )

            return success_count > 0

        # Данные успешно сохранены локально, даже если нет других узлов для репликации
        return True
