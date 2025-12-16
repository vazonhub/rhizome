#!/usr/bin/env python3
"""
Полный пример работы с двумя узлами Rhizome
Демонстрирует максимальный функционал: создание узлов, обмен данными,
обновления, популярность, репликацию
"""

import asyncio
import time
from pathlib import Path

from rhizome.config import Config
from rhizome.node.full_node import FullNode
from rhizome.storage.keys import KeyManager
from rhizome.storage.data_types import ThreadMetadata, Message
from rhizome.utils.crypto import hash_key
from rhizome.utils.serialization import serialize, deserialize


class Colors:
    """Цвета для вывода в терминал"""
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    END = '\033[0m'
    BOLD = '\033[1m'


def print_header(text: str):
    """Красивый заголовок"""
    print(f"\n{Colors.HEADER}{Colors.BOLD}{'=' * 70}{Colors.END}")
    print(f"{Colors.HEADER}{Colors.BOLD}{text.center(70)}{Colors.END}")
    print(f"{Colors.HEADER}{Colors.BOLD}{'=' * 70}{Colors.END}\n")


def print_success(text: str):
    """Успешное сообщение"""
    print(f"{Colors.GREEN}✓ {text}{Colors.END}")


def print_info(text: str):
    """Информационное сообщение"""
    print(f"{Colors.CYAN}ℹ {text}{Colors.END}")


def print_warning(text: str):
    """Предупреждение"""
    print(f"{Colors.YELLOW}⚠ {text}{Colors.END}")


def print_error(text: str):
    """Ошибка"""
    print(f"{Colors.RED}✗ {text}{Colors.END}")


async def setup_nodes():
    """Настройка двух узлов"""
    print_header("Инициализация узлов")
    
    # Загружаем базовую конфигурацию
    base_config_path = Path("config.yaml")
    if not base_config_path.exists():
        print_error(f"Конфигурационный файл не найден: {base_config_path}")
        print_info("Создайте config.yaml или скопируйте из примера")
        return None, None
    
    # Конфигурация первого узла
    config1 = Config.from_file(base_config_path)
    config1.network.listen_port = 8468
    config1.node.node_type = "full"
    config1.node.node_id_file = Path("node_id_1.pem")
    config1.storage.data_dir = Path("data_node1")
    
    # Конфигурация второго узла
    config2 = Config.from_file(base_config_path)
    config2.network.listen_port = 8469
    config2.node.node_type = "full"
    config2.node.node_id_file = Path("node_id_2.pem")
    config2.storage.data_dir = Path("data_node2")
    
    # Второй узел подключается к первому через bootstrap
    config2.network.bootstrap_nodes = [
        f"{config1.network.listen_host}:{config1.network.listen_port}"
    ]
    
    # Создаем узлы
    node1 = FullNode(config1)
    node2 = FullNode(config2)
    
    print_success(f"Узел 1 создан (порт {config1.network.listen_port})")
    print_success(f"Узел 2 создан (порт {config2.network.listen_port})")
    
    return node1, node2


async def start_nodes(node1: FullNode, node2: FullNode):
    """Запуск узлов и подключение"""
    print_header("Запуск узлов")
    
    print_info("Запуск узла 1...")
    await node1.start()
    print_success(f"Узел 1 запущен")
    print_info(f"  Node ID: {node1.node_id.id.hex()[:32]}...")
    print_info(f"  Адрес: {node1.config.network.listen_host}:{node1.config.network.listen_port}")
    
    await asyncio.sleep(1)  # Даем время на полный запуск
    
    print_info("Запуск узла 2 и подключение к узлу 1...")
    await node2.start()
    print_success("Узел 2 запущен")
    print_info(f"  Node ID: {node2.node_id.id.hex()[:32]}...")
    print_info(f"  Адрес: {node2.config.network.listen_host}:{node2.config.network.listen_port}")
    
    # Ждем bootstrap
    await asyncio.sleep(3)
    
    # Проверяем routing tables
    nodes1 = node1.routing_table.get_all_nodes()
    nodes2 = node2.routing_table.get_all_nodes()
    
    print_info(f"Узлов в routing table узла 1: {len(nodes1)}")
    print_info(f"Узлов в routing table узла 2: {len(nodes2)}")
    
    if len(nodes2) > 0:
        print_success("Узлы успешно подключены друг к другу!")
    else:
        print_warning("Узлы пока не видят друг друга (это нормально для изолированной сети)")


