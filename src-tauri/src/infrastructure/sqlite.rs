use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::{
    application::{
        ApplicationError, BookListItem, CatalogReconciliation, DatabaseHealth, DiscoveredBook,
        LibraryConfiguration, LibraryConfigurationState, LibraryError, LibraryRepository,
        ScanReason, ScanResult, ScanSummary, ThumbnailOutcome,
    },
    domain::{BookId, BookStatus, LibraryId, RelativePath},
};

const DATABASE_FILENAME: &str = "book-library.sqlite3";

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (
        1,
        "initial_foundation",
        r#"
        CREATE TABLE application_settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        ) STRICT;

        CREATE TABLE configured_libraries (
            id TEXT PRIMARY KEY NOT NULL,
            root_path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        ) STRICT;
    "#,
    ),
    (
        2,
        "library_catalog",
        r#"
        CREATE TABLE scan_jobs (
            id TEXT PRIMARY KEY NOT NULL,
            library_id TEXT NOT NULL REFERENCES configured_libraries(id),
            status TEXT NOT NULL,
            reason TEXT NOT NULL,
            started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            finished_at TEXT,
            discovered_count INTEGER NOT NULL DEFAULT 0,
            added_count INTEGER NOT NULL DEFAULT 0,
            updated_count INTEGER NOT NULL DEFAULT 0,
            missing_count INTEGER NOT NULL DEFAULT 0,
            error_count INTEGER NOT NULL DEFAULT 0
        ) STRICT;

        CREATE TABLE scan_issues (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_job_id TEXT NOT NULL REFERENCES scan_jobs(id) ON DELETE CASCADE,
            relative_path TEXT,
            severity TEXT NOT NULL,
            code TEXT NOT NULL,
            message TEXT NOT NULL
        ) STRICT;

        CREATE TABLE books (
            id TEXT PRIMARY KEY NOT NULL,
            library_id TEXT NOT NULL REFERENCES configured_libraries(id),
            kind TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            path_key TEXT NOT NULL,
            title TEXT NOT NULL,
            title_source TEXT NOT NULL DEFAULT 'derived',
            status TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            size_bytes INTEGER,
            modified_at_ms INTEGER,
            page_count INTEGER,
            thumbnail_cache_path TEXT,
            thumbnail_status TEXT NOT NULL DEFAULT 'pending',
            last_seen_scan_id TEXT REFERENCES scan_jobs(id),
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(library_id, path_key)
        ) STRICT;

        CREATE TABLE image_pages (
            book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            page_index INTEGER NOT NULL,
            relative_path TEXT NOT NULL,
            PRIMARY KEY(book_id, page_index),
            UNIQUE(book_id, relative_path)
        ) STRICT;

        CREATE TABLE thumbnails (
            book_id TEXT PRIMARY KEY NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            cache_relative_path TEXT NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            format TEXT NOT NULL,
            source_fingerprint TEXT NOT NULL,
            status TEXT NOT NULL,
            error_code TEXT,
            generated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        ) STRICT;
    "#,
    ),
];

