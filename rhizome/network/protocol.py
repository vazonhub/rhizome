"""
Сетевой протокол для обмена сообщениями DHT
"""

import asyncio
import time
from typing import Any, Dict, List, Optional, Tuple

import msgpack

from rhizome.dht.node import Node, NodeID
from rhizome.exceptions import NetworkError, RateLimitError
from rhizome.logger import get_logger
from rhizome.network.transport import Message, UDPTransport
from rhizome.security.rate_limiter import RateLimiter


class NetworkProtocol:
    """Протокол для сетевого обмена DHT сообщениями"""

    # Типы сообщений
    MSG_PING = 0x01
    MSG_PONG = 0x02
    MSG_FIND_NODE = 0x03
    MSG_FIND_NODE_RESPONSE = 0x04
    MSG_FIND_VALUE = 0x05
    MSG_FIND_VALUE_RESPONSE = 0x06
    MSG_STORE = 0x07
    MSG_STORE_RESPONSE = 0x08
    MSG_POPULARITY_EXCHANGE = 0x09
    MSG_POPULARITY_EXCHANGE_RESPONSE = 0x0A
    MSG_GLOBAL_RANKING_REQUEST = 0x0B
    MSG_GLOBAL_RANKING_RESPONSE = 0x0C

    def __init__(
        self,
        transport: UDPTransport,
        node_id: NodeID,
        local_address: Tuple[str, int],
        routing_table: Optional[Any] = None,
        storage: Optional[Any] = None,
        popularity_exchanger: Optional[Any] = None,
    ):
        self.transport = transport
        self.node_id = node_id
        self.local_address = local_address
        self.routing_table = routing_table
        self.storage = storage
        self.popularity_exchanger = popularity_exchanger
        self.logger = get_logger("network.protocol")

        # Rate limiter для защиты от атак
        self.rate_limiter = RateLimiter(max_requests=100, window_seconds=60, per_node_limit=20)

        # Ожидающие ответы запросы
        self.pending_requests: Dict[bytes, asyncio.Future] = {}
        self.request_timeout = 10.0

    async def start(self):
        """Запуск протокола"""
        await self.transport.start(self._handle_message)
        self.logger.info("Network protocol started")

    async def stop(self):
        """Остановка протокола"""
        await self.transport.stop()
        self.logger.info("Network protocol stopped")

    async def _handle_message(self, message: Message):
        """Обработка входящего сообщения"""
        try:
            # Распаковываем сообщение для проверки rate limit
            try:
                msg_data = msgpack.unpackb(message.data, raw=False)
                node_id = msg_data.get("node_id")

                # Проверяем rate limit
                self.rate_limiter.check_rate_limit(node_id)
            except RateLimitError as e:
                self.logger.warning("Rate limit exceeded", address=message.address, error=str(e))
                return  # Игнорируем запрос при превышении лимита
            except Exception:
                # Если не удалось распаковать, пропускаем rate limit
                pass

            msg_type, msg_id, payload = self._unpack_message(message.data)

            # Обработка ответов на запросы
            if msg_id in self.pending_requests:
                future = self.pending_requests.pop(msg_id)
                if not future.done():
                    future.set_result((msg_type, payload))
                return

            # Обработка входящих запросов
            await self._handle_request(msg_type, msg_id, payload, message.address)

        except Exception as e:
            self.logger.error("Error handling message", error=str(e), exc_info=True)

    def _pack_message(self, msg_type: int, msg_id: bytes, payload: Dict[str, Any]) -> bytes:
        """Упаковка сообщения"""
        message = {
            "type": msg_type,
            "id": msg_id,
            "node_id": self.node_id.id,
            "payload": payload,
            "timestamp": time.time(),
        }
        return msgpack.packb(message, use_bin_type=True)

    def _unpack_message(self, data: bytes) -> Tuple[int, bytes, Dict[str, Any]]:
        """Распаковка сообщения"""
        message = msgpack.unpackb(data, raw=False)
        return message["type"], message["id"], message["payload"]

    async def _handle_request(
        self, msg_type: int, msg_id: bytes, payload: Dict[str, Any], address: Tuple[str, int]
    ):
        """Обработка входящего запроса"""
        if msg_type == self.MSG_PING:
            # Обновляем информацию об узле в routing table
            if self.routing_table and "node_id" in payload:
                try:
                    sender_node_id = NodeID(id=payload["node_id"])
                    sender_node = Node(node_id=sender_node_id, address=address[0], port=address[1])
                    self.routing_table.add_node(sender_node)
                except Exception as e:
                    self.logger.warning("Error adding node to routing table", error=str(e))

            response = self._pack_message(
                self.MSG_PONG, msg_id, {"node_id": self.node_id.id, "address": self.local_address}
            )
            await self.transport.send(response, address)

        elif msg_type == self.MSG_FIND_NODE:
            # Обработка FIND_NODE запроса
            if "target_id" in payload and self.routing_table:
                try:
                    target_id = NodeID(id=payload["target_id"])
                    closest_nodes = self.routing_table.find_closest_nodes(
                        target_id, self.routing_table.k
                    )

                    # Сериализуем найденные узлы
                    nodes_data = []
                    for node in closest_nodes:
                        nodes_data.append(
                            {"node_id": node.node_id.id, "address": node.address, "port": node.port}
                        )

                    response = self._pack_message(
                        self.MSG_FIND_NODE_RESPONSE, msg_id, {"nodes": nodes_data}
                    )
                    await self.transport.send(response, address)
                except Exception as e:
                    self.logger.error("Error handling FIND_NODE", error=str(e), exc_info=True)

        elif msg_type == self.MSG_POPULARITY_EXCHANGE:
            # Обработка обмена популярностью
            if "items" in payload and self.popularity_exchanger:
                try:
                    # Получаем локальные метрики
                    local_metrics = self.popularity_exchanger.get_local_metrics()
                    if local_metrics:
                        # Ранжируем и берем топ-100
                        ranked = self.popularity_exchanger.ranker.rank_items(
                            local_metrics, limit=100
                        )
                        response_items = [
                            {
                                "key": item.key.hex(),
                                "score": item.score,
                                "metrics": item.metrics.to_dict(),
                            }
                            for item in ranked
                        ]
                    else:
                        response_items = []

                    response = self._pack_message(
                        self.MSG_POPULARITY_EXCHANGE_RESPONSE, msg_id, {"items": response_items}
                    )
                    await self.transport.send(response, address)

                    # Обрабатываем полученные данные
                    received_items = payload["items"]
                    if received_items:
                        await self.popularity_exchanger.process_received_items(received_items)

                except Exception as e:
                    self.logger.error(
                        "Error handling popularity exchange", error=str(e), exc_info=True
                    )

        elif msg_type == self.MSG_GLOBAL_RANKING_REQUEST:
            # Обработка запроса глобального рейтинга (для seed-узлов)
            if self.popularity_exchanger:
                try:
                    global_ranking = await self.popularity_exchanger.get_global_ranking()

                    response = self._pack_message(
                        self.MSG_GLOBAL_RANKING_RESPONSE, msg_id, {"ranking": global_ranking}
                    )
                    await self.transport.send(response, address)
                except Exception as e:
                    self.logger.error(
                        "Error handling global ranking request", error=str(e), exc_info=True
                    )

        elif msg_type == self.MSG_FIND_VALUE:
            # Обработка FIND_VALUE запроса
            if "key" in payload and self.storage:
                try:
                    key = payload["key"]
                    # Получаем значение из storage
                    value = await self.storage.get(key)

                    if value is not None:
                        response = self._pack_message(
                            self.MSG_FIND_VALUE_RESPONSE, msg_id, {"found": True, "value": value}
                        )
                    else:
                        # Значение не найдено, возвращаем ближайшие узлы
                        if self.routing_table:
                            key_hash = (
                                key[:20] if len(key) >= 20 else key + b"\x00" * (20 - len(key))
                            )
                            target_id = NodeID(id=key_hash[:20])
                            closest_nodes = self.routing_table.find_closest_nodes(
                                target_id, self.routing_table.k
                            )

                            nodes_data = []
                            for node in closest_nodes:
                                nodes_data.append(
                                    {
                                        "node_id": node.node_id.id,
                                        "address": node.address,
                                        "port": node.port,
                                    }
                                )

                            response = self._pack_message(
                                self.MSG_FIND_VALUE_RESPONSE,
                                msg_id,
                                {"found": False, "nodes": nodes_data},
                            )
                        else:
                            response = self._pack_message(
                                self.MSG_FIND_VALUE_RESPONSE, msg_id, {"found": False}
                            )

                    await self.transport.send(response, address)
                except Exception as e:
                    self.logger.error("Error handling FIND_VALUE", error=str(e), exc_info=True)

        elif msg_type == self.MSG_STORE:
            # Обработка STORE запроса
            if "key" in payload and "value" in payload and self.storage:
                try:
                    key = payload["key"]
                    value = payload["value"]
                    ttl = payload.get("ttl", 86400)  # По умолчанию 1 день

                    # Сохраняем в storage
                    await self.storage.put(key, value, ttl)

                    response = self._pack_message(
                        self.MSG_STORE_RESPONSE, msg_id, {"success": True}
                    )
                    await self.transport.send(response, address)

                    self.logger.debug("Stored value", key=key.hex()[:16], size=len(value))
                except Exception as e:
                    self.logger.error("Error handling STORE", error=str(e), exc_info=True)
                    response = self._pack_message(
                        self.MSG_STORE_RESPONSE, msg_id, {"success": False, "error": str(e)}
                    )
                    await self.transport.send(response, address)

    async def ping(self, node: Node) -> bool:
        """
        Отправка PING запроса

        Args:
            node: Узел для ping

        Returns:
            True если получен ответ
        """
        msg_id = self._generate_msg_id()
        future = asyncio.Future()
        self.pending_requests[msg_id] = future

        message = self._pack_message(self.MSG_PING, msg_id, {"node_id": self.node_id.id})

        try:
            await self.transport.send(message, (node.address, node.port))

            # Ждем ответ с таймаутом
            try:
                msg_type, payload = await asyncio.wait_for(future, timeout=self.request_timeout)
                if msg_type == self.MSG_PONG:
                    return True
            except asyncio.TimeoutError:
                self.logger.warning("PING timeout", node=node.address)
                return False
        finally:
            self.pending_requests.pop(msg_id, None)

        return False

    def _generate_msg_id(self) -> bytes:
        """Генерация уникального ID сообщения"""
        import secrets

        return secrets.token_bytes(16)

    async def find_node(self, target_id: NodeID, node: Node) -> List[Node]:
        """
        Поиск узлов через сеть

        Args:
            target_id: Целевой Node ID
            node: Узел для запроса

        Returns:
            Список найденных узлов
        """
        msg_id = self._generate_msg_id()
        future = asyncio.Future()
        self.pending_requests[msg_id] = future

        message = self._pack_message(self.MSG_FIND_NODE, msg_id, {"target_id": target_id.id})

        try:
            await self.transport.send(message, (node.address, node.port))

            # Ждем ответ с таймаутом
            try:
                msg_type, payload = await asyncio.wait_for(future, timeout=self.request_timeout)
                if msg_type == self.MSG_FIND_NODE_RESPONSE and "nodes" in payload:
                    # Преобразуем данные в объекты Node
                    nodes = []
                    for node_data in payload["nodes"]:
                        try:
                            node_id = NodeID(id=node_data["node_id"])
                            found_node = Node(
                                node_id=node_id,
                                address=node_data["address"],
                                port=node_data["port"],
                            )
                            nodes.append(found_node)
                        except Exception as e:
                            self.logger.warning("Error parsing node data", error=str(e))
                    return nodes
            except asyncio.TimeoutError:
                self.logger.warning("FIND_NODE timeout", node=node.address)
                return []
        finally:
            self.pending_requests.pop(msg_id, None)

        return []

    async def find_value(self, key: bytes, node: Node) -> Optional[bytes]:
        """
        Поиск значения по ключу через сеть

        Args:
            key: Ключ для поиска
            node: Узел для запроса

        Returns:
            Значение или None если не найдено
        """
        msg_id = self._generate_msg_id()
        future = asyncio.Future()
        self.pending_requests[msg_id] = future

        message = self._pack_message(self.MSG_FIND_VALUE, msg_id, {"key": key})

        try:
            await self.transport.send(message, (node.address, node.port))

            # Ждем ответ с таймаутом
            try:
                msg_type, payload = await asyncio.wait_for(future, timeout=self.request_timeout)
                if msg_type == self.MSG_FIND_VALUE_RESPONSE:
                    if payload.get("found") and "value" in payload:
                        return payload["value"]
                    # Если значение не найдено, но есть узлы, возвращаем None
                    # (вызывающий код может использовать nodes для дальнейшего поиска)
                    return None
            except asyncio.TimeoutError:
                self.logger.warning("FIND_VALUE timeout", node=node.address)
                return None
        finally:
            self.pending_requests.pop(msg_id, None)

        return None

    async def store(self, key: bytes, value: bytes, ttl: int, node: Node) -> bool:
        """
        Сохранение значения на удаленном узле

        Args:
            key: Ключ
            value: Значение
            ttl: Time to live в секундах
            node: Узел для сохранения

        Returns:
            True если успешно сохранено
        """
        msg_id = self._generate_msg_id()
        future = asyncio.Future()
        self.pending_requests[msg_id] = future

        message = self._pack_message(
            self.MSG_STORE, msg_id, {"key": key, "value": value, "ttl": ttl}
        )

        try:
            await self.transport.send(message, (node.address, node.port))

            # Ждем ответ с таймаутом
            try:
                msg_type, payload = await asyncio.wait_for(future, timeout=self.request_timeout)
                if msg_type == self.MSG_STORE_RESPONSE:
                    return payload.get("success", False)
            except asyncio.TimeoutError:
                self.logger.warning("STORE timeout", node=node.address)
                return False
        finally:
            self.pending_requests.pop(msg_id, None)

        return False
