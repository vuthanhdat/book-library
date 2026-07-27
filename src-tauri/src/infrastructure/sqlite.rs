use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::{
    application::{
        ApplicationError, BookDetailError, BookDetailRecord, BookDetailRepository, BookListItem,
        BookLocationRepository, BookMetadataError, BookMetadataRepository, BookRelocationError,
        BookRelocationRepository, BookSourceLocation, BookThumbnailTarget, CatalogReconciliation,
        DatabaseHealth, DiscoveredBook, LibraryConfiguration, LibraryConfigurationState,
        LibraryError, LibraryRepository, LinkedBookNote, NoteBacklink, NoteDetail, NoteListItem,
        NoteProjection, NoteRecord, NotesConfiguration, NotesError, NotesRefreshSummary,
        NotesRepository, ScanReason, ScanResult, ScanSummary, SearchDiagnostics, SearchDocument,
        SearchError, SearchRebuildSummary, SearchRepository, SearchResultItem, SourceLocationError,
        ThumbnailOutcome,
    },
    domain::{BookId, BookKind, BookStatus, ContentFingerprint, LibraryId, NoteId, RelativePath},
};

const DATABASE_FILENAME: &str = "book-library.sqlite3";

type BookDetailRow = (
    String,
    String,
    String,
    String,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<String>,
    String,
    String,
);

type BookThumbnailRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    Option<i64>,
    Option<i64>,
    Option<i64>,
);

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
    (
        3,
        "markdown_notes",
        r#"
        CREATE TABLE configured_notes_root (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            root_path TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        ) STRICT;

        CREATE TABLE notes (
            id TEXT PRIMARY KEY NOT NULL,
            relative_path TEXT NOT NULL,
            path_key TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            status TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            modified_at_ms INTEGER,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        ) STRICT;

        CREATE TABLE book_note_links (
            note_id TEXT PRIMARY KEY NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            relation_kind TEXT NOT NULL DEFAULT 'about'
        ) STRICT;

        CREATE TABLE note_headings (
            note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            heading_index INTEGER NOT NULL,
            level INTEGER NOT NULL,
            text TEXT NOT NULL,
            PRIMARY KEY(note_id, heading_index)
        ) STRICT;

        CREATE TABLE note_tags (
            note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            tag TEXT NOT NULL,
            PRIMARY KEY(note_id, tag)
        ) STRICT;

        CREATE TABLE note_links (
            source_note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            link_index INTEGER NOT NULL,
            target_ref TEXT NOT NULL,
            link_text TEXT NOT NULL,
            resolved_note_id TEXT REFERENCES notes(id) ON DELETE SET NULL,
            PRIMARY KEY(source_note_id, link_index)
        ) STRICT;
    "#,
    ),
    (
        4,
        "offline_full_text_search",
        r#"
        CREATE VIRTUAL TABLE search_documents_fts USING fts5(
            source_kind UNINDEXED,
            source_id UNINDEXED,
            scope UNINDEXED,
            title,
            body,
            relative_path,
            status UNINDEXED,
            tokenize = 'trigram case_sensitive 0 remove_diacritics 1'
        );

        CREATE TABLE search_index_runs (
            id TEXT PRIMARY KEY NOT NULL,
            status TEXT NOT NULL,
            indexed_count INTEGER NOT NULL DEFAULT 0,
            failed_count INTEGER NOT NULL DEFAULT 0,
            started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            finished_at TEXT
        ) STRICT;

        CREATE TABLE search_index_jobs (
            id TEXT PRIMARY KEY NOT NULL,
            status TEXT NOT NULL,
            attempt_count INTEGER NOT NULL DEFAULT 0,
            last_error_code TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        ) STRICT;
    "#,
    ),
    (
        5,
        "book_detail_metadata",
        r#"
        CREATE TABLE book_user_state (
            book_id TEXT PRIMARY KEY NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            reading_status TEXT NOT NULL DEFAULT 'unread',
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        ) STRICT;

        CREATE TABLE book_tags (
            book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            tag TEXT NOT NULL,
            PRIMARY KEY(book_id, tag)
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

    fn upsert_note_projection(
        transaction: &rusqlite::Transaction<'_>,
        note: &NoteProjection,
    ) -> Result<NoteId, NotesError> {
        let existing_id: Option<String> = transaction
            .query_row(
                "SELECT id FROM notes WHERE path_key = ?1",
                [note.path_key.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let note_id = existing_id
            .as_deref()
            .map(NoteId::parse)
            .transpose()
            .map_err(|_| NotesError::RepositoryFailed)?
            .unwrap_or_else(NoteId::new);
        transaction
            .execute(
                "INSERT INTO notes
                 (id, relative_path, path_key, title, fingerprint, status,
                  size_bytes, modified_at_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'available', ?6, ?7)
                 ON CONFLICT(path_key) DO UPDATE SET
                   relative_path = excluded.relative_path,
                   title = excluded.title,
                   fingerprint = excluded.fingerprint,
                   status = 'available',
                   size_bytes = excluded.size_bytes,
                   modified_at_ms = excluded.modified_at_ms,
                   updated_at = CURRENT_TIMESTAMP",
                params![
                    note_id.to_string(),
                    note.relative_path.as_str(),
                    note.path_key,
                    note.title,
                    note.fingerprint,
                    i64::try_from(note.size_bytes).unwrap_or(i64::MAX),
                    note.modified_at_ms
                ],
            )
            .map_err(|_| NotesError::RepositoryFailed)?;

        for table in [
            "book_note_links",
            "note_headings",
            "note_tags",
            "note_links",
        ] {
            transaction
                .execute(
                    &format!(
                        "DELETE FROM {table} WHERE {}",
                        if table == "book_note_links"
                            || table == "note_headings"
                            || table == "note_tags"
                        {
                            "note_id = ?1"
                        } else {
                            "source_note_id = ?1"
                        }
                    ),
                    [note_id.to_string()],
                )
                .map_err(|_| NotesError::RepositoryFailed)?;
        }
        if let Some(book_path) = &note.book_relative_path {
            let path_key = if cfg!(target_os = "windows") {
                book_path.as_str().to_lowercase()
            } else {
                book_path.as_str().to_owned()
            };
            if let Some(book_id) = transaction
                .query_row(
                    "SELECT id FROM books WHERE path_key = ?1 LIMIT 1",
                    [path_key],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|_| NotesError::RepositoryFailed)?
            {
                transaction
                    .execute(
                        "INSERT INTO book_note_links (note_id, book_id)
                         VALUES (?1, ?2)",
                        params![note_id.to_string(), book_id],
                    )
                    .map_err(|_| NotesError::RepositoryFailed)?;
            }
        }
        for (index, heading) in note.headings.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO note_headings
                     (note_id, heading_index, level, text) VALUES (?1, ?2, ?3, ?4)",
                    params![
                        note_id.to_string(),
                        i64::try_from(index).unwrap_or(i64::MAX),
                        i64::from(heading.level),
                        heading.text
                    ],
                )
                .map_err(|_| NotesError::RepositoryFailed)?;
        }
        for tag in &note.tags {
            transaction
                .execute(
                    "INSERT INTO note_tags (note_id, tag) VALUES (?1, ?2)",
                    params![note_id.to_string(), tag],
                )
                .map_err(|_| NotesError::RepositoryFailed)?;
        }
        for (index, link) in note.links.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO note_links
                     (source_note_id, link_index, target_ref, link_text)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        note_id.to_string(),
                        i64::try_from(index).unwrap_or(i64::MAX),
                        link.target_ref,
                        link.link_text
                    ],
                )
                .map_err(|_| NotesError::RepositoryFailed)?;
        }
        Ok(note_id)
    }

    fn resolve_note_links(transaction: &rusqlite::Transaction<'_>) -> Result<(), NotesError> {
        transaction
            .execute(
                "UPDATE note_links
                 SET resolved_note_id = (
                   SELECT notes.id FROM notes
                   WHERE notes.status = 'available'
                     AND (
                       lower(notes.title) = lower(note_links.target_ref)
                       OR lower(notes.relative_path) = lower(note_links.target_ref)
                       OR lower(substr(notes.relative_path, 1, length(notes.relative_path) - 3))
                          = lower(replace(note_links.target_ref, '.md', ''))
                     )
                   LIMIT 1
                 )",
                [],
            )
            .map_err(|_| NotesError::RepositoryFailed)?;
        Ok(())
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
                    "UPDATE books SET thumbnail_cache_path = ?1, thumbnail_status = 'ready',
                     status = 'available', page_count = COALESCE(?2, page_count) WHERE id = ?3",
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
                 FROM books
                 ORDER BY relative_path COLLATE NOCASE, title COLLATE NOCASE",
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
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::CatalogFailed)?;
        let referenced_covers = {
            let mut statement = connection
                .prepare(
                    "SELECT id, thumbnail_cache_path
                     FROM books
                     WHERE status IN ('available', 'unavailable')
                       AND thumbnail_cache_path IS NOT NULL",
                )
                .map_err(|_| LibraryError::CatalogFailed)?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|_| LibraryError::CatalogFailed)?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|_| LibraryError::CatalogFailed)?
        };
        let canonical_cache_root = self.cache_root.canonicalize().ok();
        let invalid_book_ids = referenced_covers
            .into_iter()
            .filter_map(|(book_id, cache_relative_path)| {
                let valid = RelativePath::new(&cache_relative_path)
                    .ok()
                    .map(|relative| self.cache_root.join(relative.as_str()))
                    .and_then(|path| path.canonicalize().ok())
                    .is_some_and(|path| {
                        canonical_cache_root
                            .as_ref()
                            .is_some_and(|root| path.starts_with(root))
                            && image::open(path).is_ok()
                    });
                (!valid).then_some(book_id)
            })
            .collect::<Vec<_>>();

        let transaction = connection
            .transaction()
            .map_err(|_| LibraryError::CatalogFailed)?;
        for book_id in invalid_book_ids {
            transaction
                .execute(
                    "UPDATE books
                     SET thumbnail_cache_path = NULL, thumbnail_status = 'pending'
                     WHERE id = ?1",
                    [book_id],
                )
                .map_err(|_| LibraryError::CatalogFailed)?;
        }
        transaction
            .execute(
                "UPDATE books SET thumbnail_status = 'pending'
                 WHERE status IN ('available', 'unavailable')
                   AND thumbnail_cache_path IS NULL",
                [],
            )
            .map_err(|_| LibraryError::CatalogFailed)?;
        transaction
            .commit()
            .map_err(|_| LibraryError::CatalogFailed)?;
        Ok(())
    }

    fn recover_thumbnails(&self) -> Result<u64, LibraryError> {
        if !self.cache_root.is_dir() {
            return Ok(0);
        }
        let canonical_cache_root = self
            .cache_root
            .canonicalize()
            .map_err(|_| LibraryError::CatalogFailed)?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| LibraryError::CatalogFailed)?;
        let candidates = {
            let mut statement = connection
                .prepare(
                    "SELECT books.id, thumbnails.cache_relative_path
                     FROM books
                     JOIN thumbnails ON thumbnails.book_id = books.id
                     WHERE books.thumbnail_cache_path IS NULL
                       AND thumbnails.cache_relative_path <> ''",
                )
                .map_err(|_| LibraryError::CatalogFailed)?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|_| LibraryError::CatalogFailed)?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|_| LibraryError::CatalogFailed)?
        };

        let recoverable = candidates
            .into_iter()
            .filter(|(_, cache_relative_path)| {
                let Ok(relative) = RelativePath::new(cache_relative_path) else {
                    return false;
                };
                let Ok(resolved) = self.cache_root.join(relative.as_str()).canonicalize() else {
                    return false;
                };
                resolved.starts_with(&canonical_cache_root) && image::open(resolved).is_ok()
            })
            .collect::<Vec<_>>();

        let transaction = connection
            .transaction()
            .map_err(|_| LibraryError::CatalogFailed)?;
        let mut recovered = 0_u64;
        for (book_id, cache_relative_path) in recoverable {
            let changed = transaction
                .execute(
                    "UPDATE books
                     SET thumbnail_cache_path = ?1, thumbnail_status = 'ready'
                     WHERE id = ?2 AND thumbnail_cache_path IS NULL",
                    params![cache_relative_path, book_id],
                )
                .map_err(|_| LibraryError::CatalogFailed)?;
            if changed == 0 {
                continue;
            }
            transaction
                .execute(
                    "UPDATE thumbnails
                     SET status = 'ready', error_code = NULL
                     WHERE book_id = ?1",
                    [book_id],
                )
                .map_err(|_| LibraryError::CatalogFailed)?;
            recovered += 1;
        }
        transaction
            .commit()
            .map_err(|_| LibraryError::CatalogFailed)?;
        Ok(recovered)
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

