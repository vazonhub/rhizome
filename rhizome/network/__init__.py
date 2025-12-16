"""
Модуль сетевого протокола
"""

from .protocol import NetworkProtocol
from .transport import UDPTransport

__all__ = [
    "UDPTransport",
    "NetworkProtocol",
]
