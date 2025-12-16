"""
Интеграционные тесты для DHT операций
"""

import asyncio
import shutil
import tempfile
from pathlib import Path

import pytest

from rhizome.config import (
    Config,
    DHTConfig,
    NetworkConfig,
    NodeConfig,
    PopularityConfig,
    SecurityConfig,
    StorageConfig,
)
from rhizome.dht.node import Node, NodeID
from rhizome.dht.protocol import DHTProtocol
from rhizome.dht.routing_table import RoutingTable
from rhizome.exceptions import ValueNotFoundError
from rhizome.storage.storage import Storage
from rhizome.utils.crypto import generate_node_id, hash_key


@pytest.fixture
def temp_dir():
    """Временная директория для тестов"""
    temp_path = Path(tempfile.mkdtemp())
    yield temp_path
    shutil.rmtree(temp_path)


@pytest.fixture
def storage_config(temp_dir):
    """Конфигурация хранилища для тестов"""
    return StorageConfig(
        data_dir=temp_dir / "storage",
        max_storage_size=100 * 1024 * 1024,  # 100 MB
        default_ttl=3600,
    )


@pytest.fixture
def dht_protocol(storage_config):
    """DHT протокол для тестов (без сетевого протокола)"""
    node_id = NodeID(id=generate_node_id())
    routing_table = RoutingTable(node_id, k=20, bucket_count=160)
    storage = Storage(storage_config)
    return DHTProtocol(routing_table, storage)


@pytest.mark.asyncio
async def test_store_and_get_local(dht_protocol):
    """Тест локального сохранения и получения"""
    key = hash_key("test_key")
    value = b"test_value"

    # Сохраняем
    result = await dht_protocol.store(key, value, ttl=3600)
    assert result is True

    # Получаем
    retrieved = await dht_protocol.storage.get(key)
    assert retrieved == value


@pytest.mark.asyncio
async def test_find_value_local(dht_protocol):
    """Тест поиска значения в локальном хранилище"""
    key = hash_key("test_key")
    value = b"test_value"

    # Сохраняем
    await dht_protocol.store(key, value, ttl=3600)

    # Ищем
    found = await dht_protocol.find_value(key)
    assert found == value


@pytest.mark.asyncio
async def test_find_value_not_found(dht_protocol):
    """Тест поиска несуществующего значения"""
    key = hash_key("nonexistent_key")

    # Ищем несуществующий ключ
    with pytest.raises(ValueNotFoundError):
        await dht_protocol.find_value(key)


@pytest.mark.asyncio
async def test_routing_table_add_node(dht_protocol):
    """Тест добавления узлов в routing table"""
    # Создаем несколько узлов
    nodes = []
    for i in range(5):
        node_id = NodeID(id=generate_node_id())
        node = Node(node_id=node_id, address="127.0.0.1", port=8468 + i)
        nodes.append(node)

    # Добавляем узлы
    for node in nodes:
        result = dht_protocol.routing_table.add_node(node)
        assert result is True

    # Проверяем, что узлы добавлены
    all_nodes = dht_protocol.routing_table.get_all_nodes()
    assert len(all_nodes) >= len(nodes)


@pytest.mark.asyncio
async def test_find_closest_nodes(dht_protocol):
    """Тест поиска ближайших узлов"""
    # Добавляем несколько узлов
    target_id = NodeID(id=generate_node_id())
    nodes = []
    for i in range(10):
        node_id = NodeID(id=generate_node_id())
        node = Node(node_id=node_id, address="127.0.0.1", port=8468 + i)
        nodes.append(node)
        dht_protocol.routing_table.add_node(node)

    # Ищем ближайшие узлы
    closest = dht_protocol.routing_table.find_closest_nodes(target_id, count=5)
    assert len(closest) <= 5
    assert len(closest) > 0