impl BookLocationRepository for SqliteDatabase {
    fn book_source_location(
        &self,
        book_id: BookId,
    ) -> Result<Option<BookSourceLocation>, SourceLocationError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SourceLocationError::RepositoryFailed)?;
        let source: Option<(String, String, String, String)> = connection
            .query_row(
                "SELECT configured_libraries.root_path, books.kind,
                        books.relative_path, books.status
                 FROM books
                 JOIN configured_libraries
                   ON configured_libraries.id = books.library_id
                 WHERE books.id = ?1",
                [book_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|_| SourceLocationError::RepositoryFailed)?;

        source
            .map(|(root, kind, relative_path, status)| {
                let kind = match kind.as_str() {
                    "pdf_file" => BookKind::PdfFile,
                    "image_folder" => BookKind::ImageFolder,
                    _ => return Err(SourceLocationError::RepositoryFailed),
                };
                let relative_path = RelativePath::new(relative_path)
                    .map_err(|_| SourceLocationError::RepositoryFailed)?;
                Ok(BookSourceLocation {
                    library_root: PathBuf::from(root),
                    kind,
                    relative_path,
                    status,
                })
            })
            .transpose()
    }
}

impl BookMetadataRepository for SqliteDatabase {
    fn update_display_title(
        &self,
        book_id: BookId,
        title: &str,
    ) -> Result<bool, BookMetadataError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| BookMetadataError::RepositoryFailed)?;
        let updated = connection
            .execute(
                "UPDATE books
                 SET title = ?1, title_source = 'user', updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?2",
                params![title, book_id.to_string()],
            )
            .map_err(|_| BookMetadataError::RepositoryFailed)?;
        Ok(updated == 1)
    }
}

impl BookDetailRepository for SqliteDatabase {
    fn book_detail(&self, book_id: BookId) -> Result<Option<BookDetailRecord>, BookDetailError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let base: Option<BookDetailRow> = connection
            .query_row(
                "SELECT books.title, books.kind, books.relative_path, books.status,
                        books.page_count, books.size_bytes, books.modified_at_ms,
                        books.thumbnail_cache_path, books.thumbnail_status,
                        COALESCE(book_user_state.reading_status, 'unread')
                 FROM books
                 LEFT JOIN book_user_state ON book_user_state.book_id = books.id
                 WHERE books.id = ?1",
                [book_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let Some((
            title,
            kind,
            relative_path,
            status,
            page_count,
            size_bytes,
            modified_at_ms,
            thumbnail_cache_path,
            thumbnail_status,
            reading_status,
        )) = base
        else {
            return Ok(None);
        };
        let mut tag_statement = connection
            .prepare("SELECT tag FROM book_tags WHERE book_id = ?1 ORDER BY tag COLLATE NOCASE")
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let tags = tag_statement
            .query_map([book_id.to_string()], |row| row.get(0))
            .map_err(|_| BookDetailError::RepositoryFailed)?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let mut note_statement = connection
            .prepare(
                "SELECT notes.id, notes.title
                 FROM book_note_links
                 JOIN notes ON notes.id = book_note_links.note_id
                 WHERE book_note_links.book_id = ?1 AND notes.status = 'available'
                 ORDER BY notes.title COLLATE NOCASE",
            )
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let notes = note_statement
            .query_map([book_id.to_string()], |row| {
                Ok(LinkedBookNote {
                    id: row.get(0)?,
                    title: row.get(1)?,
                })
            })
            .map_err(|_| BookDetailError::RepositoryFailed)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        Ok(Some(BookDetailRecord {
            id: book_id.to_string(),
            title,
            kind,
            relative_path,
            status,
            page_count: page_count.and_then(|value| u32::try_from(value).ok()),
            size_bytes: size_bytes.and_then(|value| u64::try_from(value).ok()),
            modified_at_ms,
            thumbnail_cache_path,
            thumbnail_status,
            reading_status,
            tags,
            notes,
        }))
    }

    fn update_book_detail(
        &self,
        book_id: BookId,
        reading_status: &str,
        tags: &[String],
    ) -> Result<bool, BookDetailError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let transaction = connection
            .transaction()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM books WHERE id = ?1)",
                [book_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        if !exists {
            return Ok(false);
        }
        transaction
            .execute(
                "INSERT INTO book_user_state (book_id, reading_status)
                 VALUES (?1, ?2)
                 ON CONFLICT(book_id) DO UPDATE SET
                   reading_status = excluded.reading_status,
                   updated_at = CURRENT_TIMESTAMP",
                params![book_id.to_string(), reading_status],
            )
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        transaction
            .execute(
                "DELETE FROM book_tags WHERE book_id = ?1",
                [book_id.to_string()],
            )
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        for tag in tags {
            transaction
                .execute(
                    "INSERT INTO book_tags (book_id, tag) VALUES (?1, ?2)",
                    params![book_id.to_string(), tag],
                )
                .map_err(|_| BookDetailError::RepositoryFailed)?;
        }
        transaction
            .commit()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        Ok(true)
    }

