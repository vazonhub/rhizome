"""
Тесты для системы популярности
"""

import time

import pytest

from rhizome.popularity.metrics import MetricsCollector, PopularityMetrics
from rhizome.popularity.ranking import PopularityRanker
from rhizome.utils.crypto import hash_key


def test_metrics_collector():
    """Тест сборщика метрик"""
    collector = MetricsCollector()
    key = hash_key("test_key")

    # Записываем несколько запросов
    collector.record_find_value(key)
    collector.record_find_value(key)
    collector.record_find_value(key)

    metrics = collector.get_metrics(key)
    assert metrics is not None
    assert metrics.request_count == 3
    assert metrics.request_rate > 0


def test_popularity_metrics_update():
    """Тест обновления метрик"""
    key = hash_key("test_key")
    metrics = PopularityMetrics(key=key)

    # Обновляем запросы
    metrics.update_request()
    metrics.update_request()

    assert metrics.request_count == 2
    assert metrics.request_rate > 0

    # Обновляем свежесть
    metrics.update_freshness(age_seconds=3600)  # 1 час
    assert metrics.freshness_score > 0.5

    metrics.update_freshness(age_seconds=86400)  # 1 день
    assert metrics.freshness_score < 0.5


def test_popularity_ranker():
    """Тест ранжирования"""
    ranker = PopularityRanker(popularity_threshold=7.0, active_threshold=5.0)

    # Создаем несколько метрик с разной популярностью
    metrics_dict = {}

    # Популярный элемент
    key1 = hash_key("popular_key")
    metrics1 = PopularityMetrics(key=key1)
    for _ in range(50):
        metrics1.update_request()
    metrics1.update_replication(15)
    metrics1.update_freshness(age_seconds=3600)
    metrics_dict[key1] = metrics1

    # Обычный элемент
    key2 = hash_key("normal_key")
    metrics2 = PopularityMetrics(key=key2)
    metrics2.update_request()
    metrics2.update_replication(3)
    metrics2.update_freshness(age_seconds=86400)
    metrics_dict[key2] = metrics2

    # Ранжируем
    ranked = ranker.rank_items(metrics_dict)

    assert len(ranked) == 2
    assert ranked[0].key == key1  # Популярный должен быть первым
    assert ranked[0].score > ranked[1].score

    # Проверяем популярные элементы
    popular = ranker.get_popular_items(metrics_dict)
    assert len(popular) >= 1
    assert popular[0].key == key1


def test_adaptive_weights():
    """Тест адаптивных весов"""
    ranker = PopularityRanker()

    # Новый контент (меньше 24 часов)
    key_new = hash_key("new_key")
    metrics_new = PopularityMetrics(key=key_new)
    metrics_new.update_freshness(age_seconds=3600)  # 1 час
    score_new = ranker.calculate_score(metrics_new, adaptive_weights=True)

    # Старый контент (больше 7 дней)
    key_old = hash_key("old_key")
    metrics_old = PopularityMetrics(key=key_old)
    metrics_old.update_freshness(age_seconds=604800)  # 7 дней
    score_old = ranker.calculate_score(metrics_old, adaptive_weights=True)

    # Новый контент должен иметь преимущество в свежести
    assert metrics_new.freshness_score > metrics_old.freshness_score


def test_metrics_cleanup():
    """Тест очистки старых метрик"""
    collector = MetricsCollector()

    key1 = hash_key("key1")
    key2 = hash_key("key2")

    collector.record_find_value(key1)
    collector.record_find_value(key2)

    # Симулируем старые метрики
    metrics2 = collector.get_metrics(key2)
    metrics2.last_request = time.time() - (31 * 86400)  # 31 день назад

    assert len(collector.get_all_metrics()) == 2

    # Очищаем старые метрики
    collector.cleanup_old_metrics(max_age_days=30)

    # key2 должен быть удален
    assert len(collector.get_all_metrics()) == 1
    assert collector.get_metrics(key1) is not None
    assert collector.get_metrics(key2) is None
