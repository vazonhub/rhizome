"""
Командная строка для Rhizome
"""

import asyncio
import sys
from pathlib import Path

from rhizome.config import Config
from rhizome.node.seed_node import SeedNode
from rhizome.node.full_node import FullNode
from rhizome.node.light_node import LightNode
from rhizome.node.mobile_node import MobileNode


def main():
    """Главная функция CLI"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Rhizome P2P Network Node")
    parser.add_argument(
        "--config",
        type=Path,
        default=Path("config.yaml"),
        help="Path to configuration file"
    )
    parser.add_argument(
        "--node-type",
        choices=["seed", "full", "light", "mobile"],
        help="Override node type from config"
    )
    
    args = parser.parse_args()
    
    # Загрузка конфигурации
    config = Config.from_file(args.config)
    
    # Переопределение типа узла если указано
    if args.node_type:
        config.node.node_type = args.node_type
    
    # Создание узла в зависимости от типа
    node_classes = {
        "seed": SeedNode,
        "full": FullNode,
        "light": LightNode,
        "mobile": MobileNode,
    }
    
    node_class = node_classes[config.node.node_type]
    node = node_class(config)
    
    # Запуск узла
    try:
        asyncio.run(run_node(node))
    except KeyboardInterrupt:
        print("\nShutting down...")
        asyncio.run(node.stop())


async def run_node(node):
    """Запуск узла"""
    await node.start()
    
    try:
        # Бесконечный цикл
        while node.is_running:
            await asyncio.sleep(1)
    except KeyboardInterrupt:
        await node.stop()


if __name__ == "__main__":
    main()