    fn book_thumbnail_target(
        &self,
        book_id: BookId,
    ) -> Result<Option<BookThumbnailTarget>, BookDetailError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let base: Option<BookThumbnailRow> = connection
            .query_row(
                "SELECT configured_libraries.root_path, books.kind, books.status,
                        books.relative_path, books.path_key, books.title, books.size_bytes,
                        books.modified_at_ms, books.page_count
                 FROM books
                 JOIN configured_libraries ON configured_libraries.id = books.library_id
                 WHERE books.id = ?1",
                [book_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let Some((
            root,
            kind,
            status,
            relative_path,
            path_key,
            title,
            size_bytes,
            modified_at_ms,
            page_count,
        )) = base
        else {
            return Ok(None);
        };
        let fingerprint: String = connection
            .query_row(
                "SELECT fingerprint FROM books WHERE id = ?1",
                [book_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let kind = match kind.as_str() {
            "pdf_file" => BookKind::PdfFile,
            "image_folder" => BookKind::ImageFolder,
            _ => return Err(BookDetailError::RepositoryFailed),
        };
        let status = match status.as_str() {
            "available" => BookStatus::Available,
            "unavailable" => BookStatus::Unavailable,
            "missing" => BookStatus::Missing,
            "unsupported" => BookStatus::Unsupported,
            "error" => BookStatus::Error,
            _ => return Err(BookDetailError::RepositoryFailed),
        };
        let mut page_statement = connection
            .prepare(
                "SELECT relative_path FROM image_pages
                 WHERE book_id = ?1 ORDER BY page_index",
            )
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let image_pages = page_statement
            .query_map([book_id.to_string()], |row| row.get::<_, String>(0))
            .map_err(|_| BookDetailError::RepositoryFailed)?
            .map(|value| {
                value
                    .map_err(|_| BookDetailError::RepositoryFailed)
                    .and_then(|path| {
                        RelativePath::new(path).map_err(|_| BookDetailError::RepositoryFailed)
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(BookThumbnailTarget {
            root: PathBuf::from(root),
            book: DiscoveredBook {
                kind,
                status,
                relative_path: RelativePath::new(relative_path)
                    .map_err(|_| BookDetailError::RepositoryFailed)?,
                path_key,
                title,
                fingerprint: ContentFingerprint::new(fingerprint)
                    .map_err(|_| BookDetailError::RepositoryFailed)?,
                size_bytes: size_bytes.and_then(|value| u64::try_from(value).ok()),
                modified_at_ms,
                page_count: page_count.and_then(|value| u32::try_from(value).ok()),
                image_pages,
            },
        }))
    }

    fn books_without_cover(&self) -> Result<Vec<BookId>, BookDetailError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let mut statement = connection
            .prepare(
                "SELECT id
                 FROM books
                 WHERE status IN ('available', 'unavailable')
                   AND thumbnail_cache_path IS NULL
                 ORDER BY relative_path COLLATE NOCASE, title COLLATE NOCASE",
            )
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| BookDetailError::RepositoryFailed)?;
        rows.map(|row| {
            row.map_err(|_| BookDetailError::RepositoryFailed)
                .and_then(|id| BookId::parse(&id).map_err(|_| BookDetailError::RepositoryFailed))
        })
        .collect()
    }
}

impl BookRelocationRepository for SqliteDatabase {
    fn relocation_source(
        &self,
        book_id: BookId,
    ) -> Result<Option<BookSourceLocation>, BookRelocationError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| BookRelocationError::RepositoryFailed)?;
        let source: Option<(String, String, String, String)> = connection
            .query_row(
                "SELECT configured_libraries.root_path, books.kind,
                        books.relative_path, books.status
                 FROM books
                 JOIN configured_libraries ON configured_libraries.id = books.library_id
                 WHERE books.id = ?1",
                [book_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|_| BookRelocationError::RepositoryFailed)?;
        source
            .map(|(root, kind, relative_path, status)| {
                let kind = match kind.as_str() {
                    "pdf_file" => BookKind::PdfFile,
                    "image_folder" => BookKind::ImageFolder,
                    _ => return Err(BookRelocationError::RepositoryFailed),
                };
                Ok(BookSourceLocation {
                    library_root: PathBuf::from(root),
                    kind,
                    relative_path: RelativePath::new(relative_path)
                        .map_err(|_| BookRelocationError::RepositoryFailed)?,
                    status,
                })
            })
            .transpose()
    }

    fn update_source_path(
        &self,
        book_id: BookId,
        relative_path: &RelativePath,
        path_key: &str,
    ) -> Result<(), BookRelocationError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| BookRelocationError::RepositoryFailed)?;
        let result = connection.execute(
            "UPDATE books
             SET relative_path = ?1, path_key = ?2, status = 'available',
                 thumbnail_status = 'pending', updated_at = CURRENT_TIMESTAMP
             WHERE id = ?3",
            params![relative_path.as_str(), path_key, book_id.to_string()],
        );
        match result {
            Ok(1) => Ok(()),
            Ok(_) => Err(BookRelocationError::BookNotFound),
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(BookRelocationError::PathConflict)
            }
            Err(_) => Err(BookRelocationError::RepositoryFailed),
        }
    }
}

