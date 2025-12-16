"""
Базовый класс узла
"""

import asyncio
from pathlib import Path
from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from rhizome.dht.protocol import DHTProtocol as DHTProtocolType

from rhizome.config import Config
from rhizome.dht.node import Node, NodeID
from rhizome.dht.protocol import DHTProtocol
from rhizome.dht.routing_table import RoutingTable
from rhizome.exceptions import InvalidNodeTypeError
from rhizome.logger import get_logger, setup_logging
from rhizome.network.protocol import NetworkProtocol
from rhizome.network.transport import UDPTransport
from rhizome.popularity.exchanger import PopularityExchanger
from rhizome.popularity.metrics import MetricsCollector
from rhizome.popularity.ranking import PopularityRanker
from rhizome.replication.replicator import Replicator
from rhizome.storage.storage import Storage
from rhizome.utils.crypto import generate_node_id, load_node_id, save_node_id


class BaseNode:
    """Базовый класс для всех типов узлов"""

    NODE_TYPES = ["seed", "full", "light", "mobile"]

    def __init__(self, config: Config):
        self.config = config
        self.node_type = config.node.node_type

        if self.node_type not in self.NODE_TYPES:
            raise InvalidNodeTypeError(f"Invalid node type: {self.node_type}")

        # Настройка логирования (временно без node_id)
        self.logger = setup_logging(
            log_level=config.log_level, log_file=config.log_file, node_id=None  # Добавим позже
        )
        self.logger = self.logger.bind(node_type=self.node_type)

        # Загрузка или генерация Node ID
        self.node_id = self._load_or_generate_node_id()

        # Обновляем logger с node_id
        self.logger = self.logger.bind(node_id=self.node_id.id.hex()[:16])

        # Инициализация компонентов
        self.routing_table = RoutingTable(
            self.node_id, k=config.dht.k, bucket_count=config.dht.bucket_count
        )

        self.storage = Storage(config.storage)

        # Сетевой транспорт
        self.transport = UDPTransport(
            host=config.network.listen_host, port=config.network.listen_port
        )

        # Система популярности (создаем до network_protocol)
        self.metrics_collector = MetricsCollector()
        self.popularity_ranker = PopularityRanker(
            popularity_threshold=config.popularity.popularity_threshold,
            active_threshold=config.popularity.active_threshold,
        )

        # Сетевой протокол (создаем без popularity_exchanger, добавим позже)
        self.network_protocol = NetworkProtocol(
            self.transport,
            self.node_id,
            (config.network.listen_host, config.network.listen_port),
            routing_table=self.routing_table,
            storage=self.storage,
            popularity_exchanger=None,  # Добавим позже
        )

        # DHT протокол с интеграцией сетевого протокола
        self.dht_protocol = DHTProtocol(self.routing_table, self.storage, self.network_protocol)

        # Создаем popularity_exchanger
        self.popularity_exchanger = PopularityExchanger(
            self.dht_protocol,
            self.network_protocol,
            self.popularity_ranker,
            metrics_collector=self.metrics_collector,
        )

        # Обновляем network_protocol с popularity_exchanger
        self.network_protocol.popularity_exchanger = self.popularity_exchanger

        # Система репликации
        self.replicator = Replicator(
            self.dht_protocol, self.storage, min_replication_factor=5, popular_replication_factor=10
        )

        # Состояние узла
        self.is_running = False
        self.start_time: Optional[float] = None

        self.logger.info("Node initialized", node_id=self.node_id.id.hex()[:16])

    def _load_or_generate_node_id(self) -> NodeID:
        """Загрузка Node ID из файла или генерация нового"""
        node_id_file = Path(self.config.node.node_id_file)

        # Пытаемся загрузить существующий Node ID
        node_id_bytes = load_node_id(node_id_file)

        if node_id_bytes is None:
            # Генерируем новый Node ID
            self.logger.info("Generating new node ID")
            node_id_bytes = generate_node_id()
            save_node_id(node_id_bytes, node_id_file)
            self.logger.info("Node ID saved", file=str(node_id_file))
        else:
            self.logger.info("Node ID loaded from file", file=str(node_id_file))

        return NodeID(id=node_id_bytes)

    async def start(self):
        """Запуск узла"""
        if self.is_running:
            return

        self.logger.info("Starting node")
        self.is_running = True
        self.start_time = asyncio.get_event_loop().time()

        # Запуск сетевого протокола
        await self.network_protocol.start()

        # Bootstrap процесс
        await self.bootstrap()

        # Запуск фоновых задач
        asyncio.create_task(self._background_tasks())
        asyncio.create_task(self._popularity_tasks())

        self.logger.info("Node started", node_id=self.node_id.id.hex()[:16])

    async def stop(self):
        """Остановка узла"""
        if not self.is_running:
            return

        self.logger.info("Stopping node")
        self.is_running = False

        # Остановка сетевого протокола
        await self.network_protocol.stop()

        # Сохранение состояния
        await self._save_state()

        # Закрытие хранилища
        self.storage.close()

        self.logger.info("Node stopped")

    async def _save_state(self):
        """Сохранение состояния узла"""
        # TODO: Реализовать сохранение состояния
        pass

    async def bootstrap(self):
        """Bootstrap процесс для подключения к сети"""
        bootstrap_nodes = self.config.network.bootstrap_nodes

        if not bootstrap_nodes:
            self.logger.warning("No bootstrap nodes configured")
            return

        self.logger.info("Starting bootstrap", bootstrap_count=len(bootstrap_nodes))

        # Пытаемся подключиться к bootstrap узлам
        connected = False
        for bootstrap_addr in bootstrap_nodes:
            try:
                host, port = bootstrap_addr.split(":")
                port = int(port)

                # Создаем временный узел для bootstrap
                # В реальной реализации нужно получить Node ID через запрос
                bootstrap_node = Node(
                    node_id=NodeID(id=b"\x00" * 20), address=host, port=port  # Временный ID
                )

                # Пытаемся сделать PING
                if await self.network_protocol.ping(bootstrap_node):
                    self.logger.info("Bootstrap node connected", host=host, port=port)
                    # Добавляем узел в routing table
                    self.routing_table.add_node(bootstrap_node)
                    connected = True

                    # Запрашиваем список ближайших узлов
                    # TODO: Реализовать FIND_NODE запрос
                else:
                    self.logger.warning("Bootstrap node unreachable", host=host, port=port)

            except Exception as e:
                self.logger.warning("Bootstrap failed", address=bootstrap_addr, error=str(e))

        if connected:
            self.logger.info("Bootstrap completed successfully")
        else:
            self.logger.warning("Bootstrap completed with no connections")

    async def _background_tasks(self):
        """Фоновые задачи узла"""
        while self.is_running:
            try:
                # Рефрешинг бакетов
                await self._refresh_buckets()

                # Очистка истекших данных
                deleted = await self.storage.cleanup_expired()
                if deleted > 0:
                    self.logger.debug("Cleaned up expired data", count=deleted)

                # Ждем до следующего цикла
                await asyncio.sleep(60)  # Каждую минуту

            except Exception as e:
                self.logger.error("Error in background tasks", error=str(e), exc_info=True)
                await asyncio.sleep(60)

    async def _refresh_buckets(self):
        """Рефрешинг k-бакетов"""
        # Находим бакеты, которые не обновлялись долго
        current_time = asyncio.get_event_loop().time()
        refresh_interval = self.config.dht.refresh_interval

        for i, bucket in enumerate(self.routing_table.buckets):
            if bucket.nodes and (current_time - bucket.last_updated) > refresh_interval:
                # Выбираем случайный ID в диапазоне этого бакета
                # TODO: Реализовать правильный поиск случайного ключа
                self.logger.debug("Refreshing bucket", bucket_index=i)

    async def _popularity_tasks(self):
        """Фоновые задачи для системы популярности"""
        update_interval = self.config.popularity.update_interval
        exchange_interval = self.config.popularity.exchange_interval

        last_update = 0
        last_exchange = 0

        while self.is_running:
            try:
                current_time = asyncio.get_event_loop().time()

                # Обновление рейтингов каждый час
                if current_time - last_update >= update_interval:
                    await self._update_popularity_rankings()
                    last_update = current_time

                # Обмен топ-100 каждые 6 часов
                if current_time - last_exchange >= exchange_interval:
                    await self._exchange_popularity()
                    last_exchange = current_time

                # Обновление свежести метрик
                self.metrics_collector.update_all_freshness()

                # Очистка старых метрик
                self.metrics_collector.cleanup_old_metrics()

                await asyncio.sleep(60)  # Проверяем каждую минуту

            except Exception as e:
                self.logger.error("Error in popularity tasks", error=str(e), exc_info=True)
                await asyncio.sleep(60)

    async def _update_popularity_rankings(self):
        """Обновление рейтингов популярности"""
        all_metrics = self.metrics_collector.get_all_metrics()

        if not all_metrics:
            return

        # Ранжируем элементы
        ranked = self.popularity_ranker.rank_items(all_metrics, limit=100)

        # Обновляем TTL на основе популярности
        for item in ranked:
            metrics = item.metrics
            key = item.key

            # Продлеваем TTL для популярных данных
            if item.score >= self.config.popularity.popularity_threshold:
                # Популярные данные - продлеваем до 30 дней
                await self.storage.extend_ttl(key, extension=1.0)  # Удваиваем TTL
                self.logger.debug(
                    "Extended TTL for popular item", key=key.hex()[:16], score=item.score
                )
            elif item.score >= self.config.popularity.active_threshold:
                # Активные данные - продлеваем до 7 дней
                await self.storage.extend_ttl(key, extension=0.5)

        # Реплицируем популярные элементы
        popular_items = [
            item for item in ranked if item.score >= self.config.popularity.popularity_threshold
        ]

        if popular_items:
            await self.replicator.replicate_popular_items(
                popular_items, popularity_threshold=self.config.popularity.popularity_threshold
            )

        self.logger.info(
            "Updated popularity rankings",
            total_items=len(all_metrics),
            popular_count=len(popular_items),
        )

    async def _exchange_popularity(self):
        """Обмен данными о популярности с соседними узлами"""
        all_metrics = self.metrics_collector.get_all_metrics()

        if not all_metrics:
            return

        # Получаем соседние узлы из routing table
        neighbor_nodes = self.routing_table.get_all_nodes()[:10]  # Берем первые 10

        if not neighbor_nodes:
            return

        # Обмениваемся данными
        updated_metrics = await self.popularity_exchanger.exchange_top_items(
            all_metrics, neighbor_nodes, top_n=100
        )

        self.logger.info("Exchanged popularity data", neighbors=len(neighbor_nodes))

    async def find_value(self, key: bytes) -> Optional[bytes]:
        """
        Поиск значения с сбором метрик

        Args:
            key: Ключ для поиска

        Returns:
            Значение или None если не найдено
        """
        # Собираем метрику запроса
        self.metrics_collector.record_find_value(key, node_id=self.node_id.id)

        try:
            value = await self.dht_protocol.find_value(key)
            return value
        except Exception as e:
            self.logger.warning("find_value failed", key=key.hex()[:16], error=str(e))
            raise

    async def store(self, key: bytes, value: bytes, ttl: int = 86400) -> bool:
        """
        Сохранение значения с сбором метрик

        Args:
            key: Ключ
            value: Значение
            ttl: Time to live в секундах

        Returns:
            True если успешно сохранено
        """
        # Сохраняем через DHT протокол
        result = await self.dht_protocol.store(key, value, ttl)

        # Собираем метрику сохранения
        # TODO: Получить реальное количество репликаций
        replication_count = 1
        if result:
            # Если сохранено успешно, предполагаем репликацию на k узлах
            replication_count = self.routing_table.k

        self.metrics_collector.record_store(key, replication_count=replication_count)

        return result
