"""
Seed-узел (высокая доступность, долгосрочное хранение)
"""

import asyncio

from rhizome.config import Config
from rhizome.dht.node import Node
from rhizome.node.base_node import BaseNode


class SeedNode(BaseNode):
    """Seed-узел с высокой доступностью и большим объемом хранилища"""

    def __init__(self, config: Config):
        # Принудительно устанавливаем тип seed
        config.node.node_type = "seed"
        super().__init__(config)

    async def start(self):
        """Запуск seed-узла с дополнительными задачами"""
        await super().start()

        # Запуск задач для seed-узла
        asyncio.create_task(self._seed_tasks())

    async def _seed_tasks(self):
        """Фоновые задачи для seed-узла"""
        global_update_interval = self.config.popularity.global_update_interval

        last_global_update = 0

        while self.is_running:
            try:
                current_time = asyncio.get_event_loop().time()

                # Глобальное ранжирование каждые 3 часа
                if current_time - last_global_update >= global_update_interval:
                    await self._update_global_ranking()
                    last_global_update = current_time

                await asyncio.sleep(300)  # Проверяем каждые 5 минут

            except Exception as e:
                self.logger.error("Error in seed tasks", error=str(e), exc_info=True)
                await asyncio.sleep(300)

    async def _update_global_ranking(self):
        """Обновление глобального рейтинга"""
        # Получаем локальные рейтинги
        all_metrics = self.metrics_collector.get_all_metrics()
        if not all_metrics:
            return

        local_ranked = self.popularity_ranker.rank_items(all_metrics, limit=100)

        # Получаем список других seed-узлов
        # TODO: Реализовать получение списка seed-узлов из DHT
        seed_nodes = []  # Пока пустой список

        # Агрегируем глобальный рейтинг
        global_ranking = await self.popularity_exchanger.aggregate_global_ranking(
            local_ranked, seed_nodes
        )

        self.logger.info("Updated global ranking", items=len(global_ranking))