impl NotesRepository for SqliteDatabase {
    fn save_notes_configuration(&self, root: &Path) -> Result<NotesConfiguration, NotesError> {
        let root_text = root.to_str().ok_or(NotesError::RootInvalid)?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| NotesError::RepositoryFailed)?;
        connection
            .execute(
                "INSERT INTO configured_notes_root (id, root_path)
                 VALUES (1, ?1)
                 ON CONFLICT(id) DO UPDATE SET
                   root_path = excluded.root_path,
                   updated_at = CURRENT_TIMESTAMP",
                [root_text],
            )
            .map_err(|_| NotesError::RepositoryFailed)?;
        Ok(NotesConfiguration {
            root: root.to_path_buf(),
            display_name: root
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("Notes")
                .to_owned(),
        })
    }

    fn notes_configuration(&self) -> Result<Option<NotesConfiguration>, NotesError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let root: Option<String> = connection
            .query_row(
                "SELECT root_path FROM configured_notes_root WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| NotesError::RepositoryFailed)?;
        Ok(root.map(|value| {
            let root = PathBuf::from(value);
            NotesConfiguration {
                display_name: root
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("Notes")
                    .to_owned(),
                root,
            }
        }))
    }

    fn reconcile_notes(
        &self,
        notes: &[NoteProjection],
        issues: u64,
    ) -> Result<NotesRefreshSummary, NotesError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let transaction = connection
            .transaction()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let mut added = 0;
        let mut updated = 0;
        let mut seen = std::collections::HashSet::new();
        for note in notes {
            seen.insert(note.path_key.clone());
            let previous: Option<(String, String)> = transaction
                .query_row(
                    "SELECT fingerprint, status FROM notes WHERE path_key = ?1",
                    [note.path_key.as_str()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(|_| NotesError::RepositoryFailed)?;
            match previous {
                None => added += 1,
                Some((fingerprint, status))
                    if fingerprint != note.fingerprint || status != "available" =>
                {
                    updated += 1;
                }
                Some(_) => {}
            }
            Self::upsert_note_projection(&transaction, note)?;
        }
        let known = {
            let mut statement = transaction
                .prepare("SELECT path_key FROM notes WHERE status <> 'missing'")
                .map_err(|_| NotesError::RepositoryFailed)?;
            statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|_| NotesError::RepositoryFailed)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| NotesError::RepositoryFailed)?
        };
        let mut missing = 0;
        for path_key in known {
            if !seen.contains(&path_key) {
                missing += transaction
                    .execute(
                        "UPDATE notes SET status = 'missing', updated_at = CURRENT_TIMESTAMP
                         WHERE path_key = ?1",
                        [path_key],
                    )
                    .map_err(|_| NotesError::RepositoryFailed)? as u64;
            }
        }
        Self::resolve_note_links(&transaction)?;
        transaction
            .commit()
            .map_err(|_| NotesError::RepositoryFailed)?;
        Ok(NotesRefreshSummary {
            discovered: notes.len() as u64,
            added,
            updated,
            missing,
            issues,
        })
    }

    fn upsert_note(&self, note: &NoteProjection) -> Result<NoteId, NotesError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let transaction = connection
            .transaction()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let note_id = Self::upsert_note_projection(&transaction, note)?;
        Self::resolve_note_links(&transaction)?;
        transaction
            .commit()
            .map_err(|_| NotesError::RepositoryFailed)?;
        Ok(note_id)
    }

    fn note_record(&self, note_id: NoteId) -> Result<Option<NoteRecord>, NotesError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let record: Option<(String, String)> = connection
            .query_row(
                "SELECT relative_path, status FROM notes WHERE id = ?1",
                [note_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|_| NotesError::RepositoryFailed)?;
        record
            .map(|(relative_path, status)| {
                Ok(NoteRecord {
                    relative_path: RelativePath::new(relative_path)
                        .map_err(|_| NotesError::RepositoryFailed)?,
                    status,
                })
            })
            .transpose()
    }

    fn book_relative_path(&self, book_id: BookId) -> Result<Option<RelativePath>, NotesError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let path: Option<String> = connection
            .query_row(
                "SELECT relative_path FROM books WHERE id = ?1",
                [book_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| NotesError::RepositoryFailed)?;
        path.map(|value| RelativePath::new(value).map_err(|_| NotesError::RepositoryFailed))
            .transpose()
    }

    fn list_notes(&self) -> Result<Vec<NoteListItem>, NotesError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let mut statement = connection
            .prepare(
                "SELECT notes.id, notes.title, notes.relative_path, notes.status,
                        books.id, books.title, notes.modified_at_ms
                 FROM notes
                 LEFT JOIN book_note_links ON book_note_links.note_id = notes.id
                 LEFT JOIN books ON books.id = book_note_links.book_id
                 ORDER BY notes.title COLLATE NOCASE, notes.relative_path",
            )
            .map_err(|_| NotesError::RepositoryFailed)?;
        statement
            .query_map([], |row| {
                Ok(NoteListItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    relative_path: row.get(2)?,
                    status: row.get(3)?,
                    book_id: row.get(4)?,
                    book_title: row.get(5)?,
                    modified_at_ms: row.get(6)?,
                })
            })
            .map_err(|_| NotesError::RepositoryFailed)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| NotesError::RepositoryFailed)
    }

    fn note_detail_projection(
        &self,
        note_id: NoteId,
        body: String,
    ) -> Result<Option<NoteDetail>, NotesError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let base: Option<(String, String, Option<String>, Option<String>)> = connection
            .query_row(
                "SELECT notes.title, notes.relative_path, books.id, books.title
                 FROM notes
                 LEFT JOIN book_note_links ON book_note_links.note_id = notes.id
                 LEFT JOIN books ON books.id = book_note_links.book_id
                 WHERE notes.id = ?1",
                [note_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|_| NotesError::RepositoryFailed)?;
        let Some((title, relative_path, book_id, book_title)) = base else {
            return Ok(None);
        };
        let mut statement = connection
            .prepare(
                "SELECT source.id, source.title, source.relative_path
                 FROM note_links
                 JOIN notes AS source ON source.id = note_links.source_note_id
                 WHERE note_links.resolved_note_id = ?1 AND source.status = 'available'
                 ORDER BY source.title COLLATE NOCASE",
            )
            .map_err(|_| NotesError::RepositoryFailed)?;
        let backlinks = statement
            .query_map([note_id.to_string()], |row| {
                Ok(NoteBacklink {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    relative_path: row.get(2)?,
                })
            })
            .map_err(|_| NotesError::RepositoryFailed)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| NotesError::RepositoryFailed)?;
        Ok(Some(NoteDetail {
            id: note_id.to_string(),
            title,
            relative_path,
            body,
            book_id,
            book_title,
            backlinks,
        }))
    }
}

