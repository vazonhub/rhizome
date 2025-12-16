"""
Утилиты для сериализации данных
"""

import json
from typing import Any, Union

import msgpack


def serialize(data: Any, format: str = "msgpack") -> bytes:
    """
    Сериализация данных

    Args:
        data: Данные для сериализации
        format: Формат сериализации ("msgpack" или "json")

    Returns:
        Сериализованные данные в виде bytes
    """
    if format == "msgpack":
        return msgpack.packb(data, use_bin_type=True)
    elif format == "json":
        return json.dumps(data, ensure_ascii=False).encode("utf-8")
    else:
        raise ValueError(f"Unsupported format: {format}")


def deserialize(data: bytes, format: str = "msgpack") -> Any:
    """
    Десериализация данных

    Args:
        data: Сериализованные данные
        format: Формат сериализации ("msgpack" или "json")

    Returns:
        Десериализованные данные
    """
    if format == "msgpack":
        return msgpack.unpackb(data, raw=False)
    elif format == "json":
        return json.loads(data.decode("utf-8"))
    else:
        raise ValueError(f"Unsupported format: {format}")
