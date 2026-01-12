use serde::{Deserialize, Serialize};
use serde_json::{self, Map, Value};
use std::time::{SystemTime, UNIX_EPOCH};

/// Получение текущего Unix-таймстемпа (аналог time.time())
fn current_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreadMetadata {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub creator_pubkey: String,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub message_count: i32,
    pub last_activity: i64,
    #[serde(default)]
    pub popularity_score: f64,
    #[serde(default = "default_encryption")]
    pub encryption_type: String,
    pub access_control: Option<Value>,
}

fn default_encryption() -> String {
    "public".to_string()
}

impl ThreadMetadata {
    /// Создание нового объекта (с логикой __post_init__)
    pub fn new(id: String, title: String, created_at: i64, creator_pubkey: String) -> Self {
        Self {
            id,
            title,
            created_at,
            creator_pubkey,
            category: None,
            tags: Vec::new(),
            message_count: 0,
            last_activity: created_at, // Логика __post_init__
            popularity_score: 0.0,
            encryption_type: default_encryption(),
            access_control: None,
        }
    }

    pub fn to_dict(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub fn from_dict(data: Value) -> Result<Self, serde_json::Error> {
        let mut meta: Self = serde_json::from_value(data)?;
        // Эмуляция __post_init__ при загрузке данных
        if meta.last_activity == 0 {
            meta.last_activity = meta.created_at;
        }
        Ok(meta)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub thread_id: String,
    pub parent_id: Option<String>,
    #[serde(default)]
    pub content: String,
    pub author_signature: Option<String>,
    pub timestamp: i64,
    #[serde(default = "default_content_type")]
    pub content_type: String,
    #[serde(default)]
    pub attachments: Vec<String>,
    #[serde(default = "default_empty_map")]
    pub metadata: Value,
}

fn default_content_type() -> String {
    "text/markdown".to_string()
}
fn default_empty_map() -> Value {
    Value::Object(Map::new())
}

impl Message {
    pub fn new(id: String, thread_id: String) -> Self {
        let now = current_time();
        Self {
            id,
            thread_id,
            parent_id: None,
            content: String::new(),
            author_signature: None,
            timestamp: now, // Логика __post_init__
            content_type: default_content_type(),
            attachments: Vec::new(),
            metadata: default_empty_map(),
        }
    }

    pub fn to_dict(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub fn from_dict(data: Value) -> Result<Self, serde_json::Error> {
        let mut msg: Self = serde_json::from_value(data)?;
        if msg.timestamp == 0 {
            msg.timestamp = current_time();
        }
        Ok(msg)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thread {
    pub metadata: ThreadMetadata,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[allow(dead_code)]
impl Thread {
    pub fn new(metadata: ThreadMetadata) -> Self {
        Self {
            metadata,
            messages: Vec::new(),
        }
    }

    /// Добавление сообщения в тред
    pub fn add_message(&mut self, message: Message) {
        self.metadata.last_activity = message.timestamp;
        self.messages.push(message);
        self.metadata.message_count = self.messages.len() as i32;
    }

    pub fn to_dict(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub fn from_dict(data: Value) -> Result<Self, serde_json::Error> {
        let metadata_val = data.get("metadata").cloned().unwrap_or(Value::Null);
        let metadata = ThreadMetadata::from_dict(metadata_val)?;

        let messages_val = data.get("messages").and_then(|v| v.as_array());
        let mut messages = Vec::new();

        if let Some(msgs_array) = messages_val {
            for msg_val in msgs_array {
                messages.push(Message::from_dict(msg_val.clone())?);
            }
        }

        Ok(Self { metadata, messages })
    }
}