#[derive(Debug, Error)]
pub(crate) enum DatabaseInitializationError {
    #[error("application data directory could not be created")]
    CreateAppData(#[source] std::io::Error),
    #[error("database could not be opened")]
    Open(#[source] rusqlite::Error),
    #[error("database connection could not be configured")]
    Configure(#[source] rusqlite::Error),
    #[error("database migration failed")]
    Migration(#[source] rusqlite::Error),
}

#[derive(Debug)]
pub(crate) struct SqliteDatabase {
    connection: Mutex<Connection>,
    cache_root: PathBuf,
}

impl SqliteDatabase {
    pub(crate) fn initialize(app_data_dir: &Path) -> Result<Self, DatabaseInitializationError> {
        fs::create_dir_all(app_data_dir).map_err(DatabaseInitializationError::CreateAppData)?;
        let path = app_data_dir.join(DATABASE_FILENAME);
        let cache_root = app_data_dir.join("cache");
        fs::create_dir_all(cache_root.join("thumbnails"))
            .map_err(DatabaseInitializationError::CreateAppData)?;
        let mut connection = Connection::open(&path).map_err(DatabaseInitializationError::Open)?;

        Self::configure_connection(&connection)?;
        Self::run_migrations(&mut connection)?;

        Ok(Self {
            connection: Mutex::new(connection),
            cache_root,
        })
    }

    fn configure_connection(connection: &Connection) -> Result<(), DatabaseInitializationError> {
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;
                PRAGMA busy_timeout = 5000;
                ",
            )
            .map_err(DatabaseInitializationError::Configure)
    }

    fn run_migrations(connection: &mut Connection) -> Result<(), DatabaseInitializationError> {
        let transaction = connection
            .transaction()
            .map_err(DatabaseInitializationError::Migration)?;

        transaction
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS schema_migrations (
                    version INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL UNIQUE,
                    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                ) STRICT;
                ",
            )
            .map_err(DatabaseInitializationError::Migration)?;

        for (version, name, sql) in MIGRATIONS {
            let applied = transaction
                .query_row(
                    "SELECT 1 FROM schema_migrations WHERE version = ?1",
                    [version],
                    |_| Ok(()),
                )
                .optional()
                .map_err(DatabaseInitializationError::Migration)?
                .is_some();

            if !applied {
                transaction
                    .execute_batch(sql)
                    .map_err(DatabaseInitializationError::Migration)?;
                transaction
                    .execute(
                        "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
                        params![version, name],
                    )
                    .map_err(DatabaseInitializationError::Migration)?;
            }
        }

        transaction
            .commit()
            .map_err(DatabaseInitializationError::Migration)
    }

    fn configuration_state(id: LibraryId, root: &Path) -> LibraryConfigurationState {
        LibraryConfigurationState {
            id,
            root: root.to_path_buf(),
            display_name: root
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Library")
                .to_owned(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn upsert_book(
        transaction: &rusqlite::Transaction<'_>,
        library_id: LibraryId,
        job_id: &str,
        book: &DiscoveredBook,
        existing: Option<(String, String, String, String, String)>,
        added: &mut u64,
        updated: &mut u64,
    ) -> Result<(BookId, bool), LibraryError> {
        if let Some((id, old_fingerprint, old_path, old_status, thumbnail_status)) = existing {
            let changed = old_fingerprint != book.fingerprint.as_str()
                || old_path != book.relative_path.as_str()
                || old_status != book.status.as_str();
            transaction
                .execute(
                    "UPDATE books SET relative_path = ?1, kind = ?2,
                       title = CASE WHEN title_source = 'derived' THEN ?3 ELSE title END,
                       fingerprint = ?4, status = ?5, size_bytes = ?6,
                       modified_at_ms = ?7, page_count = ?8,
                       thumbnail_status = CASE
                         WHEN fingerprint <> ?4 OR status <> ?5
                         THEN CASE WHEN ?5 = 'available' THEN 'pending' ELSE 'error' END
                         ELSE thumbnail_status
                       END,
                       last_seen_scan_id = ?9, updated_at = CURRENT_TIMESTAMP
                     WHERE id = ?10",
                    params![
                        book.relative_path.as_str(),
                        book.kind.as_str(),
                        book.title,
                        book.fingerprint.as_str(),
                        book.status.as_str(),
                        book.size_bytes.and_then(|value| i64::try_from(value).ok()),
                        book.modified_at_ms,
                        book.page_count,
                        job_id,
                        id
                    ],
                )
                .map_err(|_| LibraryError::CatalogFailed)?;
            if changed {
                *updated += 1;
            }
            Ok((
                BookId::parse(&id).map_err(|_| LibraryError::CatalogFailed)?,
                book.status == BookStatus::Available
                    && (old_fingerprint != book.fingerprint.as_str()
                        || old_status != book.status.as_str()
                        || thumbnail_status != "ready"),
            ))
        } else {
            let id = BookId::new();
            let thumbnail_status = if book.status == BookStatus::Available {
                "pending"
            } else {
                "error"
            };
            transaction
                .execute(
                    "INSERT INTO books
                     (id, library_id, kind, relative_path, path_key, title, status,
                      fingerprint, size_bytes, modified_at_ms, page_count, thumbnail_status,
                      last_seen_scan_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                    params![
                        id.to_string(),
                        library_id.to_string(),
                        book.kind.as_str(),
                        book.relative_path.as_str(),
                        book.path_key,
                        book.title,
                        book.status.as_str(),
                        book.fingerprint.as_str(),
                        book.size_bytes.and_then(|value| i64::try_from(value).ok()),
                        book.modified_at_ms,
                        book.page_count,
                        thumbnail_status,
                        job_id
                    ],
                )
                .map_err(|_| LibraryError::CatalogFailed)?;
            *added += 1;
            Ok((id, book.status == BookStatus::Available))
        }
    }
}

impl DatabaseHealth for SqliteDatabase {
    fn check_health(&self) -> Result<(), ApplicationError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| ApplicationError::DatabaseUnavailable)?;
        connection
            .query_row("SELECT 1", [], |_| Ok(()))
            .map_err(|_| ApplicationError::DatabaseUnavailable)
    }
}

impl LibraryConfiguration for SqliteDatabase {
    fn has_configured_library(&self) -> Result<bool, ApplicationError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| ApplicationError::ConfigurationUnavailable)?;
        connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM configured_libraries LIMIT 1)",
                [],
                |row| row.get(0),
            )
            .map_err(|_| ApplicationError::ConfigurationUnavailable)
    }
}

