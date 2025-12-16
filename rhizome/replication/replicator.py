"""
Система репликации данных на основе популярности
"""

import asyncio
from typing import Dict, List, Optional

from rhizome.dht.protocol import DHTProtocol
from rhizome.logger import get_logger
from rhizome.popularity.ranking import RankedItem
from rhizome.storage.storage import Storage


class Replicator:
    """Репликатор данных на основе популярности"""

    def __init__(
        self,
        dht_protocol: DHTProtocol,
        storage: Storage,
        min_replication_factor: int = 5,
        popular_replication_factor: int = 10,
    ):
        self.dht_protocol = dht_protocol
        self.storage = storage
        self.min_replication_factor = min_replication_factor
        self.popular_replication_factor = popular_replication_factor
        self.logger = get_logger("replication.replicator")

    async def replicate_popular_items(
        self, ranked_items: List[RankedItem], popularity_threshold: float = 7.0
    ) -> Dict[bytes, bool]:
        """
        Репликация популярных элементов

        Args:
            ranked_items: Отсортированный список элементов по популярности
            popularity_threshold: Порог популярности для репликации

        Returns:
            Словарь результатов репликации (key -> success)
        """
        results = {}

        # Фильтруем популярные элементы
        popular_items = [item for item in ranked_items if item.score >= popularity_threshold]

        self.logger.info(
            "Starting replication", total_items=len(ranked_items), popular_items=len(popular_items)
        )

        # Реплицируем каждый популярный элемент
        for item in popular_items:
            key = item.key
            try:
                # Получаем значение из storage
                value = await self.storage.get(key)
                if value is None:
                    self.logger.warning("Value not found for replication", key=key.hex()[:16])
                    results[key] = False
                    continue

                # Проверяем текущий replication factor
                current_replication = item.metrics.replication_count
                target_replication = self.popular_replication_factor

                if current_replication >= target_replication:
                    # Уже достаточно репликаций
                    results[key] = True
                    continue

                # Реплицируем через STORE
                # Используем увеличенный TTL для популярных данных
                ttl = 2592000  # 30 дней
                success = await self.dht_protocol.store(key, value, ttl)

                results[key] = success

                if success:
                    self.logger.debug(
                        "Replicated popular item",
                        key=key.hex()[:16],
                        score=item.score,
                        target_replication=target_replication,
                    )
                else:
                    self.logger.warning("Replication failed", key=key.hex()[:16])

            except Exception as e:
                self.logger.error(
                    "Error replicating item", key=key.hex()[:16], error=str(e), exc_info=True
                )
                results[key] = False

        successful = sum(1 for v in results.values() if v)
        self.logger.info(
            "Replication completed",
            total=len(results),
            successful=successful,
            failed=len(results) - successful,
        )

        return results

    async def ensure_minimal_replication(
        self, keys: List[bytes], min_factor: Optional[int] = None
    ) -> Dict[bytes, bool]:
        """
        Обеспечение минимальной репликации для списка ключей

        Args:
            keys: Список ключей для проверки
            min_factor: Минимальный фактор репликации (по умолчанию из конфига)

        Returns:
            Словарь результатов (key -> success)
        """
        if min_factor is None:
            min_factor = self.min_replication_factor

        results = {}

        for key in keys:
            try:
                # Получаем значение
                value = await self.storage.get(key)
                if value is None:
                    results[key] = False
                    continue

                # TODO: Проверить реальный replication factor через DHT
                # Пока просто реплицируем
                success = await self.dht_protocol.store(key, value, ttl=86400)
                results[key] = success

            except Exception as e:
                self.logger.error("Error ensuring replication", key=key.hex()[:16], error=str(e))
                results[key] = False

        return results

    async def emergency_replication(self, key: bytes, value: bytes) -> bool:
        """
        Экстренная репликация при обнаружении потери узла

        Args:
            key: Ключ для репликации
            value: Значение для репликации

        Returns:
            True если успешно реплицировано
        """
        self.logger.warning("Emergency replication", key=key.hex()[:16])

        try:
            # Реплицируем с высоким приоритетом
            # Используем увеличенный TTL
            success = await self.dht_protocol.store(key, value, ttl=2592000)  # 30 дней

            if success:
                self.logger.info("Emergency replication successful", key=key.hex()[:16])
            else:
                self.logger.error("Emergency replication failed", key=key.hex()[:16])

            return success

        except Exception as e:
            self.logger.error(
                "Error in emergency replication", key=key.hex()[:16], error=str(e), exc_info=True
            )
            return False
