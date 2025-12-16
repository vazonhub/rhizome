"""
Модуль конфигурации Rhizome
"""

import os
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

import yaml
from dotenv import load_dotenv

load_dotenv()


@dataclass
class DHTConfig:
    """Конфигурация DHT"""
    k: int = 20  # Размер k-бакета
    alpha: int = 3  # Количество параллельных запросов
    node_id_bits: int = 160  # Размер Node ID в битах
    bucket_count: int = 160  # Количество k-бакетов
    refresh_interval: int = 3600  # Интервал рефрешинга бакетов (секунды)
    ping_timeout: float = 5.0  # Таймаут PING (секунды)
    request_timeout: float = 10.0  # Таймаут запросов (секунды)


@dataclass
class StorageConfig:
    """Конфигурация хранилища"""
    data_dir: Path = Path("data")
    max_storage_size: int = 10 * 1024 * 1024 * 1024  # 10 GB по умолчанию
    default_ttl: int = 86400  # 1 день в секундах
    popular_ttl: int = 2592000  # 30 дней для популярного контента
    active_ttl: int = 604800  # 7 дней для активного контента
    private_ttl: int = 10800  # 3 часа для личных сообщений
    min_guaranteed_ttl: int = 3600  # 1 час минимальный гарантированный срок


@dataclass
class NetworkConfig:
    """Конфигурация сети"""
    listen_host: str = "0.0.0.0"
    listen_port: int = 8468
    bootstrap_nodes: list[str] = None
    max_connections: int = 100
    connection_timeout: float = 30.0


@dataclass
class NodeConfig:
    """Конфигурация узла"""
    node_type: str = "full"  # seed, full, light, mobile
    auto_detect_type: bool = True
    node_id_file: Path = Path("node_id.pem")
    state_file: Path = Path("node_state.json")


@dataclass
class PopularityConfig:
    """Конфигурация механизма популярности"""
    update_interval: int = 3600  # Обновление рейтинга каждый час
    exchange_interval: int = 21600  # Обмен топ-100 каждые 6 часов
    global_update_interval: int = 10800  # Глобальное обновление каждые 3 часа
    popularity_threshold: float = 7.0  # Порог популярности
    active_threshold: float = 5.0  # Порог активности


@dataclass
class SecurityConfig:
    """Конфигурация безопасности"""
    enable_ring_signatures: bool = True
    ring_size: int = 8  # Размер группы для кольцевых подписей
    enable_stealth_addresses: bool = True
    enable_tor: bool = False
    enable_i2p: bool = False
    rate_limit_requests: int = 100  # Запросов в секунду
    rate_limit_window: int = 60  # Окно в секундах


@dataclass
class Config:
    """Главная конфигурация"""
    dht: DHTConfig
    storage: StorageConfig
    network: NetworkConfig
    node: NodeConfig
    popularity: PopularityConfig
    security: SecurityConfig
    log_level: str = "INFO"
    log_file: Optional[Path] = None

    @classmethod
    def from_file(cls, config_path: Optional[Path] = None) -> "Config":
        """Загрузка конфигурации из файла"""
        if config_path is None:
            config_path = Path("config.yaml")
        
        if config_path.exists():
            with open(config_path, "r") as f:
                config_data = yaml.safe_load(f)
        else:
            config_data = {}
        
        return cls(
            dht=DHTConfig(**config_data.get("dht", {})),
            storage=StorageConfig(
                data_dir=Path(config_data.get("storage", {}).get("data_dir", "data")),
                **{k: v for k, v in config_data.get("storage", {}).items() if k != "data_dir"}
            ),
            network=NetworkConfig(
                bootstrap_nodes=config_data.get("network", {}).get("bootstrap_nodes", []),
                **{k: v for k, v in config_data.get("network", {}).items() if k != "bootstrap_nodes"}
            ),
            node=NodeConfig(
                node_id_file=Path(config_data.get("node", {}).get("node_id_file", "node_id.pem")),
                state_file=Path(config_data.get("node", {}).get("state_file", "node_state.json")),
                **{k: v for k, v in config_data.get("node", {}).items() if k not in ["node_id_file", "state_file"]}
            ),
            popularity=PopularityConfig(**config_data.get("popularity", {})),
            security=SecurityConfig(**config_data.get("security", {})),
            log_level=config_data.get("log_level", os.getenv("LOG_LEVEL", "INFO")),
            log_file=Path(config_data["log_file"]) if config_data.get("log_file") else None,
        )

    def to_file(self, config_path: Path) -> None:
        """Сохранение конфигурации в файл"""
        config_data = {
            "dht": {
                "k": self.dht.k,
                "alpha": self.dht.alpha,
                "node_id_bits": self.dht.node_id_bits,
                "bucket_count": self.dht.bucket_count,
                "refresh_interval": self.dht.refresh_interval,
                "ping_timeout": self.dht.ping_timeout,
                "request_timeout": self.dht.request_timeout,
            },
            "storage": {
                "data_dir": str(self.storage.data_dir),
                "max_storage_size": self.storage.max_storage_size,
                "default_ttl": self.storage.default_ttl,
                "popular_ttl": self.storage.popular_ttl,
                "active_ttl": self.storage.active_ttl,
                "private_ttl": self.storage.private_ttl,
                "min_guaranteed_ttl": self.storage.min_guaranteed_ttl,
            },
            "network": {
                "listen_host": self.network.listen_host,
                "listen_port": self.network.listen_port,
                "bootstrap_nodes": self.network.bootstrap_nodes or [],
                "max_connections": self.network.max_connections,
                "connection_timeout": self.network.connection_timeout,
            },
            "node": {
                "node_type": self.node.node_type,
                "auto_detect_type": self.node.auto_detect_type,
                "node_id_file": str(self.node.node_id_file),
                "state_file": str(self.node.state_file),
            },
            "popularity": {
                "update_interval": self.popularity.update_interval,
                "exchange_interval": self.popularity.exchange_interval,
                "global_update_interval": self.popularity.global_update_interval,
                "popularity_threshold": self.popularity.popularity_threshold,
                "active_threshold": self.popularity.active_threshold,
            },
            "security": {
                "enable_ring_signatures": self.security.enable_ring_signatures,
                "ring_size": self.security.ring_size,
                "enable_stealth_addresses": self.security.enable_stealth_addresses,
                "enable_tor": self.security.enable_tor,
                "enable_i2p": self.security.enable_i2p,
                "rate_limit_requests": self.security.rate_limit_requests,
                "rate_limit_window": self.security.rate_limit_window,
            },
            "log_level": self.log_level,
        }
        
        if self.log_file:
            config_data["log_file"] = str(self.log_file)
        
        with open(config_path, "w") as f:
            yaml.dump(config_data, f, default_flow_style=False, sort_keys=False)

