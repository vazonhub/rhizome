"""
Тесты для работы с ключами DHT
"""

import pytest

from rhizome.storage.keys import DHTKeyBuilder, KeyManager


def test_global_keys():
    """Тест глобальных ключей"""
    builder = DHTKeyBuilder()

    # Проверяем, что ключи генерируются
    threads_key = builder.global_threads()
    assert len(threads_key) == 32  # SHA-256

    popular_key = builder.global_popular()
    assert len(popular_key) == 32

    recent_key = builder.global_recent()
    assert len(recent_key) == 32

    seeds_key = builder.global_seeds()
    assert len(seeds_key) == 32

    # Проверяем детерминированность
    assert builder.global_threads() == builder.global_threads()


def test_thread_keys():
    """Тест ключей тредов"""
    builder = DHTKeyBuilder()
    thread_id = "test_thread_123"

    meta_key = builder.thread_meta(thread_id)
    assert len(meta_key) == 32

    index_key = builder.thread_index(thread_id)
    assert len(index_key) == 32

    popular_key = builder.thread_popular(thread_id)
    assert len(popular_key) == 32

    stats_key = builder.thread_stats(thread_id)
    assert len(stats_key) == 32

    # Проверяем, что разные типы дают разные ключи
    assert meta_key != index_key
    assert index_key != popular_key


def test_message_keys():
    """Тест ключей сообщений"""
    builder = DHTKeyBuilder()
    message_hash = "abc123def456"

    msg_key = builder.message(message_hash)
    assert len(msg_key) == 32

    refs_key = builder.message_refs(message_hash)
    assert len(refs_key) == 32

    votes_key = builder.message_votes(message_hash)
    assert len(votes_key) == 32

    assert msg_key != refs_key
    assert refs_key != votes_key


def test_user_keys():
    """Тест ключей пользователей"""
    builder = DHTKeyBuilder()
    pubkey = "0x1234567890abcdef"

    profile_key = builder.user_profile(pubkey)
    assert len(profile_key) == 32

    threads_key = builder.user_threads(pubkey)
    assert len(threads_key) == 32

    reputation_key = builder.user_reputation(pubkey)
    assert len(reputation_key) == 32

    assert profile_key != threads_key
    assert threads_key != reputation_key


def test_key_manager():
    """Тест KeyManager"""
    manager = KeyManager()

    thread_id = "test_thread"
    meta_key = manager.get_thread_meta_key(thread_id)
    assert len(meta_key) == 32

    message_hash = "abc123"
    msg_key = manager.get_message_key(message_hash)
    assert len(msg_key) == 32

    global_threads_key = manager.get_global_threads_key()
    assert len(global_threads_key) == 32

    global_popular_key = manager.get_global_popular_key()
    assert len(global_popular_key) == 32
