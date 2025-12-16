# Rhizome API - Руководство по использованию

## Введение

Rhizome предоставляет высокоуровневый API клиент (`RhizomeClient`) для удобной работы с P2P сетью. Этот клиент инкапсулирует все детали протокола и предоставляет простой интерфейс для работы с узлами, тредами и сообщениями.

## Быстрый старт

```python
import asyncio
from rhizome import RhizomeClient

async def main():
    # Создание и запуск клиента
    async with RhizomeClient(config_path="config.yaml") as client:
        # Создание треда
        thread = await client.create_thread(
            thread_id="my_thread",
            title="Мой первый тред",
            category="технологии",
            tags=["p2p", "python"]
        )
        
        # Добавление сообщения
        message = await client.add_message(
            thread_id="my_thread",
            content="Привет, мир!"
        )
        
        # Поиск треда
        found = await client.find_thread("my_thread")

asyncio.run(main())
```

## Основные методы

### Управление узлом

#### `start() -> None`
Запускает узел и подключается к сети.

```python
client = RhizomeClient()
await client.start()
```

#### `stop() -> None`
Останавливает узел.

```python
await client.stop()
```

#### `get_node_info() -> Dict[str, Any]`
Возвращает информацию об узле (ID, тип, адрес, размер routing table).

```python
info = client.get_node_info()
print(f"Node ID: {info['node_id']}")
print(f"Type: {info['node_type']}")
```

### Работа с тредами

#### `create_thread(thread_id, title, ...) -> ThreadMetadata`
Создает новый тред.

```python
thread = await client.create_thread(
    thread_id="tech_thread_1",
    title="Обсуждение технологий",
    category="технологии",
    tags=["python", "async"],
    creator_pubkey="0xabc123...",
    ttl=86400  # 1 день
)
```

**Параметры:**
- `thread_id` (str): Уникальный идентификатор треда (обязательно)
- `title` (str): Заголовок треда (обязательно)
- `category` (str, optional): Категория треда
- `tags` (List[str], optional): Список тегов
- `creator_pubkey` (str, optional): Публичный ключ создателя
- `ttl` (int, optional): Время жизни в секундах (по умолчанию 86400)

#### `find_thread(thread_id) -> Optional[ThreadMetadata]`
Находит тред по ID.

```python
thread = await client.find_thread("tech_thread_1")
if thread:
    print(f"Найден: {thread.title}")
```

#### `update_thread(thread_id, **updates) -> Optional[ThreadMetadata]`
Обновляет метаданные треда.

```python
updated = await client.update_thread(
    "tech_thread_1",
    message_count=10,
    popularity_score=5.5,
    last_activity=int(time.time())
)
```

#### `search_threads(query, category, tags) -> List[ThreadMetadata]`
Ищет треды по различным критериям.

```python
# Поиск по категории
tech_threads = await client.search_threads(category="технологии")

# Поиск по тегам
python_threads = await client.search_threads(tags=["python"])

# Комбинированный поиск
results = await client.search_threads(category="технологии", tags=["async"])
```

### Работа с сообщениями

#### `add_message(thread_id, content, ...) -> Message`
Добавляет сообщение в тред.

```python
message = await client.add_message(
    thread_id="tech_thread_1",
    content="Это текст сообщения",
    author_signature="user_123",
    parent_id=None,  # Для ответа на сообщение
    content_type="text/markdown",
    ttl=86400
)
```

**Параметры:**
- `thread_id` (str): ID треда (обязательно)
- `content` (str): Содержимое сообщения (обязательно)
- `author_signature` (str, optional): Подпись автора
- `parent_id` (str, optional): ID родительского сообщения для ответа
- `content_type` (str, optional): Тип контента (по умолчанию "text/markdown")
- `ttl` (int, optional): Время жизни в секундах (по умолчанию 86400)

#### `find_message(message_id) -> Optional[Message]`
Находит сообщение по ID.

```python
message = await client.find_message("msg_123456")
if message:
    print(f"Сообщение: {message.content}")
```

### Глобальные операции

#### `get_global_threads() -> List[str]`
Получает список всех тредов из глобального индекса.

```python
thread_ids = await client.get_global_threads()
print(f"Всего тредов: {len(thread_ids)}")
```

#### `update_global_threads(thread_ids, ttl) -> bool`
Обновляет глобальный индекс тредов.

