"""
Исключения для Rhizome
"""


class RhizomeError(Exception):
    """Базовое исключение для Rhizome"""
    pass


class DHTError(RhizomeError):
    """Ошибка DHT операций"""
    pass


class StorageError(RhizomeError):
    """Ошибка хранилища"""
    pass


class NetworkError(RhizomeError):
    """Ошибка сетевых операций"""
    pass


class NodeNotFoundError(DHTError):
    """Узел не найден"""
    pass


class ValueNotFoundError(DHTError):
    """Значение не найдено в DHT"""
    pass


class StorageFullError(StorageError):
    """Хранилище переполнено"""
    pass


class InvalidNodeTypeError(RhizomeError):
    """Неверный тип узла"""
    pass


class BootstrapError(NetworkError):
    """Ошибка bootstrap процесса"""
    pass


class ReplicationError(StorageError):
    """Ошибка репликации"""
    pass


class SecurityError(RhizomeError):
    """Ошибка безопасности"""
    pass


class InvalidSignatureError(SecurityError):
    """Неверная подпись"""
    pass


class RateLimitError(NetworkError):
    """Превышен лимит запросов"""
    pass