async def create_threads_on_node1(node1: FullNode):
    """Создание тредов на первом узле"""
    print_header("Создание тредов на узле 1")
    
    key_manager = KeyManager()
    threads = []
    
    # Создаем несколько тредов
    thread_configs = [
        {
            "id": "thread_p2p",
            "title": "Децентрализованные P2P сети",
            "category": "технологии",
            "tags": ["p2p", "dht", "blockchain"],
            "creator": "alice"
        },
        {
            "id": "thread_ai",
            "title": "Искусственный интеллект и машинное обучение",
            "category": "наука",
            "tags": ["ai", "ml", "neural-networks"],
            "creator": "bob"
        },
        {
            "id": "thread_crypto",
            "title": "Криптография и безопасность",
            "category": "безопасность",
            "tags": ["crypto", "security", "privacy"],
            "creator": "charlie"
        }
    ]
    
    for thread_cfg in thread_configs:
        thread_meta = ThreadMetadata(
            id=thread_cfg["id"],
            title=thread_cfg["title"],
            created_at=int(time.time()),
            creator_pubkey=f"0x{thread_cfg['creator']}_pubkey",
            category=thread_cfg["category"],
            tags=thread_cfg["tags"]
        )
        
        meta_key = key_manager.get_thread_meta_key(thread_meta.id)
        meta_data = serialize(thread_meta.to_dict())
        
        success = await node1.store(meta_key, meta_data, ttl=86400)
        
        if success:
            print_success(f"Тред '{thread_meta.title}' создан (ID: {thread_meta.id})")
            threads.append(thread_meta)
        else:
            print_error(f"Ошибка создания треда '{thread_meta.title}'")
        
        await asyncio.sleep(0.5)
    
    return threads


async def create_messages_on_node1(node1: FullNode, thread_id: str, count: int = 3):
    """Создание сообщений в треде на первом узле"""
    print_header(f"Создание {count} сообщений в треде '{thread_id}' на узле 1")
    
    key_manager = KeyManager()
    messages = []
    
    for i in range(1, count + 1):
        message = Message(
            id=f"msg_{thread_id}_{i}",
            thread_id=thread_id,
            content=f"Это сообщение #{i} в треде {thread_id}. "
                   f"Rhizome - это децентрализованная P2P сеть для обмена данными!",
            author_signature=f"sig_author_{i}",
            timestamp=int(time.time()) + i,
            content_type="text/markdown"
        )
        
        message_hash = hash_key(message.id).hex()[:16]
        message_key = key_manager.get_message_key(message_hash)
        message_data = serialize(message.to_dict())
        
        success = await node1.store(message_key, message_data, ttl=86400)
        
        if success:
            print_success(f"Сообщение #{i} создано: {message.content[:50]}...")
            messages.append(message)
        else:
            print_error(f"Ошибка создания сообщения #{i}")
        
        await asyncio.sleep(0.3)
    
    return messages


async def find_data_on_node2(node2: FullNode, thread_id: str):
    """Поиск данных на втором узле"""
    print_header(f"Поиск треда '{thread_id}' на узле 2")
    
    key_manager = KeyManager()
    meta_key = key_manager.get_thread_meta_key(thread_id)
    
    try:
        data = await node2.find_value(meta_key)
        if data:
            thread_dict = deserialize(data)
            thread_meta = ThreadMetadata.from_dict(thread_dict)
            
            print_success("Тред найден на узле 2!")
            print_info(f"  Название: {thread_meta.title}")
            print_info(f"  Категория: {thread_meta.category}")
            print_info(f"  Теги: {', '.join(thread_meta.tags)}")
            print_info(f"  Создан: {time.ctime(thread_meta.created_at)}")
            print_info(f"  Создатель: {thread_meta.creator_pubkey}")
            
            return thread_meta
        else:
            print_warning("Тред не найден на узле 2")
            return None
    except Exception as e:
        print_error(f"Ошибка при поиске: {e}")
        return None


