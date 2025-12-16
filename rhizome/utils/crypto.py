"""
Криптографические утилиты
"""

import hashlib
from pathlib import Path
from typing import Optional, Union

from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa


def generate_node_id() -> bytes:
    """
    Генерация 160-битного Node ID

    Returns:
        20 байт (160 бит) идентификатора узла
    """
    private_key = rsa.generate_private_key(
        public_exponent=65537, key_size=2048, backend=default_backend()
    )
    public_key = private_key.public_key()

    # Хэшируем публичный ключ для получения Node ID
    public_key_bytes = public_key.public_bytes(
        encoding=serialization.Encoding.DER, format=serialization.PublicFormat.SubjectPublicKeyInfo
    )

    # Используем SHA-1 для получения 160-битного ID
    node_id = hashlib.sha1(public_key_bytes).digest()
    return node_id


def compute_distance(node_id1: bytes, node_id2: bytes) -> bytes:
    """
    Вычисление XOR-расстояния между двумя Node ID

    Args:
        node_id1: Первый Node ID (20 байт)
        node_id2: Второй Node ID (20 байт)

    Returns:
        XOR-расстояние (20 байт)
    """
    if len(node_id1) != len(node_id2):
        raise ValueError("Node IDs must have the same length")

    return bytes(a ^ b for a, b in zip(node_id1, node_id2))


def hash_key(key: Union[str, bytes]) -> bytes:
    """
    Хэширование ключа для DHT (SHA-256)

    Args:
        key: Ключ для хэширования

    Returns:
        32 байта хэша (SHA-256)
    """
    if isinstance(key, str):
        key = key.encode("utf-8")

    return hashlib.sha256(key).digest()


def generate_keypair():
    """
    Генерация пары ключей для криптографии

    Returns:
        Tuple (private_key, public_key)
    """
    private_key = rsa.generate_private_key(
        public_exponent=65537, key_size=2048, backend=default_backend()
    )
    public_key = private_key.public_key()
    return private_key, public_key


def save_node_id(node_id: bytes, file_path: Path) -> None:
    """
    Сохранение Node ID в файл

    Args:
        node_id: Node ID для сохранения (20 байт)
        file_path: Путь к файлу
    """
    file_path.parent.mkdir(parents=True, exist_ok=True)
    with open(file_path, "wb") as f:
        f.write(node_id)


def load_node_id(file_path: Path) -> Optional[bytes]:
    """
    Загрузка Node ID из файла

    Args:
        file_path: Путь к файлу

    Returns:
        Node ID или None если файл не существует
    """
    if not file_path.exists():
        return None

    with open(file_path, "rb") as f:
        node_id = f.read()

    if len(node_id) != 20:
        raise ValueError(f"Invalid node ID length: {len(node_id)}, expected 20")

    return node_id
