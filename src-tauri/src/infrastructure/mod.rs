//! Infrastructure adapters composed by the desktop boundary.

mod logging;
mod scanner;
mod sqlite;
mod thumbnail;

pub(crate) use logging::{LoggingGuard, initialize_logging};
pub(crate) use scanner::FilesystemScanner;
pub(crate) use sqlite::SqliteDatabase;
pub(crate) use thumbnail::ThumbnailService;