```python
await client.update_global_threads(
    ["thread_1", "thread_2", "thread_3"],
    ttl=2592000  # 30 дней
)
```

#### `get_popular_threads(limit) -> List[Dict[str, Any]]`
Получает список популярных тредов.

```python
popular = await client.get_popular_threads(limit=10)
for item in popular:
    print(f"Ключ: {item['key']}, Рейтинг: {item['score']}")
```

## Использование с Context Manager

API поддерживает async context manager для автоматического управления жизненным циклом узла:

```python
async with RhizomeClient(config_path="config.yaml") as client:
    # Узел автоматически запускается
    thread = await client.create_thread(...)
    # Узел автоматически останавливается при выходе
```

## Обработка ошибок

Все методы API могут выбрасывать исключения:

```python
from rhizome.api import RhizomeClient
from rhizome.exceptions import RhizomeError

try:
    client = RhizomeClient()
    await client.start()
    
    # Попытка создать тред без запущенного узла
    thread = await client.create_thread(...)
except RuntimeError as e:
    print(f"Ошибка: {e}")
except Exception as e:
    print(f"Неожиданная ошибка: {e}")
finally:
    await client.stop()
```

## Примеры использования

### Пример 1: Создание форума

```python
async def create_forum():
    async with RhizomeClient() as client:
        # Создаем категории
        categories = ["технологии", "наука", "искусство"]
        
        for category in categories:
            thread = await client.create_thread(
                thread_id=f"category_{category}",
                title=f"Категория: {category}",
                category=category
            )
            print(f"Создана категория: {thread.title}")
```

### Пример 2: Добавление комментариев

```python
async def add_comments():
    async with RhizomeClient() as client:
        thread_id = "discussion_1"
        
        # Первое сообщение
        root_msg = await client.add_message(
            thread_id=thread_id,
            content="Начало обсуждения"
        )
        
        # Ответы
        for i in range(5):
            await client.add_message(
                thread_id=thread_id,
                content=f"Ответ #{i+1}",
                parent_id=root_msg.id
            )
```

### Пример 3: Поиск и фильтрация

```python
async def search_example():
    async with RhizomeClient() as client:
        # Создаем тестовые треды
        await client.create_thread("t1", "Python", category="tech", tags=["python"])
        await client.create_thread("t2", "Rust", category="tech", tags=["rust"])
        await client.create_thread("t3", "Биология", category="science", tags=["biology"])
        
        # Обновляем глобальный индекс
        await client.update_global_threads(["t1", "t2", "t3"])
        
        # Поиск по категории
        tech_threads = await client.search_threads(category="tech")
        print(f"Найдено технических тредов: {len(tech_threads)}")
        
        # Поиск по тегам
        python_threads = await client.search_threads(tags=["python"])
        print(f"Найдено тредов про Python: {len(python_threads)}")
```

## Лучшие практики

1. **Всегда используйте async context manager** для управления жизненным циклом узла:
   ```python
   async with RhizomeClient() as client:
       # работа с API
   ```

2. **Проверяйте результат операций**:
   ```python
   thread = await client.find_thread("thread_id")
   if thread:
       # тред найден
   else:
       # тред не найден
   ```

3. **Обрабатывайте исключения**:
   ```python
   try:
       await client.create_thread(...)
   except RuntimeError as e:
       # обработка ошибок
   ```

4. **Используйте правильные TTL**:
   - Короткие данные: 3600 (1 час)
   - Обычные данные: 86400 (1 день)
   - Долгосрочные данные: 2592000 (30 дней)

5. **Обновляйте глобальный индекс** после создания новых тредов:
   ```python
   thread = await client.create_thread(...)
   await client.update_global_threads([thread.id])
   ```

## Расширенные возможности

API клиент инкапсулирует все возможности протокола Rhizome. Для доступа к низкоуровневым функциям можно использовать:

```python
client = RhizomeClient()
await client.start()

# Прямой доступ к узлу
node = client.node

# Доступ к DHT протоколу
dht = node.dht_protocol

# Доступ к метрикам популярности
metrics = node.metrics_collector.get_all_metrics()
```

## Дополнительные ресурсы

- [Примеры использования](../examples/api_usage.py)
- [Техническая документация](ReadMe.md)
- [Руководство разработчика](README_DEV.md)

