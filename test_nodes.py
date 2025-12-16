#!/usr/bin/env python3
"""
Тестовый скрипт для проверки работы нескольких узлов Rhizome
"""

import asyncio
import sys
from pathlib import Path

from rhizome.config import Config
from rhizome.node.full_node import FullNode
from rhizome.utils.crypto import hash_key
from rhizome.utils.serialization import serialize


async def test_store_and_find():
    """Тест сохранения и поиска данных"""
    print("=" * 60)
    print("Тест STORE и FIND_VALUE операций")
    print("=" * 60)
    
    # Создаем конфигурации для двух узлов
    config1 = Config.from_file(Path("config.yaml"))
    config1.network.listen_port = 8468
    config1.node.node_type = "full"
    
    config2 = Config.from_file(Path("config.yaml"))
    config2.network.listen_port = 8469
    config2.node.node_type = "full"
    config2.node.node_id_file = Path("node_id_2.pem")
    config2.storage.data_dir = Path("data2")
    
    # Добавляем первый узел как bootstrap для второго
    config2.network.bootstrap_nodes = [f"{config1.network.listen_host}:{config1.network.listen_port}"]
    
    # Создаем узлы
    node1 = FullNode(config1)
    node2 = FullNode(config2)
    
    try:
        print("\n1. Запуск узлов...")
        await node1.start()
        await asyncio.sleep(1)  # Даем время на запуск
        await node2.start()
        await asyncio.sleep(2)  # Даем время на bootstrap
        
        print(f"   Node 1 ID: {node1.node_id.id.hex()[:32]}...")
        print(f"   Node 2 ID: {node2.node_id.id.hex()[:32]}...")
        
        # Тест 1: Сохранение данных
        print("\n2. Тест STORE операции...")
        test_key = hash_key("test:key:1")
        test_value = serialize({"message": "Hello, Rhizome!", "timestamp": 1234567890})
        
        print(f"   Ключ: {test_key.hex()[:32]}...")
        print(f"   Значение: {len(test_value)} байт")
        
        # Сохраняем на node1
        result = await node1.dht_protocol.store(test_key, test_value, ttl=3600)
        print(f"   STORE на node1: {'✓' if result else '✗'}")
        
        await asyncio.sleep(1)  # Даем время на распространение
        
        # Тест 2: Поиск данных
        print("\n3. Тест FIND_VALUE операции...")
        try:
            found_value = await node2.dht_protocol.find_value(test_key)
            if found_value:
                print(f"   FIND_VALUE на node2: ✓ (найдено {len(found_value)} байт)")
                # Проверяем содержимое
                from rhizome.utils.serialization import deserialize
                data = deserialize(found_value)
                print(f"   Содержимое: {data}")
            else:
                print("   FIND_VALUE на node2: ✗ (не найдено)")
        except Exception as e:
            print(f"   FIND_VALUE на node2: ✗ (ошибка: {e})")
        
        # Тест 3: Прямой поиск на node1
        print("\n4. Прямой поиск на node1...")
        try:
            found_value = await node1.dht_protocol.find_value(test_key)
            if found_value:
                print(f"   FIND_VALUE на node1: ✓ (найдено {len(found_value)} байт)")
            else:
                print("   FIND_VALUE на node1: ✗ (не найдено)")
        except Exception as e:
            print(f"   FIND_VALUE на node1: ✗ (ошибка: {e})")
        
        # Тест 4: Проверка routing table
        print("\n5. Проверка routing table...")
        nodes1 = node1.routing_table.get_all_nodes()
        nodes2 = node2.routing_table.get_all_nodes()
        print(f"   Узлов в routing table node1: {len(nodes1)}")
        print(f"   Узлов в routing table node2: {len(nodes2)}")
        
        print("\n" + "=" * 60)
        print("Тест завершен")
        print("=" * 60)
        
    except Exception as e:
        print(f"\nОшибка во время теста: {e}")
        import traceback
        traceback.print_exc()
    finally:
        print("\nОстановка узлов...")
        await node2.stop()
        await node1.stop()
        print("Узлы остановлены")


async def test_ping():
    """Тест PING операции"""
    print("=" * 60)
    print("Тест PING операции")
    print("=" * 60)
    
    config1 = Config.from_file(Path("config.yaml"))
    config1.network.listen_port = 8470
    config1.node.node_id_file = Path("node_id_test1.pem")
    config1.storage.data_dir = Path("data_test1")
    
    config2 = Config.from_file(Path("config.yaml"))
    config2.network.listen_port = 8471
    config2.node.node_id_file = Path("node_id_test2.pem")
    config2.storage.data_dir = Path("data_test2")
    config2.network.bootstrap_nodes = [f"{config1.network.listen_host}:{config1.network.listen_port}"]
    
    node1 = FullNode(config1)
    node2 = FullNode(config2)
    
    try:
        await node1.start()
        await asyncio.sleep(1)
        await node2.start()
        await asyncio.sleep(2)
        
        # Создаем узел для ping
        from rhizome.dht.node import Node
        ping_node = Node(
            node_id=node1.node_id,
            address=config1.network.listen_host,
            port=config1.network.listen_port
        )
        
        print("\nТест PING от node2 к node1...")
        result = await node2.dht_protocol.ping(ping_node)
        print(f"Результат: {'✓ Успешно' if result else '✗ Неудачно'}")
        
    finally:
        await node2.stop()
        await node1.stop()


async def main():
    """Главная функция"""
    if len(sys.argv) > 1 and sys.argv[1] == "ping":
        await test_ping()
    else:
        await test_store_and_find()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nПрервано пользователем")
        sys.exit(0)

