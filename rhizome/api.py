"""
Высокоуровневый API клиент для работы с Rhizome
Предоставляет удобный интерфейс для работы с узлами, тредами, сообщениями
"""

import asyncio
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

from rhizome.config import Config
from rhizome.node.full_node import FullNode
from rhizome.storage.data_types import Message, Thread, ThreadMetadata
from rhizome.storage.keys import KeyManager
from rhizome.utils.crypto import hash_key
from rhizome.utils.serialization import deserialize, serialize


class RhizomeClient:
    """
    Высокоуровневый клиент для работы с Rhizome P2P сетью

    Упрощает работу с узлами, тредами и сообщениями.
    Инкапсулирует детали протокола и предоставляет удобный интерфейс.

    Пример использования:
        ```python
        from rhizome.api import RhizomeClient

        async def main():
            # Создание клиента
            client = RhizomeClient(config_path="config.yaml")

            # Запуск узла
            await client.start()

            # Создание треда
            thread = await client.create_thread(
                thread_id="my_thread",
                title="Мой первый тред",
                category="технологии",
                tags=["p2p", "python"]
            )

            # Добавление сообщения
            message = await client.add_message(
                thread_id="my_thread",
                content="Привет, мир!",
                author="user123"
            )

            # Поиск треда
            found_thread = await client.find_thread("my_thread")

            # Остановка узла
            await client.stop()
        ```
    """

    def __init__(self, config_path: Optional[str] = None, config: Optional[Config] = None):
        """
        Инициализация клиента

        Args:
            config_path: Путь к файлу конфигурации (config.yaml)
            config: Объект Config (если не указан config_path)
        """
        if config:
            self.config = config
        elif config_path:
            self.config = Config.from_file(Path(config_path))
        else:
            # Используем конфигурацию по умолчанию
            self.config = (
                Config.from_file(Path("config.yaml")) if Path("config.yaml").exists() else Config()
            )

        self.node: Optional[FullNode] = None
        self.key_manager = KeyManager()
        self._is_running = False

    async def start(self) -> None:
        """
        Запуск узла и подключение к сети

        Raises:
            RuntimeError: Если узел уже запущен
        """
        if self._is_running:
            raise RuntimeError("Node is already running. Call stop() first.")

        if self.node is None:
            self.node = FullNode(self.config)

        await self.node.start()
        self._is_running = True

        # Даем время на инициализацию
        await asyncio.sleep(1)

    async def stop(self) -> None:
        """Остановка узла"""
        if self.node and self._is_running:
            await self.node.stop()
            self._is_running = False

    async def create_thread(
        self,
        thread_id: str,
        title: str,
        category: Optional[str] = None,
        tags: Optional[List[str]] = None,
        creator_pubkey: Optional[str] = None,
        ttl: int = 86400,
    ) -> ThreadMetadata:
        """
        Создание нового треда

        Args:
            thread_id: Уникальный идентификатор треда
            title: Заголовок треда
            category: Категория треда (опционально)
            tags: Список тегов (опционально)
            creator_pubkey: Публичный ключ создателя (опционально)
            ttl: Время жизни в секундах (по умолчанию 1 день)

        Returns:
            ThreadMetadata: Метаданные созданного треда

        Raises:
            RuntimeError: Если узел не запущен
        """
        if not self._is_running or not self.node:
            raise RuntimeError("Node is not running. Call start() first.")

        thread_meta = ThreadMetadata(
            id=thread_id,
            title=title,
            created_at=int(time.time()),
            creator_pubkey=creator_pubkey or f"0x{hash_key(thread_id).hex()[:16]}",
            category=category,
            tags=tags or [],
        )

        meta_key = self.key_manager.get_thread_meta_key(thread_id)
        meta_data = serialize(thread_meta.to_dict())
        success = await self.node.store(meta_key, meta_data, ttl=ttl)

        if not success:
            raise RuntimeError(f"Failed to create thread: {thread_id}")

        return thread_meta

    async def find_thread(self, thread_id: str) -> Optional[ThreadMetadata]:
        """
        Поиск треда по ID

        Args:
            thread_id: Идентификатор треда

        Returns:
            ThreadMetadata или None, если тред не найден

        Raises:
            RuntimeError: Если узел не запущен
        """
        if not self._is_running or not self.node:
            raise RuntimeError("Node is not running. Call start() first.")

        meta_key = self.key_manager.get_thread_meta_key(thread_id)

        try:
            data = await self.node.find_value(meta_key)
            if data:
                thread_dict = deserialize(data)
                return ThreadMetadata.from_dict(thread_dict)
        except Exception:
            pass

        return None

    async def update_thread(self, thread_id: str, **updates) -> Optional[ThreadMetadata]:
        """
        Обновление метаданных треда

        Args:
            thread_id: Идентификатор треда
            **updates: Поля для обновления (message_count, last_activity, popularity_score и т.д.)

        Returns:
            Обновленные метаданные треда или None, если тред не найден

        Raises:
            RuntimeError: Если узел не запущен
        """
        thread_meta = await self.find_thread(thread_id)
        if not thread_meta:
            return None

        # Обновляем поля
        for key, value in updates.items():
            if hasattr(thread_meta, key):
                setattr(thread_meta, key, value)

        # Обновляем время последней активности, если не указано явно
        if "last_activity" not in updates:
            thread_meta.last_activity = int(time.time())

        # Сохраняем обновленные метаданные
        meta_key = self.key_manager.get_thread_meta_key(thread_id)
        meta_data = serialize(thread_meta.to_dict())
        success = await self.node.store(meta_key, meta_data, ttl=86400)

        if not success:
            raise RuntimeError(f"Failed to update thread: {thread_id}")

        return thread_meta

    async def add_message(
        self,
        thread_id: str,
        content: str,
        author_signature: Optional[str] = None,
        parent_id: Optional[str] = None,
        content_type: str = "text/markdown",
        ttl: int = 86400,
    ) -> Message:
        """
        Добавление сообщения в тред

        Args:
            thread_id: Идентификатор треда
            content: Содержимое сообщения
            author_signature: Подпись автора (опционально)
            parent_id: ID родительского сообщения для ответа (опционально)
            content_type: Тип контента (по умолчанию text/markdown)
            ttl: Время жизни в секундах (по умолчанию 1 день)

        Returns:
            Message: Созданное сообщение

        Raises:
            RuntimeError: Если узел не запущен или тред не найден
        """
        if not self._is_running or not self.node:
            raise RuntimeError("Node is not running. Call start() first.")

        # Проверяем, что тред существует
        thread_meta = await self.find_thread(thread_id)
        if not thread_meta:
            raise RuntimeError(f"Thread not found: {thread_id}")

        # Создаем сообщение
        message_id = f"msg_{thread_id}_{int(time.time() * 1000)}"
        message = Message(
            id=message_id,
            thread_id=thread_id,
            content=content,
            author_signature=author_signature or f"sig_{hash_key(message_id).hex()[:16]}",
            timestamp=int(time.time()),
            parent_id=parent_id,
            content_type=content_type,
        )

        # Сохраняем сообщение
        message_hash = hash_key(message.id).hex()[:16]
        message_key = self.key_manager.get_message_key(message_hash)
        message_data = serialize(message.to_dict())
        success = await self.node.store(message_key, message_data, ttl=ttl)

        if not success:
            raise RuntimeError(f"Failed to add message to thread: {thread_id}")

        # Обновляем счетчик сообщений в треде
        thread_meta.message_count = (thread_meta.message_count or 0) + 1
        thread_meta.last_activity = int(time.time())
        await self.update_thread(
            thread_id,
            message_count=thread_meta.message_count,
            last_activity=thread_meta.last_activity,
        )

        return message

    async def find_message(self, message_id: str) -> Optional[Message]:
        """
        Поиск сообщения по ID

        Args:
            message_id: Идентификатор сообщения

        Returns:
            Message или None, если сообщение не найдено

        Raises:
            RuntimeError: Если узел не запущен
        """
        if not self._is_running or not self.node:
            raise RuntimeError("Node is not running. Call start() first.")

        message_hash = hash_key(message_id).hex()[:16]
        message_key = self.key_manager.get_message_key(message_hash)

        try:
            data = await self.node.find_value(message_key)
            if data:
                message_dict = deserialize(data)
                return Message.from_dict(message_dict)
        except Exception:
            pass

        return None

    async def get_global_threads(self) -> List[str]:
        """
        Получение списка всех тредов из глобального индекса

        Returns:
            Список ID тредов

        Raises:
            RuntimeError: Если узел не запущен
        """
        if not self._is_running or not self.node:
            raise RuntimeError("Node is not running. Call start() first.")

        threads_key = self.key_manager.get_global_threads_key()

        try:
            data = await self.node.find_value(threads_key)
            if data:
                return deserialize(data)
        except Exception:
            pass

        return []

    async def update_global_threads(self, thread_ids: List[str], ttl: int = 2592000) -> bool:
        """
        Обновление глобального индекса тредов

        Args:
            thread_ids: Список ID тредов
            ttl: Время жизни в секундах (по умолчанию 30 дней)

        Returns:
            True если успешно обновлено

        Raises:
            RuntimeError: Если узел не запущен
        """
        if not self._is_running or not self.node:
            raise RuntimeError("Node is not running. Call start() first.")

        threads_key = self.key_manager.get_global_threads_key()
        threads_data = serialize(thread_ids)
        return await self.node.store(threads_key, threads_data, ttl=ttl)

    async def get_popular_threads(self, limit: int = 10) -> List[Dict[str, Any]]:
        """
        Получение списка популярных тредов

        Args:
            limit: Максимальное количество тредов

        Returns:
            Список словарей с информацией о популярных тредах (key, score)

        Raises:
            RuntimeError: Если узел не запущен
        """
        if not self._is_running or not self.node:
            raise RuntimeError("Node is not running. Call start() first.")

        all_metrics = self.node.metrics_collector.get_all_metrics()

        if not all_metrics:
            return []

        ranked = self.node.popularity_ranker.rank_items(all_metrics, limit=limit)

        return [{"key": item.key.hex(), "score": item.score} for item in ranked]

    async def search_threads(
        self,
        query: Optional[str] = None,
        category: Optional[str] = None,
        tags: Optional[List[str]] = None,
    ) -> List[ThreadMetadata]:
        """
        Поиск тредов по различным критериям

        Args:
            query: Поисковый запрос (пока не реализовано)
            category: Фильтр по категории
            tags: Фильтр по тегам

        Returns:
            Список найденных тредов

        Raises:
            RuntimeError: Если узел не запущен
        """
        if not self._is_running or not self.node:
            raise RuntimeError("Node is not running. Call start() first.")

        # Получаем все треды из глобального индекса
        thread_ids = await self.get_global_threads()
        results = []

        for thread_id in thread_ids:
            thread_meta = await self.find_thread(thread_id)
            if not thread_meta:
                continue

            # Фильтрация по категории
            if category and thread_meta.category != category:
                continue

            # Фильтрация по тегам
            if tags and not any(tag in (thread_meta.tags or []) for tag in tags):
                continue

            results.append(thread_meta)

        return results

    def get_node_info(self) -> Dict[str, Any]:
        """
        Получение информации об узле

        Returns:
            Словарь с информацией об узле
        """
        if not self.node:
            return {"status": "not_initialized"}

        return {
            "node_id": self.node.node_id.id.hex(),
            "node_type": self.node.node_type,
            "is_running": self._is_running,
            "address": f"{self.config.network.listen_host}:{self.config.network.listen_port}",
            "routing_table_size": len(self.node.routing_table.get_all_nodes()),
        }

    async def __aenter__(self):
        """Поддержка async context manager"""
        await self.start()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Поддержка async context manager"""
        await self.stop()
