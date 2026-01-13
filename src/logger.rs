//! # Logger Module
//!
//! This module provides a centralized utility for initializing the global logging system
//! based on the `tracing` ecosystem. It supports multiple output formats:
//! - **Console Output**: Optimized for human readability in the terminal.
//! - **File Output**: Structured JSON format, ideal for log aggregation and analysis.
//!
//! ## Features
//! - Configurable log levels via function arguments or `RUST_LOG` environment variable.
//! - Timestamp formatting using the RFC 3339 standard.
//! - Contextual logging (e.g., including Node IDs).

use std::fs::File;
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initializes the global tracing subscriber for the application.
///
/// This function should be called **once** during the application startup (usually in `main`).
/// It configures how logs are filtered, formatted, and where they are written.
///
/// # Arguments
///
/// * `log_level` - A string representing the default log level (e.g., "info", "debug", "trace").
/// * `log_file` - An optional path to a file. If `Some`, logs will be written in **JSON format** to that file.
/// * `node_id` - An optional string slice representing the unique ID of the current node for startup context.
///
/// # Panics
///
/// This function will panic if:
/// * It fails to create or open the file specified in `log_file`.
/// * The global subscriber has already been initialized by another part of the code.
///
/// # Examples
///
/// **Basic initialization to console:**
/// ```rust
/// use std::path::PathBuf;
/// // Logs with level "info" or higher will be printed to stdout
/// setup_logging("info", None, Some("node_12345"));
/// ```
///
/// **Initialization to a JSON file:**
/// ```rust
/// use std::path::PathBuf;
/// let path = PathBuf::from("logs/node.log");
/// setup_logging("debug", Some(path), None);
/// ```
#[allow(dead_code)]
pub fn setup_logging(log_level: &str, log_file: Option<PathBuf>, node_id: Option<&str>) {
    // Attempt to parse filter from RUST_LOG env var; fallback to the provided log_level
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // Configure timestamp format to RFC 3339 (Local time)
    let timer = fmt::time::ChronoLocal::rfc_3339();

    if let Some(path) = log_file {
        // Configure JSON file logging
        let file = File::create(path).expect("Failed to create log file");

        let layer = fmt::layer()
            .with_timer(timer)
            .json() // Enable structured JSON output for machine parsing
            .with_writer(file);

        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();
    } else {
        // Configure human-readable console logging
        let layer = fmt::layer().with_timer(timer).with_writer(std::io::stdout);

        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();
    }

    // Log the initialization event with an optional truncated Node ID
    if let Some(id) = node_id {
        let truncated_id = if id.len() > 16 { &id[..16] } else { id };

        tracing::info!(
            node_id = %truncated_id,
            "Logging initialized"
        );
    }
}

/// Logs an informational message indicating that a specific module has been initialized.
///
/// This is a helper function used to track the initialization sequence of different
/// components within the library.
///
/// # Arguments
///
/// * `name` - The name of the module or component being initialized.
///
/// # Examples
///
/// ```rust
/// get_logger("dht_protocol");
/// ```
#[allow(dead_code)]
pub fn get_logger(name: &'static str) {
    tracing::info!(module = name, "Module logger initialized");
}