impl SearchRepository for SqliteDatabase {
    fn enqueue_search_rebuild(&self) -> Result<(), SearchError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SearchError::IndexUnavailable)?;
        connection
            .execute(
                "INSERT INTO search_index_jobs (id, status)
                 VALUES ('full-rebuild', 'pending')
                 ON CONFLICT(id) DO UPDATE SET
                   status = 'pending', last_error_code = NULL,
                   updated_at = CURRENT_TIMESTAMP",
                [],
            )
            .map_err(|_| SearchError::IndexUnavailable)?;
        Ok(())
    }

    fn canonical_search_documents(
        &self,
    ) -> Result<(Vec<SearchDocument>, Option<PathBuf>), SearchError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SearchError::IndexUnavailable)?;
        let mut documents = Vec::new();
        {
            let mut statement = connection
                .prepare(
                    "SELECT books.id, books.title, books.relative_path, books.status,
                            COALESCE(book_user_state.reading_status, 'unread') || ' ' ||
                            COALESCE(GROUP_CONCAT(book_tags.tag, ' '), '')
                     FROM books
                     LEFT JOIN book_user_state ON book_user_state.book_id = books.id
                     LEFT JOIN book_tags ON book_tags.book_id = books.id
                     GROUP BY books.id
                     ORDER BY books.title, books.relative_path",
                )
                .map_err(|_| SearchError::IndexUnavailable)?;
            let rows = statement
                .query_map([], |row| {
                    Ok(SearchDocument {
                        source_kind: "book".to_owned(),
                        source_id: row.get(0)?,
                        scope: "books".to_owned(),
                        title: row.get(1)?,
                        body: row.get(4)?,
                        relative_path: row.get(2)?,
                        status: row.get(3)?,
                    })
                })
                .map_err(|_| SearchError::IndexUnavailable)?;
            documents.extend(
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|_| SearchError::IndexUnavailable)?,
            );
        }
        {
            let mut statement = connection
                .prepare(
                    "SELECT notes.id, notes.title, notes.relative_path, notes.status,
                            COALESCE(books.title, '')
                     FROM notes
                     LEFT JOIN book_note_links ON book_note_links.note_id = notes.id
                     LEFT JOIN books ON books.id = book_note_links.book_id
                     ORDER BY notes.title, notes.relative_path",
                )
                .map_err(|_| SearchError::IndexUnavailable)?;
            let rows = statement
                .query_map([], |row| {
                    Ok(SearchDocument {
                        source_kind: "note".to_owned(),
                        source_id: row.get(0)?,
                        scope: "notes".to_owned(),
                        title: row.get(1)?,
                        relative_path: row.get(2)?,
                        status: row.get(3)?,
                        body: row.get(4)?,
                    })
                })
                .map_err(|_| SearchError::IndexUnavailable)?;
            documents.extend(
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|_| SearchError::IndexUnavailable)?,
            );
        }
        {
            let mut statement = connection
                .prepare(
                    "SELECT notes.id, notes.title, notes.relative_path, notes.status,
                            note_headings.text
                     FROM note_headings
                     JOIN notes ON notes.id = note_headings.note_id",
                )
                .map_err(|_| SearchError::IndexUnavailable)?;
            let rows = statement
                .query_map([], |row| {
                    Ok(SearchDocument {
                        source_kind: "note".to_owned(),
                        source_id: row.get(0)?,
                        scope: "headings".to_owned(),
                        title: row.get(1)?,
                        relative_path: row.get(2)?,
                        status: row.get(3)?,
                        body: row.get(4)?,
                    })
                })
                .map_err(|_| SearchError::IndexUnavailable)?;
            documents.extend(
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|_| SearchError::IndexUnavailable)?,
            );
        }
        {
            let mut statement = connection
                .prepare(
                    "SELECT notes.id, notes.title, notes.relative_path, notes.status,
                            note_tags.tag
                     FROM note_tags
                     JOIN notes ON notes.id = note_tags.note_id",
                )
                .map_err(|_| SearchError::IndexUnavailable)?;
            let rows = statement
                .query_map([], |row| {
                    Ok(SearchDocument {
                        source_kind: "note".to_owned(),
                        source_id: row.get(0)?,
                        scope: "tags".to_owned(),
                        title: row.get(1)?,
                        relative_path: row.get(2)?,
                        status: row.get(3)?,
                        body: row.get(4)?,
                    })
                })
                .map_err(|_| SearchError::IndexUnavailable)?;
            documents.extend(
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|_| SearchError::IndexUnavailable)?,
            );
        }
        let notes_root = connection
            .query_row(
                "SELECT root_path FROM configured_notes_root WHERE id = 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| SearchError::IndexUnavailable)?
            .map(PathBuf::from);
        Ok((documents, notes_root))
    }

    fn replace_search_documents(
        &self,
        documents: &[SearchDocument],
        failed: u64,
    ) -> Result<SearchRebuildSummary, SearchError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| SearchError::RebuildFailed)?;
        let transaction = connection
            .transaction()
            .map_err(|_| SearchError::RebuildFailed)?;
        let run_id = uuid::Uuid::new_v4().to_string();
        transaction
            .execute(
                "INSERT INTO search_index_runs (id, status) VALUES (?1, 'running')",
                [run_id.as_str()],
            )
            .map_err(|_| SearchError::RebuildFailed)?;
        transaction
            .execute(
                "INSERT INTO search_index_jobs (id, status, attempt_count)
                 VALUES ('full-rebuild', 'running', 1)
                 ON CONFLICT(id) DO UPDATE SET
                   status = 'running', attempt_count = attempt_count + 1,
                   updated_at = CURRENT_TIMESTAMP",
                [],
            )
            .map_err(|_| SearchError::RebuildFailed)?;
        transaction
            .execute("DELETE FROM search_documents_fts", [])
            .map_err(|_| SearchError::RebuildFailed)?;
        {
            let mut statement = transaction
                .prepare(
                    "INSERT INTO search_documents_fts
                     (source_kind, source_id, scope, title, body, relative_path, status)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                )
                .map_err(|_| SearchError::RebuildFailed)?;
            for document in documents {
                statement
                    .execute(params![
                        document.source_kind,
                        document.source_id,
                        document.scope,
                        document.title,
                        document.body,
                        document.relative_path,
                        document.status,
                    ])
                    .map_err(|_| SearchError::RebuildFailed)?;
            }
        }
        transaction
            .execute(
                "UPDATE search_index_runs
                 SET status = 'complete', indexed_count = ?1, failed_count = ?2,
                     finished_at = CURRENT_TIMESTAMP
                 WHERE id = ?3",
                params![
                    i64::try_from(documents.len()).unwrap_or(i64::MAX),
                    i64::try_from(failed).unwrap_or(i64::MAX),
                    run_id
                ],
            )
            .map_err(|_| SearchError::RebuildFailed)?;
        transaction
            .execute(
                "UPDATE search_index_jobs
                 SET status = 'complete', updated_at = CURRENT_TIMESTAMP
                 WHERE id = 'full-rebuild'",
                [],
            )
            .map_err(|_| SearchError::RebuildFailed)?;
        transaction
            .commit()
            .map_err(|_| SearchError::RebuildFailed)?;
        Ok(SearchRebuildSummary {
            indexed: documents.len() as u64,
            failed,
        })
    }

    fn search(
        &self,
        query: &str,
        scope: Option<&str>,
        limit: u32,
    ) -> Result<Vec<SearchResultItem>, SearchError> {
        let terms = query
            .split_whitespace()
            .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
            .collect::<Vec<_>>();
        if terms.is_empty() {
            return Err(SearchError::InvalidQuery);
        }
        let expression = terms.join(" AND ");
        let connection = self
            .connection
            .lock()
            .map_err(|_| SearchError::IndexUnavailable)?;
        let mut statement = connection
            .prepare(
                "SELECT source_kind, source_id, scope, title,
                        snippet(search_documents_fts, 4, '<mark>', '</mark>', ' … ', 24),
                        relative_path, status, bm25(search_documents_fts, 0.0, 0.0, 0.0, 8.0, 2.0, 1.0)
                 FROM search_documents_fts
                 WHERE search_documents_fts MATCH ?1
                   AND (?2 IS NULL OR scope = ?2)
                 ORDER BY bm25(search_documents_fts, 0.0, 0.0, 0.0, 8.0, 2.0, 1.0)
                 LIMIT ?3",
            )
            .map_err(|_| SearchError::IndexUnavailable)?;
        statement
            .query_map(params![expression, scope, i64::from(limit)], |row| {
                Ok(SearchResultItem {
                    source_kind: row.get(0)?,
                    source_id: row.get(1)?,
                    scope: row.get(2)?,
                    title: row.get(3)?,
                    snippet: row.get(4)?,
                    relative_path: row.get(5)?,
                    status: row.get(6)?,
                    rank: row.get(7)?,
                })
            })
            .map_err(|_| SearchError::IndexUnavailable)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| SearchError::IndexUnavailable)
    }

    fn search_diagnostics(&self) -> Result<SearchDiagnostics, SearchError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SearchError::IndexUnavailable)?;
        let documents: i64 = connection
            .query_row("SELECT COUNT(*) FROM search_documents_fts", [], |row| {
                row.get(0)
            })
            .map_err(|_| SearchError::IndexUnavailable)?;
        let (failed_jobs, last_rebuild_at): (i64, Option<String>) = connection
            .query_row(
                "SELECT COALESCE(SUM(failed_count), 0), MAX(finished_at)
                 FROM search_index_runs",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| SearchError::IndexUnavailable)?;
        Ok(SearchDiagnostics {
            documents: u64::try_from(documents).unwrap_or_default(),
            failed_jobs: u64::try_from(failed_jobs).unwrap_or_default(),
            last_rebuild_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::{
            CancellationToken, DiscoveredBook, ExternalPathOpener, LibraryRepository, NotesError,
            NotesWorkspace, ReconcileCatalog, ScanResult, SearchLibrary, ThumbnailGenerator,
            ThumbnailOutcome,
        },
        domain::{BookId, BookKind, ContentFingerprint, NoteId, RelativePath},
        infrastructure::{FilesystemScanner, MarkdownNotesStore},
    };
    use std::{
        sync::{Arc, Barrier},
        thread,
    };
    use tempfile::TempDir;

    struct SkipRealLibraryThumbnails;

    struct NoopExternalOpener;

    impl ExternalPathOpener for NoopExternalOpener {
        fn open_path(&self, _path: &Path) -> Result<(), NotesError> {
            Ok(())
        }
    }

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
        let book_id = first.thumbnail_targets[0].0;
        let stored_source = database.book_source_location(book_id).unwrap().unwrap();
        assert_eq!(stored_source.kind, BookKind::PdfFile);
        assert_eq!(stored_source.relative_path.as_str(), "Shelf/Book.pdf");
        assert_eq!(stored_source.status, "available");

        let thumbnail_relative_path = "thumbnails/known-good.png";
        let thumbnail_path = database.cache_root.join(thumbnail_relative_path);
        fs::create_dir_all(thumbnail_path.parent().unwrap()).unwrap();
        fs::write(&thumbnail_path, b"not an image").unwrap();
        {
            let connection = database.connection.lock().unwrap();
            connection
                .execute(
                    "INSERT INTO thumbnails
                     (book_id, cache_relative_path, width, height, format,
                      source_fingerprint, status, error_code)
                     VALUES (?1, ?2, 2, 3, 'png', 'pdf:10:20', 'error',
                             'thumbnail_failed')",
                    params![book_id.to_string(), thumbnail_relative_path],
                )
                .unwrap();
        }
        assert_eq!(database.recover_thumbnails().unwrap(), 0);
        image::DynamicImage::new_rgb8(2, 3)
            .save(&thumbnail_path)
            .unwrap();
        assert_eq!(database.recover_thumbnails().unwrap(), 1);
        assert_eq!(database.recover_thumbnails().unwrap(), 0);

        let recovered_cover = database.list_books().unwrap();
        assert_eq!(
            recovered_cover[0].thumbnail_cache_path.as_deref(),
            Some(thumbnail_relative_path)
        );
        assert_eq!(recovered_cover[0].thumbnail_status, "ready");

        database.invalidate_thumbnails().unwrap();
        let ready_cover = database.list_books().unwrap();
        assert_eq!(
            ready_cover[0].thumbnail_cache_path.as_deref(),
            Some(thumbnail_relative_path)
        );
        assert_eq!(ready_cover[0].thumbnail_status, "ready");
        {
            let connection = database.connection.lock().unwrap();
            connection
                .execute(
                    "UPDATE books SET thumbnail_status = 'error' WHERE id = ?1",
                    [book_id.to_string()],
                )
                .unwrap();
        }
        database.invalidate_thumbnails().unwrap();
        let pending_cover = database.list_books().unwrap();
        assert_eq!(
            pending_cover[0].thumbnail_cache_path.as_deref(),
            Some(thumbnail_relative_path)
        );
        assert_eq!(pending_cover[0].thumbnail_status, "error");
        database
            .save_thumbnail_failure(book_id, "thumbnail_failed")
            .unwrap();
        let failed_cover = database.list_books().unwrap();
        assert_eq!(
            failed_cover[0].thumbnail_cache_path.as_deref(),
            Some(thumbnail_relative_path)
        );
        assert_eq!(failed_cover[0].thumbnail_status, "error");

        assert!(database.update_display_title(book_id, "My title").unwrap());
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
        database
            .update_book_detail(
                book_id,
                "reading",
                &["psychology".to_owned(), "心理学".to_owned()],
            )
            .unwrap();
        let detail = database.book_detail(book_id).unwrap().unwrap();
        assert_eq!(detail.reading_status, "reading");
        assert_eq!(detail.tags, ["psychology", "心理学"]);
        let markdown = MarkdownNotesStore::new();
        let search = SearchLibrary::new(&database, &markdown);
        search.rebuild().unwrap();
        assert_eq!(search.execute("心理学", Some("books")).unwrap().len(), 1);

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
        let preserved = database.book_detail(book_id).unwrap().unwrap();
        assert_eq!(preserved.reading_status, "reading");
        assert_eq!(preserved.tags, ["psychology", "心理学"]);
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
                    books: vec![book.clone()],
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

        let repair_targets = database.books_without_cover().unwrap();
        assert_eq!(repair_targets.len(), 1);
        let book_id = repair_targets[0];
        let existing_cover = database.cache_root.join("thumbnails/existing.png");
        fs::create_dir_all(existing_cover.parent().unwrap()).unwrap();
        image::DynamicImage::new_rgb8(2, 3)
            .save(&existing_cover)
            .unwrap();
        database
            .save_thumbnail(
                book_id,
                &ThumbnailOutcome {
                    cache_relative_path: "thumbnails/existing.png".to_owned(),
                    width: 2,
                    height: 3,
                    format: "png",
                    source_fingerprint: book.fingerprint.as_str().to_owned(),
                    page_count: None,
                },
            )
            .unwrap();
        database
            .save_thumbnail_failure(book_id, "thumbnail_failed")
            .unwrap();

        assert!(
            database.books_without_cover().unwrap().is_empty(),
            "Repair must preserve an existing last-known-good cover"
        );
    }

    #[test]
    fn catalog_books_are_ordered_by_relative_path_before_title() {
        let app_data = TempDir::new().unwrap();
        let library = TempDir::new().unwrap();
        let database = SqliteDatabase::initialize(app_data.path()).unwrap();
        let configuration = database
            .save_configuration(library.path(), "Library")
            .unwrap();
        let discovered = [
            ("Z shelf/Alpha.pdf", "Alpha"),
            ("A shelf/Zebra.pdf", "Zebra"),
        ]
        .into_iter()
        .map(|(relative_path, title)| {
            let relative_path = RelativePath::new(relative_path).unwrap();
            DiscoveredBook {
                kind: BookKind::PdfFile,
                status: BookStatus::Available,
                path_key: if cfg!(target_os = "windows") {
                    relative_path.as_str().to_lowercase()
                } else {
                    relative_path.as_str().to_owned()
                },
                title: title.to_owned(),
                fingerprint: ContentFingerprint::new(format!("pdf:{relative_path}:1")).unwrap(),
                relative_path,
                size_bytes: Some(1),
                modified_at_ms: Some(1),
                page_count: None,
                image_pages: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
        let job = database
            .start_scan(configuration.id, ScanReason::Initial)
            .unwrap();
        database
            .reconcile(
                configuration.id,
                &job,
                &ScanResult {
                    books: discovered,
                    issues: Vec::new(),
                    cancelled: false,
                },
            )
            .unwrap();

        let books = database.list_books().unwrap();
        assert_eq!(books[0].relative_path, "A shelf/Zebra.pdf");
        assert_eq!(books[1].relative_path, "Z shelf/Alpha.pdf");
    }

    #[test]
    fn markdown_notes_round_trip_and_resolve_backlinks() {
        let app_data = TempDir::new().unwrap();
        let notes_root = TempDir::new().unwrap();
        let database = SqliteDatabase::initialize(app_data.path()).unwrap();
        let markdown = MarkdownNotesStore::new();
        let opener = NoopExternalOpener;
        let workspace = NotesWorkspace::new(&database, &markdown, &opener);

        workspace.configure(notes_root.path()).unwrap();
        let first = workspace.create("First note", None).unwrap();
        let second = workspace.create("Second note", None).unwrap();
        let second_id = NoteId::parse(&second.id).unwrap();
        workspace
            .save(second_id, "# Second note\n\nLinks to [[First note]].\n")
            .unwrap();

        let refreshed = workspace.refresh().unwrap();
        assert_eq!(refreshed.discovered, 2);
        assert_eq!(refreshed.missing, 0);
        let detail = workspace.read(NoteId::parse(&first.id).unwrap()).unwrap();
        assert_eq!(detail.backlinks.len(), 1);
        assert_eq!(detail.backlinks[0].title, "Second note");
        assert_eq!(
            fs::read_to_string(notes_root.path().join(second.relative_path)).unwrap(),
            "# Second note\n\nLinks to [[First note]].\n"
        );

        workspace
            .save(
                NoteId::parse(&first.id).unwrap(),
                "# First note\n\n## 脳の自動操縦\n\n重要な内容です。 #心理学\n",
            )
            .unwrap();
        let search = SearchLibrary::new(&database, &markdown);
        let rebuilt = search.rebuild().unwrap();
        assert!(rebuilt.indexed >= 4);
        assert_eq!(rebuilt.failed, 0);
        assert_eq!(search.execute("自動操縦", Some("notes")).unwrap().len(), 1);
        assert_eq!(
            search.execute("自動操縦", Some("headings")).unwrap().len(),
            1
        );
        assert_eq!(search.execute("心理学", Some("tags")).unwrap().len(), 1);
        assert!(search.diagnostics().unwrap().documents >= 4);
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
