"""
Сбор метрик популярности
"""

import time
from collections import defaultdict, deque
from dataclasses import dataclass, field
from typing import Dict, Optional, Set

from rhizome.logger import get_logger


@dataclass
class PopularityMetrics:
    """Метрики популярности для ключа"""

    key: bytes

    # Базовые метрики
    request_count: int = 0  # Общее количество запросов
    request_rate: float = 0.0  # Запросов в час
    replication_count: int = 1  # На скольких узлах хранится (начинаем с 1 - локально)
    freshness_score: float = 1.0  # Свежесть (1.0 = новое, 0.0 = старое)
    audience_size: int = 1  # Уникальные узлы, запрашивающие данные

    # Расширенные метрики
    social_engagements: int = 0  # Ответы, цитаты, упоминания
    view_time: float = 0.0  # Время просмотра (если доступно)
    seed_coverage: float = 0.0  # Доля seed-узлов, хранящих данные

    # Временные метки
    first_seen: float = field(default_factory=time.time)
    last_request: float = field(default_factory=time.time)
    created_at: Optional[float] = None  # Время создания данных (если известно)

    # История запросов (для расчета request_rate)
    request_timestamps: deque = field(default_factory=lambda: deque(maxlen=1000))
    requesting_nodes: Set[bytes] = field(default_factory=set)

    def update_request(self, node_id: Optional[bytes] = None):
        """Обновление метрик при запросе"""
        current_time = time.time()
        self.request_count += 1
        self.last_request = current_time
        self.request_timestamps.append(current_time)

        if node_id:
            self.requesting_nodes.add(node_id)
            self.audience_size = len(self.requesting_nodes)

        # Пересчитываем request_rate (запросов в час)
        if len(self.request_timestamps) > 1:
            time_span = self.request_timestamps[-1] - self.request_timestamps[0]
            if time_span > 0:
                self.request_rate = (len(self.request_timestamps) / time_span) * 3600
            else:
                self.request_rate = len(self.request_timestamps) * 3600
        else:
            self.request_rate = 1.0 if self.request_count > 0 else 0.0

    def update_freshness(self, age_seconds: Optional[float] = None):
        """Обновление метрики свежести"""
        if age_seconds is None:
            if self.created_at:
                age_seconds = time.time() - self.created_at
            else:
                age_seconds = time.time() - self.first_seen

        # Свежесть: 1.0 для новых данных, экспоненциально убывает
        # Половина через 24 часа, четверть через 7 дней
        if age_seconds < 3600:  # Меньше часа
            self.freshness_score = 1.0
        elif age_seconds < 86400:  # Меньше суток
            self.freshness_score = 1.0 - (age_seconds / 86400) * 0.5
        else:  # Больше суток
            days = age_seconds / 86400
            self.freshness_score = max(0.1, 0.5 * (0.5 ** (days / 7)))

    def update_replication(self, count: int):
        """Обновление количества репликаций"""
        self.replication_count = max(self.replication_count, count)

    def update_social_engagement(self, count: int = 1):
        """Обновление социальных взаимодействий"""
        self.social_engagements += count

    def to_dict(self) -> Dict:
        """Преобразование в словарь"""
        return {
            "key": self.key.hex(),
            "request_count": self.request_count,
            "request_rate": self.request_rate,
            "replication_count": self.replication_count,
            "freshness_score": self.freshness_score,
            "audience_size": self.audience_size,
            "social_engagements": self.social_engagements,
            "view_time": self.view_time,
            "seed_coverage": self.seed_coverage,
            "first_seen": self.first_seen,
            "last_request": self.last_request,
            "created_at": self.created_at,
        }

    @classmethod
    def from_dict(cls, data: Dict) -> "PopularityMetrics":
        """Создание из словаря"""
        metrics = cls(key=bytes.fromhex(data["key"]))
        metrics.request_count = data.get("request_count", 0)
        metrics.request_rate = data.get("request_rate", 0.0)
        metrics.replication_count = data.get("replication_count", 1)
        metrics.freshness_score = data.get("freshness_score", 1.0)
        metrics.audience_size = data.get("audience_size", 1)
        metrics.social_engagements = data.get("social_engagements", 0)
        metrics.view_time = data.get("view_time", 0.0)
        metrics.seed_coverage = data.get("seed_coverage", 0.0)
        metrics.first_seen = data.get("first_seen", time.time())
        metrics.last_request = data.get("last_request", time.time())
        metrics.created_at = data.get("created_at")
        return metrics


class MetricsCollector:
    """Сборщик метрик популярности"""

    def __init__(self):
        self.metrics: Dict[bytes, PopularityMetrics] = {}
        self.logger = get_logger("popularity.metrics")

    def record_find_value(self, key: bytes, node_id: Optional[bytes] = None):
        """Запись запроса FIND_VALUE"""
        if key not in self.metrics:
            self.metrics[key] = PopularityMetrics(key=key)

        metrics = self.metrics[key]
        metrics.update_request(node_id)
        metrics.update_freshness()

        self.logger.debug("Recorded FIND_VALUE", key=key.hex()[:16])

    def record_store(self, key: bytes, replication_count: int = 1):
        """Запись операции STORE"""
        if key not in self.metrics:
            self.metrics[key] = PopularityMetrics(key=key)

        metrics = self.metrics[key]
        metrics.update_replication(replication_count)
        metrics.update_freshness()

        self.logger.debug("Recorded STORE", key=key.hex()[:16], replication=replication_count)

    def record_social_engagement(self, key: bytes, count: int = 1):
        """Запись социального взаимодействия"""
        if key not in self.metrics:
            self.metrics[key] = PopularityMetrics(key=key)

        metrics = self.metrics[key]
        metrics.update_social_engagement(count)

        self.logger.debug("Recorded social engagement", key=key.hex()[:16], count=count)

    def get_metrics(self, key: bytes) -> Optional[PopularityMetrics]:
        """Получение метрик для ключа"""
        return self.metrics.get(key)

    def get_all_metrics(self) -> Dict[bytes, PopularityMetrics]:
        """Получение всех метрик"""
        return self.metrics.copy()

    def update_all_freshness(self):
        """Обновление свежести для всех метрик"""
        for metrics in self.metrics.values():
            metrics.update_freshness()

    def cleanup_old_metrics(self, max_age_days: int = 30):
        """Очистка старых метрик"""
        current_time = time.time()
        max_age = max_age_days * 86400

        keys_to_remove = []
        for key, metrics in self.metrics.items():
            if (current_time - metrics.last_request) > max_age:
                keys_to_remove.append(key)

        for key in keys_to_remove:
            del self.metrics[key]

        if keys_to_remove:
            self.logger.info("Cleaned up old metrics", count=len(keys_to_remove))
