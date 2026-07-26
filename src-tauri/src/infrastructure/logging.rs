use std::{fs, path::Path};

use thiserror::Error;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;

#[derive(Debug, Error)]
pub(crate) enum LoggingInitializationError {
    #[error("log directory could not be created")]
    CreateDirectory(#[source] std::io::Error),
    #[error("structured logging could not be initialized")]
    Subscriber(String),
}

pub(crate) struct LoggingGuard {
    _writer_guard: WorkerGuard,
}

pub(crate) fn initialize_logging(
    app_data_dir: &Path,
) -> Result<LoggingGuard, LoggingInitializationError> {
    let log_dir = app_data_dir.join("logs");
    fs::create_dir_all(&log_dir).map_err(LoggingInitializationError::CreateDirectory)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "book-library.jsonl");
    let (writer, writer_guard) = tracing_appender::non_blocking(file_appender);
    let max_level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .json()
        .with_max_level(max_level)
        .with_writer(writer)
        .with_current_span(false)
        .with_span_list(false)
        .try_init()
        .map_err(|error| LoggingInitializationError::Subscriber(error.to_string()))?;

    Ok(LoggingGuard {
        _writer_guard: writer_guard,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, thread, time::Duration};
    use tempfile::TempDir;

    #[test]
    fn writes_structured_logs_below_app_data_without_sensitive_content() {
        let app_data = TempDir::new().unwrap();
        let guard = initialize_logging(app_data.path()).unwrap();

        tracing::info!(
            event = "diagnostic_test_completed",
            operation_id = "test-operation",
            "safe diagnostic"
        );
        drop(guard);
        thread::sleep(Duration::from_millis(25));

        let log_dir = app_data.path().join("logs");
        let log_file = fs::read_dir(&log_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.is_file())
            .unwrap();
        let content = fs::read_to_string(log_file).unwrap();

        assert!(content.contains("\"event\":\"diagnostic_test_completed\""));
        assert!(content.contains("\"operation_id\":\"test-operation\""));
        assert!(!content.contains("note body"));
        assert!(!content.contains("api_key"));
    }
}
