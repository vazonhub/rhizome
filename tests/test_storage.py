"""
Тесты для модуля хранения
"""

import pytest
import asyncio
from pathlib import Path
import tempfile
import shutil

from rhizome.config import StorageConfig
from rhizome.storage.storage import Storage


@pytest.fixture
def temp_storage():
    """Создание временного хранилища для тестов"""
    temp_dir = Path(tempfile.mkdtemp())
    config = StorageConfig(data_dir=temp_dir, max_storage_size=100 * 1024 * 1024)
    storage = Storage(config)
    yield storage
    storage.close()
    shutil.rmtree(temp_dir)


@pytest.mark.asyncio
async def test_storage_put_get(temp_storage):
    """Тест сохранения и получения данных"""
    key = b"test_key"
    value = b"test_value"
    ttl = 3600
    
    await temp_storage.put(key, value, ttl)
    retrieved = await temp_storage.get(key)
    
    assert retrieved == value


@pytest.mark.asyncio
async def test_storage_delete(temp_storage):
    """Тест удаления данных"""
    key = b"test_key"
    value = b"test_value"
    
    await temp_storage.put(key, value, 3600)
    await temp_storage.delete(key)
    
    retrieved = await temp_storage.get(key)
    assert retrieved is None


@pytest.mark.asyncio
async def test_storage_extend_ttl(temp_storage):
    """Тест продления TTL"""
    key = b"test_key"
    value = b"test_value"
    
    await temp_storage.put(key, value, 3600)
    result = await temp_storage.extend_ttl(key, extension=0.1)
    
    assert result is True
    
    # Проверяем, что данные все еще доступны
    retrieved = await temp_storage.get(key)
    assert retrieved == value

