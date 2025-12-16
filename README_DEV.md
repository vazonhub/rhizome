# Руководство разработчика Rhizome

## Структура проекта

```
rhizome/
├── rhizome/                 # Основной пакет
│   ├── __init__.py
│   ├── config.py           # Конфигурация
│   ├── exceptions.py       # Исключения
│   ├── cli.py              # Командная строка
│   ├── dht/                # Модуль Kademlia DHT
│   │   ├── __init__.py
│   │   ├── node.py         # Node ID и представление узла
│   │   ├── routing_table.py # Таблица маршрутизации (k-бакеты)
│   │   └── protocol.py     # Протокол DHT операций
│   ├── storage/            # Модуль хранения данных
│   │   ├── __init__.py
│   │   ├── storage.py      # Хранилище на базе LMDB
│   │   └── data_types.py   # Типы данных (Thread, Message)
│   ├── node/               # Модуль узлов
│   │   ├── __init__.py
│   │   ├── base_node.py    # Базовый класс узла
│   │   ├── seed_node.py    # Seed-узел
│   │   ├── full_node.py    # Full-узел
│   │   ├── light_node.py   # Light-узел
│   │   └── mobile_node.py  # Mobile-узел
│   └── utils/              # Утилиты
│       ├── __init__.py
│       ├── crypto.py       # Криптографические утилиты
│       └── serialization.py # Сериализация данных
├── tests/                  # Тесты
│   ├── __init__.py
│   └── test_crypto.py
├── requirements.txt        # Зависимости
├── pyproject.toml          # Конфигурация проекта
├── config.yaml.example     # Пример конфигурации
├── Makefile               # Команды для разработки
├── PLAN.md                # План работ
└── ReadMe.md              # Техническая документация
```

## Установка и настройка

### 1. Создание виртуального окружения

```bash
python3 -m venv venv
source venv/bin/activate  # Linux/Mac
# или
venv\Scripts\activate  # Windows
```

### 2. Установка зависимостей

```bash
make install-dev
# или
pip install -r requirements.txt
pip install -e ".[dev]"
```

### 3. Настройка конфигурации

```bash
cp config.yaml.example config.yaml
# Отредактируйте config.yaml под свои нужды
```

## Разработка

### Запуск тестов

```bash
make test
# или
pytest
```

### Проверка кода

```bash
make lint
# или
flake8 rhizome tests
mypy rhizome
```

### Форматирование кода

```bash
make format
# или
black rhizome tests
isort rhizome tests
```

### Запуск узла

```bash
make run
# или
python -m rhizome.cli
```

## Архитектура модулей

### DHT (Distributed Hash Table)

- **node.py**: Определяет `NodeID` (160-битный идентификатор) и `Node` (представление узла в сети)
- **routing_table.py**: Реализует k-бакеты и таблицу маршрутизации Kademlia
- **protocol.py**: Реализует основные операции DHT (PING, FIND_NODE, FIND_VALUE, STORE)

### Storage (Хранилище)

- **storage.py**: Локальное хранилище на базе LMDB с поддержкой TTL
- **data_types.py**: Типы данных для тредов и сообщений

### Node (Узлы)

- **base_node.py**: Базовый класс для всех типов узлов
- **seed_node.py**, **full_node.py**, **light_node.py**, **mobile_node.py**: Конкретные реализации типов узлов

### Utils (Утилиты)

- **crypto.py**: Криптографические функции (генерация Node ID, XOR-расстояние, хэширование)
- **serialization.py**: Сериализация/десериализация данных (msgpack, JSON)

## Следующие шаги разработки

См. `PLAN.md` для детального плана работ. Приоритетные задачи:

1. Реализация сетевого протокола (UDP/TCP)
2. Реализация bootstrap процесса
3. Реализация механизма популярности
4. Реализация репликации
5. Реализация анонимности (кольцевые подписи, stealth-адреса)

## Стиль кода

- Используйте `black` для форматирования
- Следуйте PEP 8
- Добавляйте docstrings для всех публичных функций и классов
- Покрывайте код тестами

## Вклад в проект

1. Создайте ветку для новой функции
2. Реализуйте изменения
3. Добавьте тесты
4. Убедитесь, что все тесты проходят
5. Создайте pull request

