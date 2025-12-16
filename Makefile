.PHONY: help install install-dev test lint format clean run

help:
	@echo "Доступные команды:"
	@echo "  make install      - Установка зависимостей"
	@echo "  make install-dev  - Установка зависимостей для разработки"
	@echo "  make test         - Запуск тестов"
	@echo "  make lint         - Проверка кода линтерами"
	@echo "  make format       - Форматирование кода"
	@echo "  make clean        - Очистка временных файлов"
	@echo "  make run          - Запуск узла"

install:
	pip install -r requirements.txt

install-dev:
	pip install -r requirements.txt
	pip install -e ".[dev]"

test:
	pytest

lint:
	flake8 rhizome tests
	mypy rhizome

format:
	black rhizome tests
	isort rhizome tests

clean:
	find . -type d -name __pycache__ -exec rm -r {} +
	find . -type f -name "*.pyc" -delete
	find . -type f -name "*.pyo" -delete
	find . -type d -name "*.egg-info" -exec rm -r {} +
	rm -rf build dist .pytest_cache .mypy_cache htmlcov .coverage

run:
	python -m rhizome.cli

