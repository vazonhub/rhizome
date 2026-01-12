use std::fs::File;
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Настройка системы логирования
#[allow(dead_code)]
pub fn setup_logging(log_level: &str, log_file: Option<PathBuf>, node_id: Option<&str>) {
    // Настройка фильтра уровня логирования
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // Настройка таймстемпа (ISO)
    let timer = fmt::time::ChronoLocal::rfc_3339();

    // Выбор рендерера и вывод (JSON в файл или текст в консоль)
    if let Some(path) = log_file {
        let file = File::create(path).expect("Не удалось создать файл лога");
        let layer = fmt::layer()
            .with_timer(timer)
            .json() // JSONRenderer
            .with_writer(file);

        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();
    } else {
        let layer = fmt::layer().with_timer(timer).with_writer(std::io::stdout); // ConsoleRenderer

        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();
    }

    // Аналог logger.bind(node_id=...)
    if let Some(id) = node_id {
        let truncated_id = if id.len() > 16 { &id[..16] } else { id };
        tracing::info!(node_id = %truncated_id, "Logging initialized");
    }
}

/// Получение логгера для модуля
/// В Rust tracing автоматически добавляет имя модуля,
/// но можно добавить дополнительный контекст
#[allow(dead_code)]
pub fn get_logger(name: &'static str) {
    tracing::info!(module = name, "Module logger initialized");
}
