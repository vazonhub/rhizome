use crate::utils::crypto::hash_key; // Предполагаем, что функция hash_key находится в модуле crypto

/// Построитель ключей для DHT
pub struct DHTKeyBuilder;

#[allow(dead_code)]
impl DHTKeyBuilder {
    /// Ключ для списка всех тредов
    pub fn global_threads() -> [u8; 32] {
        hash_key("global:threads".as_bytes())
    }

    /// Ключ для топ-N популярных тредов
    pub fn global_popular() -> [u8; 32] {
        hash_key("global:popular".as_bytes())
    }

    /// Ключ для последних N сообщений
    pub fn global_recent() -> [u8; 32] {
        hash_key("global:recent".as_bytes())
    }

    /// Ключ для списка активных seed-узлов
    pub fn global_seeds() -> [u8; 32] {
        hash_key("global:seeds".as_bytes())
    }

    /// Ключ для метаданных треда
    pub fn thread_meta(thread_id: &str) -> [u8; 32] {
        hash_key(format!("thread:{}:meta", thread_id).as_bytes())
    }

    /// Ключ для хронологического списка сообщений треда
    pub fn thread_index(thread_id: &str) -> [u8; 32] {
        hash_key(format!("thread:{}:index", thread_id).as_bytes())
    }

    /// Ключ для популярных сообщений в треде
    pub fn thread_popular(thread_id: &str) -> [u8; 32] {
        hash_key(format!("thread:{}:popular", thread_id).as_bytes())
    }

    /// Ключ для статистики треда
    pub fn thread_stats(thread_id: &str) -> [u8; 32] {
        hash_key(format!("thread:{}:stats", thread_id).as_bytes())
    }

    /// Ключ для сообщения
    pub fn message(message_hash: &str) -> [u8; 32] {
        hash_key(format!("msg:{}", message_hash).as_bytes())
    }

    /// Ключ для ссылок на ответы/цитаты сообщения
    pub fn message_refs(message_hash: &str) -> [u8; 32] {
        hash_key(format!("msg:{}:refs", message_hash).as_bytes())
    }

    /// Ключ для голосов/реакций к сообщению
    pub fn message_votes(message_hash: &str) -> [u8; 32] {
        hash_key(format!("msg:{}:votes", message_hash).as_bytes())
    }

    /// Ключ для профиля пользователя
    pub fn user_profile(pubkey: &str) -> [u8; 32] {
        hash_key(format!("user:{}:profile", pubkey).as_bytes())
    }

    /// Ключ для тредов пользователя
    pub fn user_threads(pubkey: &str) -> [u8; 32] {
        hash_key(format!("user:{}:threads", pubkey).as_bytes())
    }

    /// Ключ для репутации пользователя
    pub fn user_reputation(pubkey: &str) -> [u8; 32] {
        hash_key(format!("user:{}:reputation", pubkey).as_bytes())
    }

    /// Парсинг ключа для определения его типа
    pub fn parse_key(_key: &[u8]) -> Option<std::collections::HashMap<String, String>> {
        // В оригинале упрощенная версия возвращает None
        None
    }
}

/// Менеджер для работы с ключами DHT
pub struct KeyManager {
    // В Rust нам не обязательно хранить инстанс билдера,
    // если методы статические, но для сохранения структуры Python-кода:
}

impl KeyManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Получение ключа для метаданных треда
    pub fn get_thread_meta_key(&self, thread_id: &str) -> [u8; 32] {
        DHTKeyBuilder::thread_meta(thread_id)
    }

    /// Получение ключа для сообщения
    pub fn get_message_key(&self, message_hash: &str) -> [u8; 32] {
        DHTKeyBuilder::message(message_hash)
    }

    /// Получение ключа для глобального списка тредов
    pub fn get_global_threads_key(&self) -> [u8; 32] {
        DHTKeyBuilder::global_threads()
    }

    /// Получение ключа для популярных тредов
    pub fn get_global_popular_key(&self) -> [u8; 32] {
        DHTKeyBuilder::global_popular()
    }
}
