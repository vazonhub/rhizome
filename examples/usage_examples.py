#!/usr/bin/env python3
"""
Примеры использования Rhizome API
"""

import asyncio
import time
from pathlib import Path

from rhizome.config import Config
from rhizome.node.full_node import FullNode
from rhizome.storage.data_types import Message, ThreadMetadata
from rhizome.storage.keys import KeyManager
from rhizome.utils.crypto import hash_key
from rhizome.utils.serialization import deserialize, serialize


async def example_create_thread(node):
    """Пример создания и сохранения треда"""
    print("\n=== Пример: Создание треда ===")

    key_manager = KeyManager()

    # Создаем метаданные треда
    thread_meta = ThreadMetadata(
        id="thread_001",
        title="Пример треда в Rhizome",
        created_at=int(time.time()),
        creator_pubkey="0xabc123def456",
        category="технологии",
        tags=["p2p", "dht", "python"],
    )

    # Получаем ключ для метаданных
    meta_key = key_manager.get_thread_meta_key(thread_meta.id)

    # Сериализуем и сохраняем
    meta_data = serialize(thread_meta.to_dict())
    success = await node.store(meta_key, meta_data, ttl=86400)

    if success:
        print(f"✓ Тред сохранен: {thread_meta.title}")
        print(f"  ID: {thread_meta.id}")
        print(f"  Категория: {thread_meta.category}")
    else:
        print("✗ Ошибка при сохранении треда")


async def example_find_thread(node, thread_id):
    """Пример поиска треда"""
    print(f"\n=== Пример: Поиск треда '{thread_id}' ===")

    key_manager = KeyManager()
    meta_key = key_manager.get_thread_meta_key(thread_id)

    try:
        data = await node.find_value(meta_key)
        if data:
            thread_dict = deserialize(data)
            thread_meta = ThreadMetadata.from_dict(thread_dict)

            print(f"✓ Тред найден:")
            print(f"  Название: {thread_meta.title}")
            print(f"  Создан: {time.ctime(thread_meta.created_at)}")
            print(f"  Категория: {thread_meta.category}")
            print(f"  Теги: {', '.join(thread_meta.tags)}")
        else:
            print("✗ Тред не найден")
    except Exception as e:
        print(f"✗ Ошибка при поиске: {e}")


async def example_create_message(node, thread_id):
    """Пример создания сообщения в треде"""
    print(f"\n=== Пример: Создание сообщения в треде '{thread_id}' ===")

    key_manager = KeyManager()

    # Создаем сообщение
    message = Message(
        id="msg_001",
        thread_id=thread_id,
        content="Это пример сообщения в треде. Rhizome - это децентрализованная P2P сеть!",
        author_signature="sig_abc123",
        timestamp=int(time.time()),
        content_type="text/markdown",
    )

    # Хэшируем ID сообщения для получения ключа
    message_hash = hash_key(message.id).hex()[:16]
    message_key = key_manager.get_message_key(message_hash)

    # Сохраняем сообщение
    message_data = serialize(message.to_dict())
    success = await node.store(message_key, message_data, ttl=86400)

    if success:
        print(f"✓ Сообщение сохранено:")
        print(f"  ID: {message.id}")
        print(f"  Тред: {message.thread_id}")
        print(f"  Содержимое: {message.content[:50]}...")
    else:
        print("✗ Ошибка при сохранении сообщения")


async def example_global_index(node):
    """Пример работы с глобальными индексами"""
    print("\n=== Пример: Глобальный индекс тредов ===")

    key_manager = KeyManager()

    # Получаем ключ для глобального списка тредов
    threads_key = key_manager.get_global_threads_key()

    # Список ID тредов
    thread_ids = ["thread_001", "thread_002", "thread_003"]

    # Сохраняем список
    threads_data = serialize(thread_ids)
    success = await node.store(threads_key, threads_data, ttl=2592000)  # 30 дней

    if success:
        print("✓ Глобальный список тредов сохранен")

        # Получаем список обратно
        try:
            retrieved_data = await node.find_value(threads_key)
            if retrieved_data:
                thread_list = deserialize(retrieved_data)
                print(f"✓ Найдено тредов: {len(thread_list)}")
                for tid in thread_list:
                    print(f"  - {tid}")
        except Exception as e:
            print(f"✗ Ошибка при получении списка: {e}")
    else:
        print("✗ Ошибка при сохранении глобального индекса")


async def example_popularity_check(node):
    """Пример проверки популярности данных"""
    print("\n=== Пример: Проверка популярности ===")

    # Получаем метрики популярности
    all_metrics = node.metrics_collector.get_all_metrics()

    if all_metrics:
        print(f"✓ Найдено метрик: {len(all_metrics)}")

        # Ранжируем элементы
        ranked = node.popularity_ranker.rank_items(all_metrics, limit=10)

        print("\nТоп-10 популярных элементов:")
        for i, item in enumerate(ranked, 1):
            print(f"  {i}. Ключ: {item.key.hex()[:16]}... | Рейтинг: {item.score:.2f}")
    else:
        print("✗ Нет данных для ранжирования")


async def main():
    """Главная функция с примерами"""
    print("=" * 60)
    print("Примеры использования Rhizome API")
    print("=" * 60)

    # Загрузка конфигурации
    config_path = Path("config.yaml")
    if not config_path.exists():
        print(f"Ошибка: Файл конфигурации не найден: {config_path}")
        print("Создайте config.yaml или скопируйте из config.yaml.example")
        return

    config = Config.from_file(config_path)

    # Создание узла
    print("\nИнициализация узла...")
    node = FullNode(config)

    try:
        # Запуск узла
        print("Запуск узла...")
        await node.start()
        print("✓ Узел запущен")
        print(f"  Node ID: {node.node_id.id.hex()[:32]}...")
        print(f"  Тип: {node.node_type}")
        print(f"  Адрес: {config.network.listen_host}:{config.network.listen_port}")

        # Ждем немного для инициализации
        await asyncio.sleep(2)

        # Примеры использования
        await example_create_thread(node)
        await asyncio.sleep(1)

        await example_find_thread(node, "thread_001")
        await asyncio.sleep(1)

        await example_create_message(node, "thread_001")
        await asyncio.sleep(1)

        await example_global_index(node)
        await asyncio.sleep(1)

        await example_popularity_check(node)

        print("\n" + "=" * 60)
        print("Примеры выполнены успешно!")
        print("Нажмите Ctrl+C для остановки узла")
        print("=" * 60)

        # Ожидание
        while node.is_running:
            await asyncio.sleep(1)

    except KeyboardInterrupt:
        print("\n\nОстановка узла...")
        await node.stop()
        print("✓ Узел остановлен")
    except Exception as e:
        print(f"\n✗ Ошибка: {e}")
        import traceback

        traceback.print_exc()
        await node.stop()


if __name__ == "__main__":
    asyncio.run(main())
