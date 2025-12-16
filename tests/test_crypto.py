"""
Тесты для криптографических утилит
"""

import pytest

from rhizome.utils.crypto import compute_distance, generate_node_id, hash_key


def test_generate_node_id():
    """Тест генерации Node ID"""
    node_id = generate_node_id()
    assert len(node_id) == 20  # 160 бит = 20 байт


def test_compute_distance():
    """Тест вычисления XOR-расстояния"""
    id1 = b"\x00" * 19 + b"\x01"
    id2 = b"\x00" * 19 + b"\x02"

    distance = compute_distance(id1, id2)
    assert len(distance) == 20
    assert distance[-1] == 0x03  # 0x01 XOR 0x02 = 0x03


def test_hash_key():
    """Тест хэширования ключа"""
    key = "test_key"
    hash_result = hash_key(key)
    assert len(hash_result) == 32  # SHA-256 = 32 байта

    # Проверка детерминированности
    hash_result2 = hash_key(key)
    assert hash_result == hash_result2
