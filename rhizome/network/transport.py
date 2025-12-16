"""
UDP транспорт для сетевого протокола
"""

import asyncio
import socket
from dataclasses import dataclass
from typing import Awaitable, Callable, Optional, Tuple

from rhizome.exceptions import NetworkError
from rhizome.logger import get_logger


@dataclass
class Message:
    """Сетевое сообщение"""

    data: bytes
    address: Tuple[str, int]
    timestamp: float


class UDPTransport:
    """UDP транспорт для обмена сообщениями"""

    def __init__(self, host: str = "0.0.0.0", port: int = 8468):
        self.host = host
        self.port = port
        self.socket: Optional[asyncio.DatagramTransport] = None
        self.protocol: Optional[asyncio.DatagramProtocol] = None
        self.message_handler: Optional[Callable[[Message], Awaitable[None]]] = None
        self.logger = get_logger("network.transport")
        self.is_running = False

    async def start(self, message_handler: Callable[[Message], Awaitable[None]]):
        """
        Запуск UDP транспорта

        Args:
            message_handler: Обработчик входящих сообщений
        """
        if self.is_running:
            return

        self.message_handler = message_handler

        loop = asyncio.get_event_loop()

        # Создание UDP сокета
        self.socket, self.protocol = await loop.create_datagram_endpoint(
            lambda: _UDPProtocol(self._handle_message), local_addr=(self.host, self.port)
        )

        self.is_running = True
        self.logger.info("UDP transport started", host=self.host, port=self.port)

    async def stop(self):
        """Остановка UDP транспорта"""
        if not self.is_running:
            return

        if self.socket:
            self.socket.close()

        self.is_running = False
        self.logger.info("UDP transport stopped")

    def _handle_message(self, data: bytes, addr: Tuple[str, int]):
        """Обработка входящего сообщения"""
        if self.message_handler:
            message = Message(data=data, address=addr, timestamp=asyncio.get_event_loop().time())
            # Запускаем обработчик в event loop
            try:
                loop = asyncio.get_event_loop()
                if loop.is_running():
                    loop.create_task(self.message_handler(message))
                else:
                    # Если loop не запущен, запускаем синхронно
                    asyncio.run(self.message_handler(message))
            except RuntimeError:
                # Если нет event loop, создаем новый
                asyncio.run(self.message_handler(message))
            except Exception as e:
                self.logger.error("Error scheduling message handler", error=str(e), exc_info=True)

    async def send(self, data: bytes, address: Tuple[str, int]) -> bool:
        """
        Отправка сообщения

        Args:
            data: Данные для отправки
            address: Адрес получателя (host, port)

        Returns:
            True если отправлено успешно
        """
        if not self.is_running or not self.socket:
            raise NetworkError("Transport is not running")

        try:
            self.socket.sendto(data, address)
            return True
        except Exception as e:
            self.logger.error("Error sending message", error=str(e), address=address)
            return False

    def get_address(self) -> Tuple[str, int]:
        """Получение адреса транспорта"""
        if self.socket:
            return self.socket.get_extra_info("sockname")
        return (self.host, self.port)


class _UDPProtocol(asyncio.DatagramProtocol):
    """Внутренний протокол для UDP"""

    def __init__(self, message_handler: Callable[[bytes, Tuple[str, int]], None]):
        self.message_handler = message_handler
        self.transport: Optional[asyncio.DatagramTransport] = None

    def connection_made(self, transport: asyncio.DatagramTransport):
        """Вызывается при создании соединения"""
        self.transport = transport

    def datagram_received(self, data: bytes, addr: Tuple[str, int]):
        """Вызывается при получении датаграммы"""
        if self.message_handler:
            self.message_handler(data, addr)

    def error_received(self, exc: Exception):
        """Вызывается при ошибке"""
        # Логируем ошибку, но не прерываем работу
        pass

    def connection_lost(self, exc: Optional[Exception]):
        """Вызывается при потере соединения"""
        pass
