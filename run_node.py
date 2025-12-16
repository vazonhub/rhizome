#!/usr/bin/env python3
"""
Простой скрипт для запуска узла Rhizome
"""

import asyncio
import sys
from pathlib import Path

from rhizome.config import Config
from rhizome.node.full_node import FullNode


async def main():
    """Главная функция"""
    # Загрузка конфигурации
    config_path = Path("config.yaml")
    if not config_path.exists():
        print(f"Error: Config file not found: {config_path}")
        print("Please create config.yaml or copy from config.yaml.example")
        sys.exit(1)
    
    config = Config.from_file(config_path)
    
    # Создание узла
    node = FullNode(config)
    
    try:
        print(f"Starting Rhizome node (type: {node.node_type})...")
        print(f"Node ID: {node.node_id.id.hex()[:32]}...")
        print(f"Listening on {config.network.listen_host}:{config.network.listen_port}")
        print("Press Ctrl+C to stop")
        
        await node.start()
        
        # Бесконечный цикл
        while node.is_running:
            await asyncio.sleep(1)
            
    except KeyboardInterrupt:
        print("\nShutting down...")
        await node.stop()
        print("Node stopped")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nInterrupted by user")
        sys.exit(0)

