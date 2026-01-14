use crate::utils::time::get_now_i64;
use serde::{Deserialize, Serialize};
use serde_json::{self, Map, Value};

/// This structure describe the fields of threads
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreadMetadata {
    /// Uniq theme ID
    pub id: String,
    /// Title of the theme which describe main goal
    pub title: String,
    /// When does this thread created
    pub created_at: i64,
    /// Public author of the topic key
    pub creator_pubkey: String,
    /// Category for the thread
    ///
    /// Optional field. You can skip it when create thread
    pub category: Option<String>,
    #[serde(default)]
    /// List of thread Tags
    ///
    /// This list is default empty
    pub tags: Vec<String>,
    #[serde(default)]
    /// How much messages in this thread
    pub message_count: i32,
    /// Time of the last message
    pub last_activity: i64,
    #[serde(default)]
    /// Popularity of this thread
    ///
    /// Work by Seed-nodes
    pub popularity_score: f64,
    #[serde(default = "default_encryption")]
    /// Type of encryption
    pub encryption_type: String,
    /// Who can access
    ///
    /// In JSON format
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
            last_activity: created_at,
            popularity_score: 0.0,
            encryption_type: default_encryption(),
            access_control: None,
        }
    }

    /// Return Thread metadata in JSON format
    pub fn to_dict(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// Return Thread metadata from JSON in Rust object
    pub fn from_dict(data: Value) -> Result<Self, serde_json::Error> {
        let mut meta: Self = serde_json::from_value(data)?;
        if meta.last_activity == 0 {
            meta.last_activity = meta.created_at;
        }
        Ok(meta)
    }
}

/// Describe Message in Thread
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    /// Uniq message id
    pub id: String,
    /// For which thread this message
    pub thread_id: String,
    /// Link to the parent message
    ///
    /// Can be empty for main message and for reply field value is id of parent message
    pub parent_id: Option<String>,
    #[serde(default)]
    /// Message text in `content_type`
    pub content: String,
    /// Cryptography author subscription
    pub author_signature: Option<String>,
    /// Time of message sending
    pub timestamp: i64,
    #[serde(default = "default_content_type")]
    /// Type of content
    ///
    /// _(default: Markdown)_
    pub content_type: String,
    #[serde(default)]
    /// Links to the files or external links
    pub attachments: Vec<String>,
    #[serde(default = "default_empty_map")]
    /// Other data in JSON
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
        let now = get_now_i64();
        Self {
            id,
            thread_id,
            parent_id: None,
            content: String::new(),
            author_signature: None,
            timestamp: now,
            content_type: default_content_type(),
            attachments: Vec::new(),
            metadata: default_empty_map(),
        }
    }

    /// Return Message in JSON format
    pub fn to_dict(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// Return Message from JSON in Rust object
    pub fn from_dict(data: Value) -> Result<Self, serde_json::Error> {
        let mut msg: Self = serde_json::from_value(data)?;
        if msg.timestamp == 0 {
            msg.timestamp = get_now_i64();
        }
        Ok(msg)
    }
}

/// Container for thread data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thread {
    /// Thread metadata
    pub metadata: ThreadMetadata,
    #[serde(default)]
    /// All messages
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

    /// Add new message in the thread
    pub fn add_message(&mut self, message: Message) {
        self.metadata.last_activity = message.timestamp;
        self.messages.push(message);
        self.metadata.message_count = self.messages.len() as i32;
    }

    /// Return Thread in JSON format
    pub fn to_dict(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// Return Thread from JSON in Rust object
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
