"""
Модуль сетевого протокола
"""

from .transport import UDPTransport
from .protocol import NetworkProtocol

__all__ = [
    "UDPTransport",
    "NetworkProtocol",
]

