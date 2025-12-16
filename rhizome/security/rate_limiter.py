"""
Rate limiting для защиты от атак
"""

import time
from collections import defaultdict, deque
from typing import Dict, Optional, Tuple

from rhizome.exceptions import RateLimitError
from rhizome.logger import get_logger


class RateLimiter:
    """Rate limiter для ограничения частоты запросов"""

    def __init__(self, max_requests: int = 100, window_seconds: int = 60, per_node_limit: int = 20):
        """
        Инициализация rate limiter

        Args:
            max_requests: Максимальное количество запросов в окне
            window_seconds: Размер окна в секундах
            per_node_limit: Лимит запросов от одного узла в окне
        """
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self.per_node_limit = per_node_limit

        # История запросов: (timestamp, node_id)
        self.request_history: deque = deque(maxlen=max_requests * 2)

        # Запросы по узлам
        self.node_requests: Dict[bytes, deque] = defaultdict(
            lambda: deque(maxlen=per_node_limit * 2)
        )

        self.logger = get_logger("security.rate_limiter")

    def check_rate_limit(self, node_id: Optional[bytes] = None) -> bool:
        """
        Проверка rate limit

        Args:
            node_id: ID узла (опционально, для per-node лимита)

        Returns:
            True если запрос разрешен, False если превышен лимит

        Raises:
            RateLimitError если превышен лимит
        """
        current_time = time.time()

        # Очищаем старые запросы
        self._cleanup_old_requests(current_time)

        # Проверяем общий лимит
        recent_requests = sum(
            1
            for timestamp in self.request_history
            if current_time - timestamp < self.window_seconds
        )

        if recent_requests >= self.max_requests:
            self.logger.warning(
                "Rate limit exceeded", requests=recent_requests, limit=self.max_requests
            )
            raise RateLimitError(
                f"Rate limit exceeded: {recent_requests}/{self.max_requests} requests in {self.window_seconds}s"
            )

        # Проверяем per-node лимит
        if node_id:
            node_recent = sum(
                1
                for timestamp in self.node_requests[node_id]
                if current_time - timestamp < self.window_seconds
            )

            if node_recent >= self.per_node_limit:
                self.logger.warning(
                    "Per-node rate limit exceeded",
                    node_id=node_id.hex()[:16],
                    requests=node_recent,
                    limit=self.per_node_limit,
                )
                raise RateLimitError(
                    f"Per-node rate limit exceeded: {node_recent}/{self.per_node_limit} requests"
                )

        # Запрос разрешен, добавляем в историю
        self.request_history.append(current_time)
        if node_id:
            self.node_requests[node_id].append(current_time)

        return True

    def _cleanup_old_requests(self, current_time: float):
        """Очистка старых запросов из истории"""
        # Очищаем общую историю
        while (
            self.request_history and (current_time - self.request_history[0]) > self.window_seconds
        ):
            self.request_history.popleft()

        # Очищаем историю по узлам
        for node_id in list(self.node_requests.keys()):
            node_history = self.node_requests[node_id]
            while node_history and (current_time - node_history[0]) > self.window_seconds:
                node_history.popleft()

            # Удаляем пустые истории
            if not node_history:
                del self.node_requests[node_id]

    def get_stats(self) -> Dict[str, int]:
        """Получение статистики rate limiter"""
        current_time = time.time()
        self._cleanup_old_requests(current_time)

        recent_requests = sum(
            1
            for timestamp in self.request_history
            if current_time - timestamp < self.window_seconds
        )

        return {
            "recent_requests": recent_requests,
            "max_requests": self.max_requests,
            "window_seconds": self.window_seconds,
            "active_nodes": len(self.node_requests),
        }
