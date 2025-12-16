"""
Тесты для DHT модуля
"""

import pytest

from rhizome.dht.node import Node, NodeID
from rhizome.dht.routing_table import KBucket, RoutingTable
from rhizome.utils.crypto import compute_distance, generate_node_id


def test_node_id():
    """Тест создания Node ID"""
    node_id_bytes = generate_node_id()
    node_id = NodeID(id=node_id_bytes)
    assert len(node_id.id) == 20


def test_node_id_distance():
    """Тест вычисления расстояния между Node ID"""
    id1 = NodeID(id=b"\x00" * 19 + b"\x01")
    id2 = NodeID(id=b"\x00" * 19 + b"\x02")

    distance = id1.distance_to(id2)
    assert len(distance) == 20
    assert distance[-1] == 0x03  # 0x01 XOR 0x02 = 0x03


def test_node():
    """Тест создания узла"""
    node_id = NodeID(id=generate_node_id())
    node = Node(node_id=node_id, address="127.0.0.1", port=8468)

    assert node.node_id == node_id
    assert node.address == "127.0.0.1"
    assert node.port == 8468
    assert node.failed_pings == 0


def test_kbucket():
    """Тест k-бакета"""
    bucket = KBucket(k=3)
    node_id = NodeID(id=generate_node_id())
    node = Node(node_id=node_id, address="127.0.0.1", port=8468)

    # Добавление узлов
    assert bucket.add_node(node) is True
    assert len(bucket.nodes) == 1
    assert bucket.is_full() is False

    # Добавление до заполнения
    for i in range(2):
        node_id = NodeID(id=generate_node_id())
        node = Node(node_id=node_id, address="127.0.0.1", port=8468 + i + 1)
        bucket.add_node(node)

    assert bucket.is_full() is True
    assert len(bucket.nodes) == 3

    # Попытка добавить еще один узел
    node_id = NodeID(id=generate_node_id())
    node = Node(node_id=node_id, address="127.0.0.1", port=8471)
    assert bucket.add_node(node) is False


def test_routing_table():
    """Тест таблицы маршрутизации"""
    node_id = NodeID(id=generate_node_id())
    routing_table = RoutingTable(node_id, k=3, bucket_count=160)

    # Добавление узлов
    for i in range(5):
        other_id = NodeID(id=generate_node_id())
        other_node = Node(node_id=other_id, address="127.0.0.1", port=8468 + i)
        routing_table.add_node(other_node)

    # Поиск ближайших узлов
    target_id = NodeID(id=generate_node_id())
    closest = routing_table.find_closest_nodes(target_id, count=3)
    assert len(closest) <= 3

    # Получение всех узлов
    all_nodes = routing_table.get_all_nodes()
    assert len(all_nodes) == 5