impl LibraryRepository for SqliteDatabase {
    fn save_configuration(
        &self,
        root: &Path,
        _display_name: &str,
    ) -> Result<LibraryConfigurationState, LibraryError> {
        let root_text = root.to_str().ok_or(LibraryError::RootInvalid)?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::ConfigurationFailed)?;
        let existing_id: Option<String> = connection
            .query_row(
                "SELECT id FROM configured_libraries ORDER BY created_at LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| LibraryError::ConfigurationFailed)?;
        let id = if let Some(value) = existing_id {
            connection
                .execute(
                    "UPDATE configured_libraries SET root_path = ?1 WHERE id = ?2",
                    params![root_text, value],
                )
                .map_err(|_| LibraryError::ConfigurationFailed)?;
            LibraryId::parse(&value).map_err(|_| LibraryError::ConfigurationFailed)?
        } else {
            let id = LibraryId::new();
            connection
                .execute(
                    "INSERT INTO configured_libraries (id, root_path) VALUES (?1, ?2)",
                    params![id.to_string(), root_text],
                )
                .map_err(|_| LibraryError::ConfigurationFailed)?;
            id
        };

        Ok(Self::configuration_state(id, root))
    }

    fn configuration(&self) -> Result<Option<LibraryConfigurationState>, LibraryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::ConfigurationFailed)?;
        let row: Option<(String, String)> = connection
            .query_row(
                "SELECT id, root_path FROM configured_libraries ORDER BY created_at LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|_| LibraryError::ConfigurationFailed)?;
        row.map(|(id, root)| {
            let id = LibraryId::parse(&id).map_err(|_| LibraryError::ConfigurationFailed)?;
            Ok(Self::configuration_state(id, &PathBuf::from(root)))
        })
        .transpose()
    }

    fn start_scan(
        &self,
        library_id: LibraryId,
        reason: ScanReason,
    ) -> Result<String, LibraryError> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::CatalogFailed)?;
        connection
            .execute(
                "INSERT INTO scan_jobs (id, library_id, status, reason)
                 VALUES (?1, ?2, 'running', ?3)",
                params![job_id, library_id.to_string(), reason.as_str()],
            )
            .map_err(|_| LibraryError::CatalogFailed)?;
        Ok(job_id)
    }

    fn reconcile(
        &self,
        library_id: LibraryId,
        job_id: &str,
        result: &ScanResult,
    ) -> Result<CatalogReconciliation, LibraryError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::CatalogFailed)?;
        let transaction = connection
            .transaction()
            .map_err(|_| LibraryError::CatalogFailed)?;
        let mut added = 0_u64;
        let mut updated = 0_u64;
        let mut thumbnail_targets = Vec::new();

        for issue in &result.issues {
            transaction
                .execute(
                    "INSERT INTO scan_issues
                     (scan_job_id, relative_path, severity, code, message)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        job_id,
                        issue.relative_path,
                        issue.severity,
                        issue.code,
                        issue.message
                    ],
                )
                .map_err(|_| LibraryError::CatalogFailed)?;
        }

        for book in &result.books {
            let existing: Option<(String, String, String, String, String)> = transaction
                .query_row(
                    "SELECT id, fingerprint, relative_path, status, thumbnail_status
                     FROM books WHERE library_id = ?1 AND path_key = ?2",
                    params![library_id.to_string(), book.path_key],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    },
                )
                .optional()
                .map_err(|_| LibraryError::CatalogFailed)?;
            let (book_id, needs_thumbnail) = Self::upsert_book(
                &transaction,
                library_id,
                job_id,
                book,
                existing,
                &mut added,
                &mut updated,
            )?;

            transaction
                .execute(
                    "DELETE FROM image_pages WHERE book_id = ?1",
                    [book_id.to_string()],
                )
                .map_err(|_| LibraryError::CatalogFailed)?;
            for (index, page) in book.image_pages.iter().enumerate() {
                transaction
                    .execute(
                        "INSERT INTO image_pages (book_id, page_index, relative_path)
                         VALUES (?1, ?2, ?3)",
                        params![
                            book_id.to_string(),
                            i64::try_from(index).unwrap_or(i64::MAX),
                            page.as_str()
                        ],
                    )
                    .map_err(|_| LibraryError::CatalogFailed)?;
            }
            if needs_thumbnail {
                thumbnail_targets.push((book_id, book.clone()));
            }
        }

        let missing = if result.cancelled {
            0
        } else {
            transaction
                .execute(
                    "UPDATE books SET status = 'missing', updated_at = CURRENT_TIMESTAMP
                     WHERE library_id = ?1 AND status <> 'missing'
                       AND (last_seen_scan_id IS NULL OR last_seen_scan_id <> ?2)",
                    params![library_id.to_string(), job_id],
                )
                .map_err(|_| LibraryError::CatalogFailed)? as u64
        };
        transaction
            .commit()
            .map_err(|_| LibraryError::CatalogFailed)?;
        Ok(CatalogReconciliation {
            added,
            updated,
            missing,
            thumbnail_targets,
        })
    }

    fn save_thumbnail(
        &self,
        book_id: BookId,
        outcome: &ThumbnailOutcome,
    ) -> Result<(), LibraryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::CatalogFailed)?;
        connection
            .execute(
                "INSERT INTO thumbnails
                 (book_id, cache_relative_path, width, height, format, source_fingerprint, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'ready')
                 ON CONFLICT(book_id) DO UPDATE SET
                   cache_relative_path = excluded.cache_relative_path,
                   width = excluded.width, height = excluded.height,
                   format = excluded.format,
                   source_fingerprint = excluded.source_fingerprint,
                   status = 'ready', error_code = NULL,
                   generated_at = CURRENT_TIMESTAMP",
                params![
                    book_id.to_string(),
                    outcome.cache_relative_path,
                    outcome.width,
                    outcome.height,
                    outcome.format,
                    outcome.source_fingerprint
                ],
            )
            .and_then(|_| {
                connection.execute(
                    "UPDATE books SET thumbnail_cache_path = ?1, thumbnail_status = 'ready'
                     , page_count = COALESCE(?2, page_count) WHERE id = ?3",
                    params![
                        outcome.cache_relative_path,
                        outcome.page_count,
                        book_id.to_string()
                    ],
                )
            })
            .map_err(|_| LibraryError::CatalogFailed)?;
        Ok(())
    }

    fn save_thumbnail_failure(
        &self,
        book_id: BookId,
        error_code: &'static str,
    ) -> Result<(), LibraryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::CatalogFailed)?;
        connection
            .execute(
                "UPDATE books SET thumbnail_status = 'error' WHERE id = ?1",
                [book_id.to_string()],
            )
            .and_then(|_| {
                connection.execute(
                    "INSERT INTO thumbnails
                     (book_id, cache_relative_path, width, height, format,
                      source_fingerprint, status, error_code)
                     VALUES (?1, '', 0, 0, 'png', '', 'error', ?2)
                     ON CONFLICT(book_id) DO UPDATE SET
                       status = 'error', error_code = excluded.error_code",
                    params![book_id.to_string(), error_code],
                )
            })
            .map_err(|_| LibraryError::CatalogFailed)?;
        Ok(())
    }

    fn finish_scan(&self, summary: &ScanSummary) -> Result<(), LibraryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::CatalogFailed)?;
        connection
            .execute(
                "UPDATE scan_jobs SET status = ?1, finished_at = CURRENT_TIMESTAMP,
                   discovered_count = ?2, added_count = ?3, updated_count = ?4,
                   missing_count = ?5, error_count = ?6 WHERE id = ?7",
                params![
                    if summary.cancelled {
                        "cancelled"
                    } else {
                        "completed"
                    },
                    i64::try_from(summary.discovered).unwrap_or(i64::MAX),
                    i64::try_from(summary.added).unwrap_or(i64::MAX),
                    i64::try_from(summary.updated).unwrap_or(i64::MAX),
                    i64::try_from(summary.missing).unwrap_or(i64::MAX),
                    i64::try_from(summary.issues + summary.thumbnail_failures).unwrap_or(i64::MAX),
                    summary.job_id
                ],
            )
            .map_err(|_| LibraryError::CatalogFailed)?;
        Ok(())
    }

    fn list_books(&self) -> Result<Vec<BookListItem>, LibraryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::CatalogFailed)?;
        let mut statement = connection
            .prepare(
                "SELECT id, title, kind, relative_path, status, page_count,
                        size_bytes, modified_at_ms, thumbnail_cache_path, thumbnail_status
                 FROM books ORDER BY title COLLATE NOCASE, relative_path",
            )
            .map_err(|_| LibraryError::CatalogFailed)?;
        let rows = statement
            .query_map([], |row| {
                Ok(BookListItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    kind: row.get(2)?,
                    relative_path: row.get(3)?,
                    status: row.get(4)?,
                    page_count: row.get(5)?,
                    size_bytes: row
                        .get::<_, Option<i64>>(6)?
                        .and_then(|value| u64::try_from(value).ok()),
                    modified_at_ms: row.get(7)?,
                    thumbnail_cache_path: row.get(8)?,
                    thumbnail_status: row.get(9)?,
                })
            })
            .map_err(|_| LibraryError::CatalogFailed)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| LibraryError::CatalogFailed)
    }

    fn invalidate_thumbnails(&self) -> Result<(), LibraryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::CatalogFailed)?;
        connection
            .execute(
                "UPDATE books SET thumbnail_status = 'pending', thumbnail_cache_path = NULL
                 WHERE status = 'available'",
                [],
            )
            .map_err(|_| LibraryError::CatalogFailed)?;
        Ok(())
    }

    fn thumbnail_bytes(&self, cache_relative_path: &str) -> Result<Vec<u8>, LibraryError> {
        let relative =
            RelativePath::new(cache_relative_path).map_err(|_| LibraryError::ThumbnailFailed)?;
        let resolved = self
            .cache_root
            .join(relative.as_str())
            .canonicalize()
            .map_err(|_| LibraryError::ThumbnailFailed)?;
        let root = self
            .cache_root
            .canonicalize()
            .map_err(|_| LibraryError::ThumbnailFailed)?;
        if !resolved.starts_with(root) {
            return Err(LibraryError::ThumbnailFailed);
        }
        fs::read(resolved).map_err(|_| LibraryError::ThumbnailFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::{
            CancellationToken, DiscoveredBook, LibraryRepository, ReconcileCatalog, ScanResult,
            ThumbnailGenerator, ThumbnailOutcome,
        },
        domain::{BookId, BookKind, ContentFingerprint, RelativePath},
        infrastructure::FilesystemScanner,
    };
    use std::{
        sync::{Arc, Barrier},
        thread,
    };
    use tempfile::TempDir;

    struct SkipRealLibraryThumbnails;

    impl ThumbnailGenerator for SkipRealLibraryThumbnails {
        fn generate(
            &self,
            _root: &Path,
            _book_id: BookId,
            _book: &DiscoveredBook,
        ) -> Result<ThumbnailOutcome, LibraryError> {
            Err(LibraryError::ThumbnailFailed)
        }
    }

    #[test]
    fn initializes_outside_a_separate_library_fixture() {
        let app_data = TempDir::new().unwrap();
        let library = TempDir::new().unwrap();

        SqliteDatabase::initialize(app_data.path()).unwrap();

        assert!(app_data.path().join(DATABASE_FILENAME).is_file());
        assert_eq!(fs::read_dir(library.path()).unwrap().count(), 0);
    }

    #[test]
    fn applies_each_migration_exactly_once() {
        let app_data = TempDir::new().unwrap();
        SqliteDatabase::initialize(app_data.path()).unwrap();
        let database = SqliteDatabase::initialize(app_data.path()).unwrap();
        let connection = database.connection.lock().unwrap();

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
    }

    #[test]
    fn enables_foreign_keys_on_every_initialized_connection() {
        let app_data = TempDir::new().unwrap();
        let database = SqliteDatabase::initialize(app_data.path()).unwrap();
        let connection = database.connection.lock().unwrap();

        let enabled: bool = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert!(enabled);
    }

    #[test]
    fn exposes_healthy_unconfigured_status() {
        let app_data = TempDir::new().unwrap();
        let database = SqliteDatabase::initialize(app_data.path()).unwrap();

        assert!(database.check_health().is_ok());
        assert!(!database.has_configured_library().unwrap());
    }

    #[test]
    fn returns_a_typed_error_when_migration_state_is_incompatible() {
        let app_data = TempDir::new().unwrap();
        let path = app_data.path().join(DATABASE_FILENAME);
        let connection = Connection::open(path).unwrap();
        connection
            .execute_batch(
                "
                CREATE TABLE schema_migrations (
                    incompatible_column TEXT NOT NULL
                );
                ",
            )
            .unwrap();
        drop(connection);

        let error = SqliteDatabase::initialize(app_data.path()).unwrap_err();
        assert!(matches!(error, DatabaseInitializationError::Migration(_)));
    }

    #[test]
    fn serializes_concurrent_writes_without_enabling_wal() {
        let app_data = TempDir::new().unwrap();
        let database = Arc::new(SqliteDatabase::initialize(app_data.path()).unwrap());
        let barrier = Arc::new(Barrier::new(5));
        let mut workers = Vec::new();

        for worker_id in 0..4 {
            let database = Arc::clone(&database);
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                let connection = database.connection.lock().unwrap();
                connection
                    .execute(
                        "INSERT INTO application_settings (key, value) VALUES (?1, ?2)",
                        params![format!("worker-{worker_id}"), "complete"],
                    )
                    .unwrap();
            }));
        }

        barrier.wait();
        for worker in workers {
            worker.join().unwrap();
        }

        let connection = database.connection.lock().unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM application_settings", [], |row| {
                row.get(0)
            })
            .unwrap();
        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 4);
        assert_eq!(journal_mode.to_ascii_lowercase(), "delete");
    }

    #[test]
    fn catalog_reconciliation_is_idempotent_preserves_user_title_and_marks_missing() {
        let app_data = TempDir::new().unwrap();
        let library = TempDir::new().unwrap();
        let database = SqliteDatabase::initialize(app_data.path()).unwrap();
        let configuration = database
            .save_configuration(library.path(), "Library")
            .unwrap();
        let book = DiscoveredBook {
            kind: BookKind::PdfFile,
            status: BookStatus::Available,
            relative_path: RelativePath::new("Shelf/Book.pdf").unwrap(),
            path_key: if cfg!(target_os = "windows") {
                "shelf/book.pdf".to_owned()
            } else {
                "Shelf/Book.pdf".to_owned()
            },
            title: "Book".to_owned(),
            fingerprint: ContentFingerprint::new("pdf:10:20").unwrap(),
            size_bytes: Some(10),
            modified_at_ms: Some(20),
            page_count: None,
            image_pages: Vec::new(),
        };
        let scan = ScanResult {
            books: vec![book.clone()],
            issues: Vec::new(),
            cancelled: false,
        };

        let first_job = database
            .start_scan(configuration.id, ScanReason::Initial)
            .unwrap();
        let first = database
            .reconcile(configuration.id, &first_job, &scan)
            .unwrap();
        assert_eq!(first.added, 1);

        {
            let connection = database.connection.lock().unwrap();
            connection
                .execute(
                    "UPDATE books SET title = 'My title', title_source = 'user'",
                    [],
                )
                .unwrap();
        }
        let second_job = database
            .start_scan(configuration.id, ScanReason::Manual)
            .unwrap();
        let second = database
            .reconcile(configuration.id, &second_job, &scan)
            .unwrap();
        assert_eq!(second.added, 0);
        assert_eq!(second.updated, 0);
        let books = database.list_books().unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].title, "My title");

        let third_job = database
            .start_scan(configuration.id, ScanReason::Manual)
            .unwrap();
        let missing = database
            .reconcile(
                configuration.id,
                &third_job,
                &ScanResult {
                    books: Vec::new(),
                    issues: Vec::new(),
                    cancelled: false,
                },
            )
            .unwrap();
        assert_eq!(missing.missing, 1);
        assert_eq!(database.list_books().unwrap()[0].status, "missing");
    }

    #[test]
    fn unavailable_books_are_cataloged_without_thumbnail_work() {
        let app_data = TempDir::new().unwrap();
        let library = TempDir::new().unwrap();
        let database = SqliteDatabase::initialize(app_data.path()).unwrap();
        let configuration = database
            .save_configuration(library.path(), "Library")
            .unwrap();
        let book = DiscoveredBook {
            kind: BookKind::PdfFile,
            status: BookStatus::Unavailable,
            relative_path: RelativePath::new("Cloud/Book.pdf").unwrap(),
            path_key: if cfg!(target_os = "windows") {
                "cloud/book.pdf".to_owned()
            } else {
                "Cloud/Book.pdf".to_owned()
            },
            title: "Book".to_owned(),
            fingerprint: ContentFingerprint::new("pdf-unavailable:Cloud/Book.pdf").unwrap(),
            size_bytes: None,
            modified_at_ms: None,
            page_count: None,
            image_pages: Vec::new(),
        };
        let job = database
            .start_scan(configuration.id, ScanReason::Initial)
            .unwrap();
        let reconciliation = database
            .reconcile(
                configuration.id,
                &job,
                &ScanResult {
                    books: vec![book],
                    issues: Vec::new(),
                    cancelled: false,
                },
            )
            .unwrap();

        assert_eq!(reconciliation.added, 1);
        assert!(reconciliation.thumbnail_targets.is_empty());
        let books = database.list_books().unwrap();
        assert_eq!(books[0].status, "unavailable");
        assert_eq!(books[0].thumbnail_status, "error");
    }

    #[test]
    #[ignore = "requires BOOK_LIBRARY_SMOKE_ROOT and a real read-only library"]
    fn real_library_scan_is_idempotent_and_non_destructive() {
        let root = PathBuf::from(
            std::env::var("BOOK_LIBRARY_SMOKE_ROOT")
                .expect("BOOK_LIBRARY_SMOKE_ROOT must point to the real library"),
        );
        let root_inventory = |root: &Path| {
            let mut entries = std::fs::read_dir(root)
                .unwrap()
                .filter_map(Result::ok)
                .filter_map(|entry| Some((entry.file_name(), entry.file_type().ok()?.is_dir())))
                .collect::<Vec<_>>();
            entries.sort();
            entries
        };
        let before = root_inventory(&root);
        eprintln!("root inventory captured: {}", before.len());
        let app_data = TempDir::new().unwrap();
        let database = SqliteDatabase::initialize(app_data.path()).unwrap();
        database.save_configuration(&root, "Library").unwrap();
        eprintln!("temporary catalog configured");
        let scanner = FilesystemScanner::new();
        let thumbnails = SkipRealLibraryThumbnails;
        let use_case = ReconcileCatalog::new(&database, &scanner, &thumbnails);
        let mut progress = |value: crate::application::ScanProgress| {
            if value.visited_entries == 1 || value.visited_entries.is_multiple_of(100) {
                eprintln!(
                    "visited={} discovered={}",
                    value.visited_entries, value.discovered_books
                );
            }
        };
        eprintln!("starting first scan");
        let first = use_case
            .execute(
                ScanReason::Initial,
                &CancellationToken::default(),
                &mut progress,
            )
            .unwrap();
        eprintln!(
            "first scan complete: discovered={} thumbnails={}",
            first.discovered, first.thumbnails_generated
        );
        assert!(first.discovered > 0);
        let second = use_case
            .execute(
                ScanReason::Manual,
                &CancellationToken::default(),
                &mut progress,
            )
            .unwrap();
        eprintln!("second scan complete: added={}", second.added);
        assert_eq!(second.added, 0);
        assert_eq!(
            database.list_books().unwrap().len() as u64,
            first.discovered
        );
        assert_eq!(root_inventory(&root), before);
    }
}
