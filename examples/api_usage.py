#!/usr/bin/env python3
"""
Пример использования высокоуровневого API Rhizome
Демонстрирует удобный интерфейс для работы с сетью
"""

import asyncio
from pathlib import Path

from rhizome.api import RhizomeClient


async def example_basic_usage(client: RhizomeClient):
    """Базовый пример использования API"""
    print("=" * 60)
    print("Пример: Базовое использование API")
    print("=" * 60)

    print(f"\n✓ Узел уже запущен: {client.get_node_info()}")

    # Создание треда
    print("\n1. Создание треда...")
    thread = await client.create_thread(
        thread_id="example_thread_1",
        title="Пример использования Rhizome API",
        category="документация",
        tags=["api", "пример", "python"],
    )
    print(f"✓ Тред создан: {thread.title}")
    print(f"  ID: {thread.id}")
    print(f"  Категория: {thread.category}")
    print(f"  Теги: {', '.join(thread.tags)}")

    # Добавление сообщений
    print("\n2. Добавление сообщений...")
    msg1 = await client.add_message(
        thread_id="example_thread_1",
        content="Это первое сообщение в треде. API очень удобный!",
        author_signature="user_123",
    )
    print(f"✓ Сообщение #1 добавлено: {msg1.id}")

    msg2 = await client.add_message(
        thread_id="example_thread_1",
        content="Это второе сообщение. Можно легко добавлять ответы!",
        author_signature="user_456",
        parent_id=msg1.id,
    )
    print(f"✓ Сообщение #2 добавлено: {msg2.id} (ответ на {msg1.id[:16]}...)")

    # Поиск треда
    print("\n3. Поиск треда...")
    found_thread = await client.find_thread("example_thread_1")
    if found_thread:
        print(f"✓ Тред найден: {found_thread.title}")
        print(f"  Сообщений: {found_thread.message_count}")
        print(f"  Последняя активность: {found_thread.last_activity}")

    # Обновление глобального индекса
    print("\n4. Обновление глобального индекса...")
    await client.update_global_threads(["example_thread_1"])
    print("✓ Глобальный индекс обновлен")

    # Получение популярных тредов
    print("\n5. Получение популярных тредов...")
    popular = await client.get_popular_threads(limit=5)
    print(f"✓ Найдено популярных элементов: {len(popular)}")
    for item in popular[:3]:
        print(f"  - Ключ: {item['key'][:16]}... | Рейтинг: {item['score']:.2f}")

    print("\n" + "=" * 60)
    print("Пример выполнен успешно!")
    print("=" * 60)


async def example_context_manager(client: RhizomeClient):
    """Пример использования с context manager"""
    print("\n" + "=" * 60)
    print("Пример: Использование с async context manager")
    print("=" * 60)

    print(f"✓ Узел уже запущен: {client.get_node_info()['node_id'][:16]}...")

    # Создание нескольких тредов
    threads = []
    for i in range(3):
        thread = await client.create_thread(
            thread_id=f"thread_{i}",
            title=f"Тред #{i}",
            category="тест",
            tags=["example"],
        )
        threads.append(thread)
        print(f"✓ Создан тред: {thread.title}")

    # Поиск всех тредов
    print("\nПоиск всех тредов...")
    for thread_id in ["thread_0", "thread_1", "thread_2"]:
        found = await client.find_thread(thread_id)
        if found:
            print(f"✓ Найден: {found.title}")


async def example_thread_operations(client: RhizomeClient):
    """Пример операций с тредами"""
    print("\n" + "=" * 60)
    print("Пример: Операции с тредами")
    print("=" * 60)

    # Создание треда
    thread = await client.create_thread(
        thread_id="ops_example",
        title="Операции с тредом",
        category="пример",
        tags=["operations"],
    )

    # Добавление нескольких сообщений
    for i in range(5):
        await client.add_message(
            thread_id="ops_example",
            content=f"Сообщение #{i+1}",
            author_signature=f"author_{i}",
        )

    # Обновление метаданных треда
    updated = await client.update_thread("ops_example", popularity_score=10.5)
    if updated:
        print(f"✓ Тред обновлен: популярность = {updated.popularity_score}")

    # Поиск обновленного треда
    found = await client.find_thread("ops_example")
    if found:
        print(f"✓ Найден тред с {found.message_count} сообщениями")
        print(f"  Популярность: {found.popularity_score}")


async def example_search(client: RhizomeClient):
    """Пример поиска тредов"""
    print("\n" + "=" * 60)
    print("Пример: Поиск тредов")
    print("=" * 60)

    # Создание нескольких тредов в разных категориях
    await client.create_thread("tech_1", "Технологии #1", category="технологии", tags=["python"])
    await client.create_thread("tech_2", "Технологии #2", category="технологии", tags=["rust"])
    await client.create_thread("science_1", "Наука #1", category="наука", tags=["python"])

    # Добавление в глобальный индекс
    await client.update_global_threads(["tech_1", "tech_2", "science_1"])

    # Поиск по категории
    tech_threads = await client.search_threads(category="технологии")
    print(f"✓ Найдено тредов в категории 'технологии': {len(tech_threads)}")
    for thread in tech_threads:
        print(f"  - {thread.title}")

    # Поиск по тегам
    python_threads = await client.search_threads(tags=["python"])
    print(f"\n✓ Найдено тредов с тегом 'python': {len(python_threads)}")
    for thread in python_threads:
        print(f"  - {thread.title} ({thread.category})")


async def main():
    """Главная функция"""
    print("Rhizome API - Примеры использования")
    print("=" * 60)

    config_path = Path("config.yaml")
    config_file = str(config_path) if config_path.exists() else None

    # Используем один клиент для всех примеров, чтобы избежать конфликтов портов
    async with RhizomeClient(config_path=config_file) as client:
        print("\n✓ Узел запущен, начинаем примеры...\n")

        # Пример 1: Базовое использование
        await example_basic_usage(client)

        # Пример 2: Context manager демонстрация
        await example_context_manager(client)

        # Пример 3: Операции с тредами
        await example_thread_operations(client)

        # Пример 4: Поиск
        await example_search(client)

    print("\n" + "=" * 60)
    print("Все примеры выполнены!")
    print("✓ Узел остановлен автоматически")
    print("=" * 60)


if __name__ == "__main__":
    asyncio.run(main())
