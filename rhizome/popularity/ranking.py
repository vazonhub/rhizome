"""
Алгоритм ранжирования популярности
"""

import time
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple

from rhizome.logger import get_logger
from rhizome.popularity.metrics import PopularityMetrics


@dataclass
class RankedItem:
    """Элемент с рейтингом популярности"""

    key: bytes
    score: float
    metrics: PopularityMetrics

    def __lt__(self, other):
        return self.score < other.score

    def __gt__(self, other):
        return self.score > other.score


class PopularityRanker:
    """Ранжирование контента по популярности"""

    def __init__(self, popularity_threshold: float = 7.0, active_threshold: float = 5.0):
        self.popularity_threshold = popularity_threshold
        self.active_threshold = active_threshold
        self.logger = get_logger("popularity.ranking")

    def calculate_score(self, metrics: PopularityMetrics, adaptive_weights: bool = True) -> float:
        """
        Вычисление рейтинга популярности

        Args:
            metrics: Метрики популярности
            adaptive_weights: Использовать адаптивные веса в зависимости от возраста

        Returns:
            Рейтинг популярности (0.0 - 10.0)
        """
        # Базовые веса
        weights = {
            "request_rate": 0.25,
            "replication_factor": 0.20,
            "freshness": 0.15,
            "audience_size": 0.10,
            "social_engagements": 0.20,
            "seed_coverage": 0.10,
        }

        # Адаптивные веса в зависимости от возраста
        if adaptive_weights:
            age_seconds = time.time() - metrics.first_seen

            if age_seconds < 86400:  # Меньше 24 часов
                # Выше вес свежести
                weights["freshness"] = 0.30
                weights["social_engagements"] = 0.10
                weights["seed_coverage"] = 0.05
            elif age_seconds < 604800:  # Меньше 7 дней
                # Выше вес социальных взаимодействий
                weights["social_engagements"] = 0.30
                weights["freshness"] = 0.10
                weights["seed_coverage"] = 0.05
            else:  # Больше 7 дней
                # Выше вес устойчивости
                weights["seed_coverage"] = 0.25
                weights["social_engagements"] = 0.15
                weights["freshness"] = 0.05

        # Нормализация метрик
        normalized_metrics = self._normalize_metrics(metrics)

        # Вычисление рейтинга
        score = (
            normalized_metrics["request_rate"] * weights["request_rate"]
            + normalized_metrics["replication_factor"] * weights["replication_factor"]
            + normalized_metrics["freshness"] * weights["freshness"]
            + normalized_metrics["audience_size"] * weights["audience_size"]
            + normalized_metrics["social_engagements"] * weights["social_engagements"]
            + normalized_metrics["seed_coverage"] * weights["seed_coverage"]
        ) * 10.0  # Масштабируем до 0-10

        return min(10.0, max(0.0, score))

    def _normalize_metrics(self, metrics: PopularityMetrics) -> Dict[str, float]:
        """Нормализация метрик к диапазону 0-1"""
        # Request rate: нормализуем к 0-100 запросов/час
        normalized_request_rate = min(1.0, metrics.request_rate / 100.0)

        # Replication factor: нормализуем к 0-20 узлам
        normalized_replication = min(1.0, metrics.replication_count / 20.0)

        # Freshness уже в диапазоне 0-1
        normalized_freshness = metrics.freshness_score

        # Audience size: нормализуем к 0-50 уникальным узлам
        normalized_audience = min(1.0, metrics.audience_size / 50.0)

        # Social engagements: нормализуем к 0-100 взаимодействиям
        normalized_social = min(1.0, metrics.social_engagements / 100.0)

        # Seed coverage уже в диапазоне 0-1
        normalized_seed = metrics.seed_coverage

        return {
            "request_rate": normalized_request_rate,
            "replication_factor": normalized_replication,
            "freshness": normalized_freshness,
            "audience_size": normalized_audience,
            "social_engagements": normalized_social,
            "seed_coverage": normalized_seed,
        }

    def rank_items(
        self, metrics_dict: Dict[bytes, PopularityMetrics], limit: Optional[int] = None
    ) -> List[RankedItem]:
        """
        Ранжирование элементов по популярности

        Args:
            metrics_dict: Словарь метрик (key -> metrics)
            limit: Максимальное количество элементов в результате

        Returns:
            Отсортированный список элементов по убыванию популярности
        """
        ranked_items = []

        for key, metrics in metrics_dict.items():
            score = self.calculate_score(metrics)
            ranked_items.append(RankedItem(key=key, score=score, metrics=metrics))

        # Сортируем по убыванию рейтинга
        ranked_items.sort(reverse=True)

        if limit:
            ranked_items = ranked_items[:limit]

        return ranked_items

    def get_popular_items(
        self, metrics_dict: Dict[bytes, PopularityMetrics], limit: int = 100
    ) -> List[RankedItem]:
        """Получение популярных элементов (score >= popularity_threshold)"""
        ranked = self.rank_items(metrics_dict, limit=None)
        popular = [item for item in ranked if item.score >= self.popularity_threshold]
        return popular[:limit]

    def get_active_items(
        self, metrics_dict: Dict[bytes, PopularityMetrics], limit: int = 100
    ) -> List[RankedItem]:
        """Получение активных элементов (score >= active_threshold)"""
        ranked = self.rank_items(metrics_dict, limit=None)
        active = [item for item in ranked if item.score >= self.active_threshold]
        return active[:limit]
