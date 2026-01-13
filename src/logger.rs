use std::fs::File;
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[allow(dead_code)]
pub fn setup_logging(log_level: &str, log_file: Option<PathBuf>, node_id: Option<&str>) {
    // Configure log level filter
    // Uses RUST_LOG environment variable if set, otherwise uses the provided level
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // Configure timestamp format using ISO 8601 (RFC 3339)
    // Format: 2024-01-12T15:30:45.123456789+03:00
    let timer = fmt::time::ChronoLocal::rfc_3339();

    // Choose renderer and output (JSON to file or text to console)
    if let Some(path) = log_file {
        // Create file for logging
        // Panics with error message if file cannot be created
        let file = File::create(path).expect("Failed to create log file");

        // Create layer for JSON file output
        let layer = fmt::layer()
            .with_timer(timer) // Add timestamps
            .json() // Use JSON format
            .with_writer(file); // Write to file

        // Initialize subscriber with filter and file layer
        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();
    } else {
        // Create layer for console text output
        let layer = fmt::layer()
            .with_timer(timer) // Add timestamps
            .with_writer(std::io::stdout); // Output to stdout

        // Initialize subscriber with filter and console layer
        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();
    }

    // Analogous to logger.bind(node_id=...) - log initialization with node ID
    // If node identifier is provided, logs it in the first log entry
    if let Some(id) = node_id {
        // Truncate identifier to 16 characters for readability
        let truncated_id = if id.len() > 16 { &id[..16] } else { id };

        // Log informational message about logging initialization
        // Uses structured logging with node_id field
        tracing::info!(
            node_id = %truncated_id,  // Field for node identifier
            "Logging initialized"     // Message
        );
    }
}

#[allow(dead_code)]
pub fn get_logger(name: &'static str) {
    // Log informational message about module logger initialization
    // Uses structured logging with module field
    tracing::info!(
        module = name,               // Field for module name
        "Module logger initialized"  // Message
    );
}
