use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::StorageConfig;
use crate::exceptions::StorageError;
use crate::utils::serialization::{deserialize, serialize};
use heed::types::Bytes;
use heed::{Database, Env, EnvOpenOptions};
use serde::{Deserialize, Serialize};
use tokio::task;

#[derive(Serialize, Deserialize, Debug)]
struct MetaData {
    pub expires_at: f64,
    pub size: usize,
}

pub struct Storage {
    #[allow(dead_code)]
    config: StorageConfig,
    env: Env,
    db: Database<Bytes, Bytes>,
    meta_db: Database<Bytes, Bytes>,
}

impl Storage {
    pub fn new(config: StorageConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir = PathBuf::from(&config.data_dir);
        fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("data.lmdb");
        // Создаем папку для LMDB, если её нет (LMDB ожидает папку или файл)
        if !db_path.exists() {
            fs::create_dir_all(&db_path)?;
        }

        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(config.max_storage_size as usize)
                .max_dbs(10)
                .open(db_path)?
        };

        let mut wtxn = env.write_txn()?;

        // Открываем или создаем базы данных
        // В heed базы данных типизированы, используем ByteSlice для сырых данных (bytes)
        let db = env.create_database(&mut wtxn, Some("main"))?;
        let meta_db = env.create_database(&mut wtxn, Some("meta"))?;

        wtxn.commit()?;

        Ok(Self {
            config,
            env,
            db,
            meta_db,
        })
    }

    fn get_current_time(&self) -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    }

    pub async fn put(&self, key: Vec<u8>, value: Vec<u8>, ttl: i32) -> Result<(), StorageError> {
        if !self.has_space(value.len()) {
            return Err(StorageError::StorageFull);
        }

        let expires_at = self.get_current_time() + ttl as f64;
        let meta = MetaData {
            expires_at,
            size: value.len(),
        };

        // Сериализация метаданных в msgpack
        let meta_bytes = serialize(&meta, "msgpack").map_err(|_| StorageError::General)?;

        let env = self.env.clone();
        let db = self.db;
        let meta_db = self.meta_db;

        task::spawn_blocking(move || {
            let mut txn = env.write_txn().unwrap();
            db.put(&mut txn, &key, &value).unwrap();
            meta_db.put(&mut txn, &key, &meta_bytes).unwrap();
            txn.commit().unwrap();
        })
        .await
        .map_err(|_| StorageError::General)?;

        Ok(())
    }

    pub async fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, StorageError> {
        let env = self.env.clone();
        let db = self.db;
        let meta_db = self.meta_db;
        let current_time = self.get_current_time();

        // Мы должны клонировать ключ для использования в spawn_blocking
        let key_clone = key.clone();

        let result = task::spawn_blocking(move || {
            let txn = env.read_txn().unwrap();

            // Проверка TTL
            if let Some(meta_bytes) = meta_db.get(&txn, &key_clone).unwrap() {
                let meta: MetaData = deserialize(meta_bytes, "msgpack").unwrap();
                if current_time > meta.expires_at {
                    return Ok(None); // Сигнал, что истек TTL
                }
            }

            // Получение значения
            let value = db.get(&txn, &key_clone).unwrap().map(|b| b.to_vec());
            Ok(value)
        })
        .await
        .map_err(|_| StorageError::General)??;

        if result.is_none() {
            // Если ключ просрочен (вернули None выше), удаляем его
            // В Python это было внутри get, поэтому повторяем логику
            self.delete(key).await?;
            return Ok(None);
        }

        Ok(result)
    }

    pub async fn delete(&self, key: Vec<u8>) -> Result<(), StorageError> {
        let env = self.env.clone();
        let db = self.db;
        let meta_db = self.meta_db;

        task::spawn_blocking(move || {
            let mut txn = env.write_txn().unwrap();
            db.delete(&mut txn, &key).unwrap();
            meta_db.delete(&mut txn, &key).unwrap();
            txn.commit().unwrap();
        })
        .await
        .map_err(|_| StorageError::General)?;

        Ok(())
    }

    pub async fn extend_ttl(&self, key: Vec<u8>, extension: f64) -> Result<bool, StorageError> {
        let env = self.env.clone();
        let meta_db = self.meta_db;
        let current_time = self.get_current_time();

        task::spawn_blocking(move || {
            let mut txn = env.write_txn().unwrap();
            let meta_data = meta_db.get(&txn, &key).unwrap();

            if let Some(bytes) = meta_data {
                let mut meta: MetaData = deserialize(bytes, "msgpack").unwrap();
                let current_ttl = meta.expires_at - current_time;
                let new_ttl = current_ttl * (1.0 + extension);
                meta.expires_at = current_time + new_ttl;

                let new_meta_bytes = serialize(&meta, "msgpack").unwrap();
                meta_db.put(&mut txn, &key, &new_meta_bytes).unwrap();
                txn.commit().unwrap();
                Ok(true)
            } else {
                Ok(false)
            }
        })
        .await
        .map_err(|_| StorageError::General)?
    }

    fn has_space(&self, _size: usize) -> bool {
        // Заглушка из оригинального кода
        true
    }

    pub async fn cleanup_expired(&self) -> Result<i32, StorageError> {
        let env = self.env.clone();
        let db = self.db;
        let meta_db = self.meta_db;
        let current_time = self.get_current_time();

        task::spawn_blocking(move || {
            let mut deleted_count = 0;
            let mut txn = env.write_txn().unwrap();

            // В heed мы используем итераторы.
            // Собираем ключи для удаления, чтобы не нарушать итератор
            let mut to_delete = Vec::new();

            {
                let iter = meta_db.iter(&txn).unwrap();
                for item in iter {
                    let (key_bytes, meta_bytes) = item.unwrap();
                    let meta: MetaData = deserialize(meta_bytes, "msgpack").unwrap();
                    if current_time > meta.expires_at {
                        to_delete.push(key_bytes.to_vec());
                    }
                }
            }

            for key in to_delete {
                db.delete(&mut txn, &key).unwrap();
                meta_db.delete(&mut txn, &key).unwrap();
                deleted_count += 1;
            }

            txn.commit().unwrap();
            Ok(deleted_count)
        })
        .await
        .map_err(|_| StorageError::General)?
    }

    pub fn close(self) {
        // В Rust Env закрывается автоматически, когда выходит из области видимости (Drop)
        // Но для явности можно вызвать метод закрытия, если библиотека это поддерживает
        // heed не требует явного закрытия, так как использует RAII
    }
}
