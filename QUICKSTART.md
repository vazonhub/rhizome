# Быстрый старт Rhizome

## Установка

1. **Клонируйте репозиторий:**
   ```bash
   git clone <repository-url>
   cd rhizome
   ```

2. **Создайте виртуальное окружение (рекомендуется):**
   ```bash
   python3 -m venv venv
   source venv/bin/activate  # Linux/Mac
   # или
   venv\Scripts\activate  # Windows
   ```

3. **Установите зависимости:**
   ```bash
   pip install -r requirements.txt
   # или
   python3 -m pip install -r requirements.txt
   ```
   
   Если возникают проблемы с правами доступа:
   ```bash
   python3 -m pip install --user -r requirements.txt
   ```

4. **Настройте конфигурацию (опционально):**
   ```bash
   cp config.yaml.example config.yaml
   # Отредактируйте config.yaml под свои нужды
   ```
   
   **Примечание:** Если файл `config.yaml` отсутствует, будет использована конфигурация по умолчанию.

## Запуск узла

### Базовый запуск

```bash
python3 run_node.py
```

### Через CLI с параметрами

```bash
# С указанием типа узла
python3 -m rhizome.cli --node-type full

# С указанием конфигурационного файла
python3 -m rhizome.cli --config /path/to/config.yaml

# Доступные типы узлов: seed, full, light, mobile
```

## Примеры использования

### Запуск примеров

```bash
python3 examples/usage_examples.py
```

### Программное использование

```python
import asyncio
from rhizome.config import Config
from rhizome.node.full_node import FullNode
from rhizome.storage.keys import KeyManager
from rhizome.storage.data_types import ThreadMetadata
from rhizome.utils.serialization import serialize

async def main():
    # Загрузка конфигурации
    config = Config.from_file("config.yaml")
    
    # Создание узла
    node = FullNode(config)
    
    # Запуск узла
    await node.start()
    
    # Использование API
    key_manager = KeyManager()
    
    # Создание треда
    thread = ThreadMetadata(
        id="my_thread",
        title="Мой тред",
        created_at=1234567890,
        creator_pubkey="0xabc123"
    )
    
    key = key_manager.get_thread_meta_key(thread.id)
    data = serialize(thread.to_dict())
    
    # Сохранение
    await node.store(key, data, ttl=86400)
    
    # Поиск
    result = await node.find_value(key)
    
    print("Готово!")

asyncio.run(main())
```

## Типы узлов

- **seed** - Высокая доступность, большой объем диска (>100GB), долгосрочное хранение
- **full** - Основная рабочая нагрузка, средний объем диска (>10GB)
- **light** - Ограниченные ресурсы, малый объем диска (<1GB)
- **mobile** - Максимально легкий клиент, минимальный объем диска (<100MB)

## Устранение неполадок

### Ошибка "ModuleNotFoundError"

Убедитесь, что все зависимости установлены:
```bash
pip install -r requirements.txt
```

### Ошибка "Config file not found"

Создайте файл конфигурации:
```bash
cp config.yaml.example config.yaml
```

### Порт уже занят

Измените порт в `config.yaml`:
```yaml
network:
  listen_port: 8469  # Другой порт
```

## Дополнительная информация

Полная документация находится в файле `ReadMe.md`.

