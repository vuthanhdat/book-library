//! Infrastructure adapters composed by the desktop boundary.

mod file_manager;
mod logging;
mod markdown_notes;
mod scanner;
mod sqlite;
mod thumbnail;

pub(crate) use file_manager::SystemFileManager;
pub(crate) use logging::{LoggingGuard, initialize_logging};
pub(crate) use markdown_notes::MarkdownNotesStore;
pub(crate) use scanner::FilesystemScanner;
pub(crate) use sqlite::SqliteDatabase;
pub(crate) use thumbnail::ThumbnailService;