async def update_thread_on_node1(node1: FullNode, thread_id: str):
    """Обновление треда на первом узле (добавление новых сообщений)"""
    print_header(f"Обновление треда '{thread_id}' на узле 1")
    
    # Добавляем новые сообщения
    new_messages = await create_messages_on_node1(node1, thread_id, count=2)
    
    # Обновляем метаданные треда
    key_manager = KeyManager()
    meta_key = key_manager.get_thread_meta_key(thread_id)
    
    try:
        data = await node1.find_value(meta_key)
        if data:
            thread_dict = deserialize(data)
            thread_meta = ThreadMetadata.from_dict(thread_dict)
            
            # Обновляем счетчик сообщений и время активности
            # Учитываем уже добавленные сообщения
            current_count = thread_meta.message_count if thread_meta.message_count > 0 else 0
            thread_meta.message_count = current_count + len(new_messages)
            thread_meta.last_activity = int(time.time())
            
            # Сохраняем обновленные метаданные
            updated_data = serialize(thread_meta.to_dict())
            success = await node1.store(meta_key, updated_data, ttl=86400)
            
            if success:
                print_success(f"Тред обновлен: добавлено {len(new_messages)} новых сообщений (всего: {thread_meta.message_count})")
                return True
    except Exception as e:
        print_error(f"Ошибка при обновлении: {e}")
    
    return False


async def create_global_index(node1: FullNode):
    """Создание глобального индекса тредов"""
    print_header("Создание глобального индекса тредов на узле 1")
    
    key_manager = KeyManager()
    threads_key = key_manager.get_global_threads_key()
    
    # Список всех тредов
    thread_ids = ["thread_p2p", "thread_ai", "thread_crypto"]
    
    threads_data = serialize(thread_ids)
    success = await node1.store(threads_key, threads_data, ttl=2592000)  # 30 дней
    
    if success:
        print_success(f"Глобальный индекс создан с {len(thread_ids)} тредами")
        
        # Пытаемся получить обратно
        try:
            retrieved_data = await node1.find_value(threads_key)
            if retrieved_data:
                thread_list = deserialize(retrieved_data)
                print_info(f"Проверка: найдено {len(thread_list)} тредов в индексе")
                for tid in thread_list:
                    print_info(f"  - {tid}")
        except Exception as e:
            print_warning(f"Не удалось прочитать индекс обратно: {e}")
        
        return True
    else:
        print_warning("Не удалось создать глобальный индекс (возможно, rate limit)")
        print_info("Это нормально при большом количестве запросов - данные сохранены локально")
        return False


async def check_popularity_metrics(node1: FullNode, node2: FullNode):
    """Проверка метрик популярности на обоих узлах"""
    print_header("Метрики популярности")
    
    for node_name, node in [("Узел 1", node1), ("Узел 2", node2)]:
        print_info(f"\n{node_name}:")
        all_metrics = node.metrics_collector.get_all_metrics()
        
        if all_metrics:
            print_success(f"Найдено метрик: {len(all_metrics)}")
            
            # Ранжируем элементы
            ranked = node.popularity_ranker.rank_items(all_metrics, limit=10)
            
            if ranked:
                print_info("Топ популярных элементов:")
                for i, item in enumerate(ranked[:5], 1):
                    print_info(f"  {i}. Ключ: {item.key.hex()[:16]}... | Рейтинг: {item.score:.2f}")
            else:
                print_warning("Нет ранжированных элементов")
        else:
            print_warning("Нет собранных метрик")


async def check_routing_tables(node1: FullNode, node2: FullNode):
    """Проверка состояния routing tables"""
    print_header("Состояние Routing Tables")
    
    nodes1 = node1.routing_table.get_all_nodes()
    nodes2 = node2.routing_table.get_all_nodes()
    
    print_info(f"Узел 1 - узлов в routing table: {len(nodes1)}")
    for node in nodes1[:3]:  # Показываем первые 3
        print_info(f"  - {node.node_id.id.hex()[:16]}... @ {node.address}:{node.port}")
    
    print_info(f"\nУзел 2 - узлов в routing table: {len(nodes2)}")
    for node in nodes2[:3]:
        print_info(f"  - {node.node_id.id.hex()[:16]}... @ {node.address}:{node.port}")


