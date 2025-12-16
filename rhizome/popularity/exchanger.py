"""
Обмен рейтингами популярности между узлами
"""

import asyncio
import time
from collections import defaultdict
from typing import Any, Dict, List, Optional

from rhizome.dht.node import Node
from rhizome.logger import get_logger
from rhizome.popularity.metrics import PopularityMetrics
from rhizome.popularity.ranking import RankedItem


class PopularityExchanger:
    """Обмен топ-100 популярных элементов между узлами"""

    def __init__(self, dht_protocol, network_protocol, ranker, metrics_collector=None):
        self.dht_protocol = dht_protocol
        self.network_protocol = network_protocol
        self.ranker = ranker
        self.metrics_collector = metrics_collector
        self.logger = get_logger("popularity.exchanger")

        # Глобальный рейтинг (для seed-узлов)
        self.global_ranking: List[RankedItem] = []
        self.global_ranking_updated: float = 0.0

        # Агрегированные рейтинги от других узлов
        self.aggregated_rankings: Dict[bytes, List[RankedItem]] = {}

    async def exchange_top_items(
        self, local_metrics: Dict[bytes, PopularityMetrics], neighbor_nodes: List, top_n: int = 100
    ) -> Dict[bytes, PopularityMetrics]:
        """
        Обмен топ-N элементами с соседними узлами

        Args:
            local_metrics: Локальные метрики
            neighbor_nodes: Список соседних узлов
            top_n: Количество топ элементов для обмена

        Returns:
            Обновленные метрики с учетом данных от других узлов
        """
        # Получаем локальный топ
        local_ranked = self.ranker.rank_items(local_metrics, limit=top_n)

        # Подготавливаем данные для отправки
        exchange_data = []
        for item in local_ranked:
            exchange_data.append(
                {"key": item.key.hex(), "score": item.score, "metrics": item.metrics.to_dict()}
            )

        # Отправляем данные соседним узлам и получаем их данные
        if not neighbor_nodes:
            return local_metrics

        # Подготавливаем данные для отправки
        exchange_data = [
            {"key": item.key.hex(), "score": item.score, "metrics": item.metrics.to_dict()}
            for item in local_ranked
        ]

        # Параллельно обмениваемся с соседними узлами
        tasks = [
            self.network_protocol.exchange_popularity(exchange_data, node)
            for node in neighbor_nodes[:5]  # Ограничиваем до 5 узлов
        ]

        results = await asyncio.gather(*tasks, return_exceptions=True)

        # Обрабатываем полученные данные
        updated_metrics = local_metrics.copy()
        received_count = 0

        for result in results:
            if isinstance(result, list):
                received_count += len(result)
                for item_data in result:
                    try:
                        key = bytes.fromhex(item_data["key"])
                        received_score = item_data.get("score", 0.0)
                        received_metrics = item_data.get("metrics", {})

                        # Обновляем replication_count на основе полученных данных
                        if key in updated_metrics:
                            # Если у нас уже есть метрики, обновляем replication_count
                            received_replication = received_metrics.get("replication_count", 1)
                            updated_metrics[key].update_replication(
                                max(updated_metrics[key].replication_count, received_replication)
                            )
                        else:
                            # Создаем новые метрики из полученных данных
                            updated_metrics[key] = PopularityMetrics.from_dict(received_metrics)
                    except Exception as e:
                        self.logger.warning("Error processing received item", error=str(e))

        self.logger.info(
            "Exchanged popularity data",
            local_items=len(exchange_data),
            neighbors=len(neighbor_nodes),
            received_items=received_count,
        )

        return updated_metrics

    def get_local_metrics(self) -> Optional[Dict[bytes, PopularityMetrics]]:
        """Получение локальных метрик"""
        if self.metrics_collector:
            return self.metrics_collector.get_all_metrics()
        return None

    async def process_received_items(self, items: List[Dict[str, Any]]):
        """Обработка полученных элементов при обмене"""
        # Обновляем метрики на основе полученных данных
        if not self.metrics_collector:
            return

        for item_data in items:
            try:
                key = bytes.fromhex(item_data["key"])
                received_metrics = item_data.get("metrics", {})

                # Обновляем replication_count
                if key in self.metrics_collector.metrics:
                    received_replication = received_metrics.get("replication_count", 1)
                    self.metrics_collector.metrics[key].update_replication(
                        max(
                            self.metrics_collector.metrics[key].replication_count,
                            received_replication,
                        )
                    )
            except Exception as e:
                self.logger.warning("Error processing received item", error=str(e))

    async def aggregate_global_ranking(
        self, local_rankings: List[RankedItem], seed_nodes: List[Node]
    ) -> List[RankedItem]:
        """
        Агрегация глобального рейтинга (для seed-узлов)

        Args:
            local_rankings: Локальные рейтинги
            seed_nodes: Список seed-узлов

        Returns:
            Консенсусный глобальный рейтинг
        """
        # Собираем рейтинги от других seed-узлов
        all_rankings: Dict[bytes, List[float]] = defaultdict(list)

        # Добавляем локальные рейтинги
        for item in local_rankings:
            all_rankings[item.key].append(item.score)

        # Запрашиваем рейтинги от других seed-узлов
        tasks = [
            self.network_protocol.request_global_ranking(seed_node)
            for seed_node in seed_nodes[:10]  # Ограничиваем до 10 seed-узлов
        ]

        results = await asyncio.gather(*tasks, return_exceptions=True)

        # Обрабатываем полученные рейтинги
        for result in results:
            if isinstance(result, list):
                for item_data in result:
                    try:
                        key = bytes.fromhex(item_data["key"])
                        score = item_data.get("score", 0.0)
                        all_rankings[key].append(score)
                    except Exception as e:
                        self.logger.warning("Error processing ranking item", error=str(e))

        # Вычисляем консенсусный рейтинг (медиана или среднее)
        consensus_ranking = []

        for key, scores in all_rankings.items():
            if scores:
                # Используем медиану для устойчивости к выбросам
                sorted_scores = sorted(scores)
                median_score = sorted_scores[len(sorted_scores) // 2]

                # Получаем метрики для ключа
                metrics = None
                if self.metrics_collector:
                    metrics = self.metrics_collector.get_metrics(key)

                if metrics:
                    consensus_ranking.append(
                        RankedItem(key=key, score=median_score, metrics=metrics)
                    )

        # Сортируем по убыванию рейтинга
        consensus_ranking.sort(reverse=True)

        # Обновляем глобальный рейтинг
        self.global_ranking = consensus_ranking[:100]  # Топ-100
        self.global_ranking_updated = time.time()

        self.logger.info(
            "Aggregated global ranking",
            local_items=len(local_rankings),
            seed_nodes=len(seed_nodes),
            consensus_items=len(self.global_ranking),
        )

        return self.global_ranking

    async def get_global_ranking(self) -> List[Dict[str, Any]]:
        """
        Получение глобального рейтинга (для seed-узлов)

        Returns:
            Список элементов глобального рейтинга
        """
        # Если рейтинг устарел (старше 3 часов), обновляем
        if time.time() - self.global_ranking_updated > 10800:  # 3 часа
            # TODO: Запустить обновление рейтинга
            pass

        return [
            {"key": item.key.hex(), "score": item.score, "metrics": item.metrics.to_dict()}
            for item in self.global_ranking
        ]
