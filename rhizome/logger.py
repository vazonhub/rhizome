"""
Система логирования для Rhizome
"""

import logging
import sys
from pathlib import Path
from typing import Optional

import structlog


def setup_logging(
    log_level: str = "INFO",
    log_file: Optional[Path] = None,
    node_id: Optional[str] = None
) -> structlog.BoundLogger:
    """
    Настройка системы логирования
    
    Args:
        log_level: Уровень логирования (DEBUG, INFO, WARNING, ERROR, CRITICAL)
        log_file: Путь к файлу логов (опционально)
        node_id: ID узла для добавления в логи
    
    Returns:
        Настроенный логгер
    """
    # Настройка structlog
    structlog.configure(
        processors=[
            structlog.contextvars.merge_contextvars,
            structlog.processors.add_log_level,
            structlog.processors.StackInfoRenderer(),
            structlog.processors.format_exc_info,
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.processors.JSONRenderer() if log_file else structlog.dev.ConsoleRenderer(),
        ],
        wrapper_class=structlog.make_filtering_bound_logger(
            logging.getLevelName(log_level.upper())
        ),
        context_class=dict,
        logger_factory=structlog.PrintLoggerFactory(),
        cache_logger_on_first_use=True,
    )
    
    # Настройка стандартного logging
    logging.basicConfig(
        format="%(message)s",
        stream=sys.stdout if not log_file else None,
        level=getattr(logging, log_level.upper()),
        handlers=[
            logging.FileHandler(log_file) if log_file else logging.StreamHandler(sys.stdout)
        ] if log_file else None,
    )
    
    # Создание логгера с контекстом узла
    logger = structlog.get_logger()
    if node_id:
        logger = logger.bind(node_id=node_id[:16])  # Первые 16 символов для краткости
    
    return logger


def get_logger(name: Optional[str] = None) -> structlog.BoundLogger:
    """
    Получение логгера для модуля
    
    Args:
        name: Имя модуля (опционально)
    
    Returns:
        Логгер
    """
    logger = structlog.get_logger()
    if name:
        logger = logger.bind(module=name)
    return logger