async def test_replication(node1: FullNode, node2: FullNode, thread_id: str):
    """Тест репликации данных между узлами"""
    print_header(f"Тест репликации треда '{thread_id}'")
    
    key_manager = KeyManager()
    meta_key = key_manager.get_thread_meta_key(thread_id)
    
    # Проверяем наличие на узле 1
    print_info("Проверка на узле 1 (источник)...")
    data1 = await node1.find_value(meta_key)
    if data1:
        print_success("Данные найдены на узле 1")
    else:
        print_error("Данные не найдены на узле 1")
        return
    
    # Проверяем наличие на узле 2 (репликация)
    print_info("Проверка на узле 2 (репликация)...")
    await asyncio.sleep(1)  # Даем время на репликацию
    
    try:
        data2 = await node2.find_value(meta_key)
        if data2:
            print_success("Данные найдены на узле 2 (репликация работает!)")
            
            # Сравниваем данные
            thread1 = ThreadMetadata.from_dict(deserialize(data1))
            thread2 = ThreadMetadata.from_dict(deserialize(data2))
            
            if thread1.id == thread2.id and thread1.title == thread2.title:
                print_success("Данные идентичны на обоих узлах")
            else:
                print_warning("Данные различаются между узлами")
        else:
            print_warning("Данные не найдены на узле 2 (репликация еще не завершена)")
    except Exception as e:
        print_warning(f"Ошибка при поиске на узле 2: {e}")


async def main():
    """Главная функция с полным демо"""
    print_header("Rhizome - Демонстрация работы двух узлов")
    print_info("Этот пример демонстрирует полный функционал P2P сети")
    print_info("Создание узлов, обмен данными, обновления, популярность\n")
    
    try:
        # 1. Настройка узлов
        node1, node2 = await setup_nodes()
        if not node1 or not node2:
            return
        
        # 2. Запуск узлов
        await start_nodes(node1, node2)
        
        # 3. Создание тредов на узле 1
        threads = await create_threads_on_node1(node1)
        await asyncio.sleep(1)
        
        # 4. Создание сообщений в первом треде
        if threads:
            await create_messages_on_node1(node1, threads[0].id, count=3)
            await asyncio.sleep(1)
        
        # 5. Поиск данных на узле 2
        if threads:
            found_thread = await find_data_on_node2(node2, threads[0].id)
            await asyncio.sleep(1)
        
        # 6. Обновление треда на узле 1
        if threads:
            await update_thread_on_node1(node1, threads[0].id)
            await asyncio.sleep(2)  # Даем время на обработку и избегаем rate limit
        
        # 7. Создание глобального индекса (с задержкой для избежания rate limit)
        await asyncio.sleep(1)  # Дополнительная задержка перед глобальным индексом
        await create_global_index(node1)
        await asyncio.sleep(1)
        
        # 8. Проверка метрик популярности
        await check_popularity_metrics(node1, node2)
        await asyncio.sleep(1)
        
        # 9. Проверка routing tables
        await check_routing_tables(node1, node2)
        await asyncio.sleep(1)
        
        # 10. Тест репликации
        if threads:
            await test_replication(node1, node2, threads[0].id)
        
        print_header("Демонстрация завершена успешно!")
        print_info("Узлы продолжают работать. Нажмите Ctrl+C для остановки.")
        print_info("\nДля остановки узлов нажмите Ctrl+C...\n")
        
        # Ждем до прерывания
        while True:
            await asyncio.sleep(1)
    
    except KeyboardInterrupt:
        print_header("Остановка узлов...")
        if node1:
            await node1.stop()
            print_success("Узел 1 остановлен")
        if node2:
            await node2.stop()
            print_success("Узел 2 остановлен")
        print_header("Все узлы остановлены. До свидания!")
    
    except Exception as e:
        print_error(f"Ошибка: {e}")
        import traceback
        traceback.print_exc()
        if node1:
            await node1.stop()
        if node2:
            await node2.stop()


if __name__ == "__main__":
    asyncio.run(main())

