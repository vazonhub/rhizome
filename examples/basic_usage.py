#!/usr/bin/env python3
"""
Пример базового использования Rhizome API
"""

import asyncio

from rhizome.storage.data_types import Message, ThreadMetadata
from rhizome.storage.keys import KeyManager
from rhizome.utils.crypto import hash_key
from rhizome.utils.serialization import deserialize, serialize


async def example_store_and_retrieve(dht_protocol):
    """Пример сохранения и получения данных"""
    key_manager = KeyManager()

    # Создаем метаданные треда
    thread_meta = ThreadMetadata(
        id="thread_001",
        title="Пример треда",
        created_at=1234567890,
        creator_pubkey="0xabc123",
        category="технологии",
        tags=["p2p", "dht"],
    )

    # Получаем ключ для метаданных
    meta_key = key_manager.get_thread_meta_key(thread_meta.id)

    # Сериализуем данные
    meta_data = serialize(thread_meta.to_dict())

    # Сохраняем в DHT
    await dht_protocol.store(meta_key, meta_data, ttl=86400)
    print(f"Сохранены метаданные треда: {thread_meta.title}")

    # Получаем данные обратно
    retrieved_data = await dht_protocol.find_value(meta_key)
    if retrieved_data:
        retrieved_dict = deserialize(retrieved_data)
        retrieved_meta = ThreadMetadata.from_dict(retrieved_dict)
        print(f"Получены метаданные: {retrieved_meta.title}")
    else:
        print("Данные не найдены")


async def example_create_message(dht_protocol):
    """Пример создания сообщения"""
    key_manager = KeyManager()

    # Создаем сообщение
    message = Message(
        id="msg_001",
        thread_id="thread_001",
        content="Это пример сообщения в треде",
        author_signature="sig_abc123",
        timestamp=1234567890,
    )

    # Хэшируем сообщение для получения ключа
    message_hash = hash_key(message.id).hex()[:16]
    message_key = key_manager.get_message_key(message_hash)

    # Сохраняем сообщение
    message_data = serialize(message.to_dict())
    await dht_protocol.store(message_key, message_data, ttl=86400)
    print(f"Сохранено сообщение: {message.id}")


async def example_global_index(dht_protocol):
    """Пример работы с глобальными индексами"""
    key_manager = KeyManager()

    # Получаем ключ для глобального списка тредов
    threads_key = key_manager.get_global_threads_key()

    # Список ID тредов
    thread_ids = ["thread_001", "thread_002", "thread_003"]

    # Сохраняем список
    threads_data = serialize(thread_ids)
    await dht_protocol.store(threads_key, threads_data, ttl=2592000)  # 30 дней
    print("Сохранен глобальный список тредов")

    # Получаем список обратно
    retrieved_data = await dht_protocol.find_value(threads_key)
    if retrieved_data:
        thread_list = deserialize(retrieved_data)
        print(f"Найдено тредов: {len(thread_list)}")


if __name__ == "__main__":
    print("Примеры использования Rhizome API")
    print("Для запуска нужен активный узел с dht_protocol")
    print("См. run_node.py для запуска узла")
