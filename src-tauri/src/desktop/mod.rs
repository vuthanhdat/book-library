//! Thin desktop composition and Tauri boundary.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    application::{
        ApplicationError, BookDetailError, BookDetailRecord, BookMetadataError,
        BookRelocationError, CancellationToken, ConfigureLibrary, ForceBookCover,
        GetApplicationStatus, GetBookDetail, LibraryError, LibraryRepository, NoteDetail,
        NoteListItem, NotesError, NotesRefreshSummary, NotesRepository, NotesWorkspace,
        OpenBookLocation, ReconcileCatalog, RelinkMissingBook, ScanProgress, ScanReason,
        SearchDiagnostics, SearchError, SearchLibrary, SearchRebuildSummary, SearchRepository,
        SearchResultItem, SourceLocationError, UpdateBookDetail, UpdateBookDisplayTitle,
    },
    domain::{BookId, NoteId},
    infrastructure::{
        FilesystemScanner, LoggingGuard, MarkdownNotesStore, SqliteDatabase, SystemFileManager,
        ThumbnailService, initialize_logging,
    },
};

struct BackendState {
    database: Arc<SqliteDatabase>,
    scanner: Arc<FilesystemScanner>,
    thumbnails: Arc<ThumbnailService>,
    file_manager: Arc<SystemFileManager>,
    markdown_notes: Arc<MarkdownNotesStore>,
    active_scan: Arc<Mutex<Option<CancellationToken>>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplicationStatusResponse {
    database_healthy: bool,
    library_configured: bool,
    platform: PlatformResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlatformResponse {
    os: &'static str,
    architecture: &'static str,
    supported: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryConfigurationResponse {
    display_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanProgressResponse {
    visited_entries: u64,
    discovered_books: u64,
    current_relative_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanSummaryResponse {
    discovered: u64,
    added: u64,
    updated: u64,
    missing: u64,
    issues: u64,
    thumbnails_generated: u64,
    thumbnail_failures: u64,
    cancelled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BookResponse {
    id: String,
    title: String,
    kind: String,
    relative_path: String,
    status: String,
    page_count: Option<u32>,
    size_bytes: Option<u64>,
    modified_at_ms: Option<i64>,
    thumbnail_data_url: Option<String>,
    thumbnail_status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdatedBookTitleResponse {
    title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkedBookNoteResponse {
    id: String,
    title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BookDetailResponse {
    id: String,
    title: String,
    kind: String,
    relative_path: String,
    status: String,
    page_count: Option<u32>,
    size_bytes: Option<u64>,
    modified_at_ms: Option<i64>,
    thumbnail_data_url: Option<String>,
    thumbnail_status: String,
    reading_status: String,
    tags: Vec<String>,
    notes: Vec<LinkedBookNoteResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotesConfigurationResponse {
    display_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NoteListResponse {
    id: String,
    title: String,
    relative_path: String,
    status: String,
    book_id: Option<String>,
    book_title: Option<String>,
    modified_at_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NoteBacklinkResponse {
    id: String,
    title: String,
    relative_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NoteDetailResponse {
    id: String,
    title: String,
    relative_path: String,
    body: String,
    book_id: Option<String>,
    book_title: Option<String>,
    backlinks: Vec<NoteBacklinkResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotesRefreshResponse {
    discovered: u64,
    added: u64,
    updated: u64,
    missing: u64,
    issues: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResultResponse {
    source_kind: String,
    source_id: String,
    scope: String,
    title: String,
    snippet: String,
    relative_path: String,
    status: String,
    rank: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchRebuildResponse {
    indexed: u64,
    failed: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchDiagnosticsResponse {
    documents: u64,
    failed_jobs: u64,
    last_rebuild_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopError {
    code: &'static str,
    message: &'static str,
}

impl From<ApplicationError> for DesktopError {
    fn from(error: ApplicationError) -> Self {
        match error {
            ApplicationError::DatabaseUnavailable => Self {
                code: "database_unavailable",
                message: "The local database is unavailable.",
            },
            ApplicationError::ConfigurationUnavailable => Self {
                code: "configuration_unavailable",
                message: "Library configuration could not be read.",
            },
        }
    }
}

impl From<LibraryError> for DesktopError {
    fn from(error: LibraryError) -> Self {
        match error {
            LibraryError::RootMissing => Self {
                code: "library_root_missing",
                message: "The selected library folder does not exist.",
            },
            LibraryError::RootUnreadable => Self {
                code: "library_root_unreadable",
                message: "The selected library folder is not readable.",
            },
            LibraryError::RootInvalid => Self {
                code: "library_root_invalid",
                message: "The selected library folder could not be resolved safely.",
            },
            LibraryError::NotConfigured => Self {
                code: "library_not_configured",
                message: "Choose a library folder before scanning.",
            },
            LibraryError::ThumbnailFailed => Self {
                code: "thumbnail_failed",
                message: "A thumbnail could not be generated.",
            },
            LibraryError::ConfigurationFailed
            | LibraryError::ScanFailed
            | LibraryError::CatalogFailed => Self {
                code: "library_operation_failed",
                message: "The library operation could not be completed.",
            },
        }
    }
}

impl From<SourceLocationError> for DesktopError {
    fn from(error: SourceLocationError) -> Self {
        match error {
            SourceLocationError::InvalidBookId => Self {
                code: "invalid_book_id",
                message: "The selected book identifier is invalid.",
            },
            SourceLocationError::BookNotFound => Self {
                code: "book_not_found",
                message: "The selected book is no longer in the catalog.",
            },
            SourceLocationError::SourceUnavailable => Self {
                code: "book_source_unavailable",
                message: "The selected book source is currently unavailable.",
            },
            SourceLocationError::InvalidSourcePath => Self {
                code: "book_source_invalid",
                message: "The selected book source path is not safe to open.",
            },
            SourceLocationError::RepositoryFailed => Self {
                code: "catalog_read_failed",
                message: "The selected catalog record could not be read.",
            },
            SourceLocationError::LaunchFailed => Self {
                code: "file_manager_launch_failed",
                message: "The operating system file manager could not be opened.",
            },
        }
    }
}

impl From<BookMetadataError> for DesktopError {
    fn from(error: BookMetadataError) -> Self {
        match error {
            BookMetadataError::InvalidBookId => Self {
                code: "invalid_book_id",
                message: "The selected book identifier is invalid.",
            },
            BookMetadataError::BookNotFound => Self {
                code: "book_not_found",
                message: "The selected book is no longer in the catalog.",
            },
            BookMetadataError::InvalidTitle => Self {
                code: "invalid_book_title",
                message: "Enter a title between 1 and 512 characters without line breaks.",
            },
            BookMetadataError::RepositoryFailed => Self {
                code: "book_metadata_save_failed",
                message: "The book title could not be saved.",
            },
        }
    }
}

impl From<BookDetailError> for DesktopError {
    fn from(error: BookDetailError) -> Self {
        match error {
            BookDetailError::BookNotFound => Self {
                code: "book_not_found",
                message: "The selected book is no longer in the catalog.",
            },
            BookDetailError::InvalidReadingStatus => Self {
                code: "reading_status_invalid",
                message: "Choose unread, reading, or read.",
            },
            BookDetailError::InvalidTags => Self {
                code: "book_tags_invalid",
                message: "Use up to 100 tags without spaces, each at most 64 characters.",
            },
            BookDetailError::SourceUnavailable => Self {
                code: "cover_source_unavailable",
                message: "The source must be available before generating a cover.",
            },
            BookDetailError::CoverFailed => Self {
                code: "cover_generation_failed",
                message: "The cover could not be generated within 30 seconds.",
            },
            BookDetailError::RepositoryFailed => Self {
                code: "book_detail_failed",
                message: "The book details could not be saved.",
            },
        }
    }
}

fn book_detail_response(detail: BookDetailRecord, database: &SqliteDatabase) -> BookDetailResponse {
    let thumbnail_data_url = detail.thumbnail_cache_path.as_deref().and_then(|path| {
        database
            .thumbnail_bytes(path)
            .ok()
            .map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
    });
    BookDetailResponse {
        id: detail.id,
        title: detail.title,
        kind: detail.kind,
        relative_path: detail.relative_path,
        status: detail.status,
        page_count: detail.page_count,
        size_bytes: detail.size_bytes,
        modified_at_ms: detail.modified_at_ms,
        thumbnail_data_url,
        thumbnail_status: detail.thumbnail_status,
        reading_status: detail.reading_status,
        tags: detail.tags,
        notes: detail
            .notes
            .into_iter()
            .map(|note| LinkedBookNoteResponse {
                id: note.id,
                title: note.title,
            })
            .collect(),
    }
}

impl From<NotesError> for DesktopError {
    fn from(error: NotesError) -> Self {
        match error {
            NotesError::RootUnavailable => Self {
                code: "notes_root_unavailable",
                message: "The selected notes folder is unavailable or unreadable.",
            },
            NotesError::RootInvalid | NotesError::InvalidNotePath => Self {
                code: "notes_path_invalid",
                message: "The selected notes path is not safe to use.",
            },
            NotesError::NotConfigured => Self {
                code: "notes_not_configured",
                message: "Choose a notes folder before using notes.",
            },
            NotesError::NoteNotFound => Self {
                code: "note_not_found",
                message: "The selected note is missing or no longer indexed.",
            },
            NotesError::InvalidTitle => Self {
                code: "invalid_note_title",
                message: "Enter a note title between 1 and 200 characters.",
            },
            NotesError::InvalidBody => Self {
                code: "invalid_note_body",
                message: "The note is too large or contains invalid content.",
            },
            NotesError::BookNotFound => Self {
                code: "book_not_found",
                message: "The selected book is no longer in the catalog.",
            },
            NotesError::ReadFailed => Self {
                code: "note_read_failed",
                message: "The Markdown note could not be read.",
            },
            NotesError::WriteFailed => Self {
                code: "note_write_failed",
                message: "The Markdown note could not be saved.",
            },
            NotesError::RepositoryFailed => Self {
                code: "notes_projection_failed",
                message: "The local notes projection could not be updated.",
            },
            NotesError::LaunchFailed => Self {
                code: "note_launch_failed",
                message: "The external notes application could not be opened.",
            },
        }
    }
}

impl From<SearchError> for DesktopError {
    fn from(error: SearchError) -> Self {
        match error {
            SearchError::InvalidQuery => Self {
                code: "search_query_invalid",
                message: "Enter a valid search query.",
            },
            SearchError::IndexUnavailable => Self {
                code: "search_index_unavailable",
                message: "The local search index is unavailable.",
            },
            SearchError::RebuildFailed => Self {
                code: "search_rebuild_failed",
                message: "The local search index could not be rebuilt.",
            },
        }
    }
}

impl From<BookRelocationError> for DesktopError {
    fn from(error: BookRelocationError) -> Self {
        match error {
            BookRelocationError::BookNotFound => Self {
                code: "book_not_found",
                message: "The selected book is no longer in the catalog.",
            },
            BookRelocationError::WrongSourceType => Self {
                code: "replacement_type_invalid",
                message: "Choose a PDF for a PDF book or a folder for an image book.",
            },
            BookRelocationError::OutsideLibrary => Self {
                code: "replacement_outside_library",
                message: "Choose a replacement inside the configured library folder.",
            },
            BookRelocationError::InvalidPath => Self {
                code: "replacement_path_invalid",
                message: "The selected replacement path is unavailable or unsafe.",
            },
            BookRelocationError::PathConflict => Self {
                code: "replacement_path_conflict",
                message: "Another catalog book already uses that source path.",
            },
            BookRelocationError::RepositoryFailed => Self {
                code: "book_relink_failed",
                message: "The replacement source could not be saved.",
            },
        }
    }
}

fn search_result_response(result: SearchResultItem) -> SearchResultResponse {
    SearchResultResponse {
        source_kind: result.source_kind,
        source_id: result.source_id,
        scope: result.scope,
        title: result.title,
        snippet: result.snippet,
        relative_path: result.relative_path,
        status: result.status,
        rank: result.rank,
    }
}

fn search_rebuild_response(summary: SearchRebuildSummary) -> SearchRebuildResponse {
    SearchRebuildResponse {
        indexed: summary.indexed,
        failed: summary.failed,
    }
}

fn search_diagnostics_response(diagnostics: SearchDiagnostics) -> SearchDiagnosticsResponse {
    SearchDiagnosticsResponse {
        documents: diagnostics.documents,
        failed_jobs: diagnostics.failed_jobs,
        last_rebuild_at: diagnostics.last_rebuild_at,
    }
}

fn note_list_response(note: NoteListItem) -> NoteListResponse {
    NoteListResponse {
        id: note.id,
        title: note.title,
        relative_path: note.relative_path,
        status: note.status,
        book_id: note.book_id,
        book_title: note.book_title,
        modified_at_ms: note.modified_at_ms,
    }
}

fn note_detail_response(note: NoteDetail) -> NoteDetailResponse {
    NoteDetailResponse {
        id: note.id,
        title: note.title,
        relative_path: note.relative_path,
        body: note.body,
        book_id: note.book_id,
        book_title: note.book_title,
        backlinks: note
            .backlinks
            .into_iter()
            .map(|backlink| NoteBacklinkResponse {
                id: backlink.id,
                title: backlink.title,
                relative_path: backlink.relative_path,
            })
            .collect(),
    }
}

fn notes_refresh_response(summary: NotesRefreshSummary) -> NotesRefreshResponse {
    NotesRefreshResponse {
        discovered: summary.discovered,
        added: summary.added,
        updated: summary.updated,
        missing: summary.missing,
        issues: summary.issues,
    }
}

fn queue_search_refresh(backend: &BackendState) {
    if let Err(error) = backend.database.enqueue_search_rebuild() {
        tracing::warn!(
            event = "search_index_enqueue_failed",
            error = %error,
            "search projection refresh could not be queued"
        );
        return;
    }
    let database = Arc::clone(&backend.database);
    let markdown_notes = Arc::clone(&backend.markdown_notes);
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = SearchLibrary::new(database.as_ref(), markdown_notes.as_ref()).rebuild()
        {
            tracing::warn!(
                event = "search_incremental_rebuild_failed",
                error = %error,
                "search projection could not be refreshed"
            );
        }
    });
}

#[tauri::command]
fn get_application_status(
    backend: State<'_, BackendState>,
) -> Result<ApplicationStatusResponse, DesktopError> {
    let status = GetApplicationStatus::new(backend.database.as_ref(), backend.database.as_ref())
        .execute()
        .map_err(DesktopError::from)?;
    Ok(ApplicationStatusResponse {
        database_healthy: status.database_healthy,
        library_configured: status.library_configured,
        platform: PlatformResponse {
            os: status.platform.os,
            architecture: status.platform.architecture,
            supported: status.platform.supported,
        },
    })
}

#[tauri::command]
fn configure_library(
    selected_root: String,
    backend: State<'_, BackendState>,
) -> Result<LibraryConfigurationResponse, DesktopError> {
    let configuration = ConfigureLibrary::new(backend.database.as_ref())
        .execute(selected_root)
        .map_err(DesktopError::from)?;
    Ok(LibraryConfigurationResponse {
        display_name: configuration.display_name,
    })
}

#[tauri::command]
fn get_library_configuration(
    backend: State<'_, BackendState>,
) -> Result<Option<LibraryConfigurationResponse>, DesktopError> {
    backend
        .database
        .configuration()
        .map(|configuration| {
            configuration.map(|value| LibraryConfigurationResponse {
                display_name: value.display_name,
            })
        })
        .map_err(DesktopError::from)
}

async fn execute_scan(
    app: AppHandle,
    database: Arc<SqliteDatabase>,
    scanner: Arc<FilesystemScanner>,
    thumbnails: Arc<ThumbnailService>,
    markdown_notes: Arc<MarkdownNotesStore>,
    active_scan: Arc<Mutex<Option<CancellationToken>>>,
    reason: ScanReason,
) -> Result<ScanSummaryResponse, DesktopError> {
    let cancellation = CancellationToken::default();
    {
        let mut active = active_scan.lock().map_err(|_| DesktopError {
            code: "scan_state_failed",
            message: "The scan state is unavailable.",
        })?;
        if active.is_some() {
            return Err(DesktopError {
                code: "scan_already_running",
                message: "A library scan is already running.",
            });
        }
        *active = Some(cancellation.clone());
    }

    let app_for_progress = app.clone();
    let scan_database = Arc::clone(&database);
    let result = tauri::async_runtime::spawn_blocking(move || {
        let use_case = ReconcileCatalog::new(
            scan_database.as_ref(),
            scanner.as_ref(),
            thumbnails.as_ref(),
        );
        let mut progress = |value: ScanProgress| {
            let payload = ScanProgressResponse {
                visited_entries: value.visited_entries,
                discovered_books: value.discovered_books,
                current_relative_path: value.current_relative_path,
            };
            if let Err(error) = app_for_progress.emit("library_scan_progressed", payload) {
                tracing::warn!(
                    event = "library_scan_progress_emit_failed",
                    error_code = "event_emit_failed",
                    error = %error,
                    "scan progress event could not be emitted"
                );
            }
        };
        use_case.execute(reason, &cancellation, &mut progress)
    })
    .await
    .map_err(|_| DesktopError {
        code: "scan_task_failed",
        message: "The scan worker stopped unexpectedly.",
    })?;

    if let Ok(mut active) = active_scan.lock() {
        *active = None;
    }
    let summary = result.map_err(DesktopError::from)?;
    if let Err(error) = SearchLibrary::new(database.as_ref(), markdown_notes.as_ref()).rebuild() {
        tracing::warn!(event = "post_scan_search_rebuild_failed", error = %error);
    }
    Ok(ScanSummaryResponse {
        discovered: summary.discovered,
        added: summary.added,
        updated: summary.updated,
        missing: summary.missing,
        issues: summary.issues,
        thumbnails_generated: summary.thumbnails_generated,
        thumbnail_failures: summary.thumbnail_failures,
        cancelled: summary.cancelled,
    })
}

#[tauri::command]
async fn rescan_library(
    app: AppHandle,
    backend: State<'_, BackendState>,
) -> Result<ScanSummaryResponse, DesktopError> {
    execute_scan(
        app,
        Arc::clone(&backend.database),
        Arc::clone(&backend.scanner),
        Arc::clone(&backend.thumbnails),
        Arc::clone(&backend.markdown_notes),
        Arc::clone(&backend.active_scan),
        ScanReason::Manual,
    )
    .await
}

#[tauri::command]
async fn initialize_library(
    app: AppHandle,
    backend: State<'_, BackendState>,
) -> Result<ScanSummaryResponse, DesktopError> {
    execute_scan(
        app,
        Arc::clone(&backend.database),
        Arc::clone(&backend.scanner),
        Arc::clone(&backend.thumbnails),
        Arc::clone(&backend.markdown_notes),
        Arc::clone(&backend.active_scan),
        ScanReason::Initial,
    )
    .await
}

#[tauri::command]
async fn repair_library(
    app: AppHandle,
    backend: State<'_, BackendState>,
) -> Result<ScanSummaryResponse, DesktopError> {
    execute_scan(
        app,
        Arc::clone(&backend.database),
        Arc::clone(&backend.scanner),
        Arc::clone(&backend.thumbnails),
        Arc::clone(&backend.markdown_notes),
        Arc::clone(&backend.active_scan),
        ScanReason::Repair,
    )
    .await
}

#[tauri::command]
fn cancel_library_scan(backend: State<'_, BackendState>) -> Result<bool, DesktopError> {
    let active = backend.active_scan.lock().map_err(|_| DesktopError {
        code: "scan_state_failed",
        message: "The scan state is unavailable.",
    })?;
    if let Some(cancellation) = active.as_ref() {
        cancellation.cancel();
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn list_library_books(backend: State<'_, BackendState>) -> Result<Vec<BookResponse>, DesktopError> {
    let books = backend.database.list_books().map_err(DesktopError::from)?;
    Ok(books
        .into_iter()
        .map(|book| {
            let thumbnail_data_url = book.thumbnail_cache_path.as_deref().and_then(|path| {
                backend
                    .database
                    .thumbnail_bytes(path)
                    .ok()
                    .map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
            });
            BookResponse {
                id: book.id,
                title: book.title,
                kind: book.kind,
                relative_path: book.relative_path,
                status: book.status,
                page_count: book.page_count,
                size_bytes: book.size_bytes,
                modified_at_ms: book.modified_at_ms,
                thumbnail_data_url,
                thumbnail_status: book.thumbnail_status,
            }
        })
        .collect())
}

#[tauri::command]
fn open_book_location(
    book_id: String,
    backend: State<'_, BackendState>,
) -> Result<(), DesktopError> {
    let book_id = BookId::parse(&book_id)
        .map_err(|_| DesktopError::from(SourceLocationError::InvalidBookId))?;
    OpenBookLocation::new(backend.database.as_ref(), backend.file_manager.as_ref())
        .execute(book_id)
        .map_err(DesktopError::from)
}

#[tauri::command]
fn update_book_display_title(
    book_id: String,
    title: String,
    backend: State<'_, BackendState>,
) -> Result<UpdatedBookTitleResponse, DesktopError> {
    let book_id = BookId::parse(&book_id)
        .map_err(|_| DesktopError::from(BookMetadataError::InvalidBookId))?;
    let title = UpdateBookDisplayTitle::new(backend.database.as_ref())
        .execute(book_id, &title)
        .map_err(DesktopError::from)?;
    queue_search_refresh(&backend);
    Ok(UpdatedBookTitleResponse { title })
}

#[tauri::command]
fn get_book_detail(
    book_id: String,
    backend: State<'_, BackendState>,
) -> Result<BookDetailResponse, DesktopError> {
    let book_id =
        BookId::parse(&book_id).map_err(|_| DesktopError::from(BookDetailError::BookNotFound))?;
    let detail = GetBookDetail::new(backend.database.as_ref())
        .execute(book_id)
        .map_err(DesktopError::from)?;
    Ok(book_detail_response(detail, backend.database.as_ref()))
}

#[tauri::command]
fn update_book_detail(
    book_id: String,
    reading_status: String,
    tags: Vec<String>,
    backend: State<'_, BackendState>,
) -> Result<BookDetailResponse, DesktopError> {
    let book_id =
        BookId::parse(&book_id).map_err(|_| DesktopError::from(BookDetailError::BookNotFound))?;
    UpdateBookDetail::new(backend.database.as_ref())
        .execute(book_id, &reading_status, tags)
        .map_err(DesktopError::from)?;
    queue_search_refresh(&backend);
    let detail = GetBookDetail::new(backend.database.as_ref())
        .execute(book_id)
        .map_err(DesktopError::from)?;
    Ok(book_detail_response(detail, backend.database.as_ref()))
}

#[tauri::command]
async fn force_book_cover(
    book_id: String,
    backend: State<'_, BackendState>,
) -> Result<BookDetailResponse, DesktopError> {
    let book_id =
        BookId::parse(&book_id).map_err(|_| DesktopError::from(BookDetailError::BookNotFound))?;
    let database = Arc::clone(&backend.database);
    let thumbnails = Arc::clone(&backend.thumbnails);
    tauri::async_runtime::spawn_blocking(move || {
        ForceBookCover::new(database.as_ref(), thumbnails.as_ref()).execute(book_id)
    })
    .await
    .map_err(|_| DesktopError {
        code: "cover_task_failed",
        message: "The cover worker stopped unexpectedly.",
    })?
    .map_err(DesktopError::from)?;
    let detail = GetBookDetail::new(backend.database.as_ref())
        .execute(book_id)
        .map_err(DesktopError::from)?;
    Ok(book_detail_response(detail, backend.database.as_ref()))
}

#[tauri::command]
fn relink_missing_book(
    book_id: String,
    selected_path: String,
    backend: State<'_, BackendState>,
) -> Result<String, DesktopError> {
    let book_id = BookId::parse(&book_id)
        .map_err(|_| DesktopError::from(BookRelocationError::BookNotFound))?;
    let relative_path = RelinkMissingBook::new(backend.database.as_ref())
        .execute(book_id, selected_path)
        .map_err(DesktopError::from)?;
    queue_search_refresh(&backend);
    Ok(relative_path.to_string())
}

#[tauri::command]
fn get_notes_configuration(
    backend: State<'_, BackendState>,
) -> Result<Option<NotesConfigurationResponse>, DesktopError> {
    backend
        .database
        .notes_configuration()
        .map(|value| {
            value.map(|configuration| NotesConfigurationResponse {
                display_name: configuration.display_name,
            })
        })
        .map_err(DesktopError::from)
}

#[tauri::command]
fn configure_notes_root(
    selected_root: String,
    backend: State<'_, BackendState>,
) -> Result<NotesConfigurationResponse, DesktopError> {
    let workspace = NotesWorkspace::new(
        backend.database.as_ref(),
        backend.markdown_notes.as_ref(),
        backend.file_manager.as_ref(),
    );
    let configuration = workspace
        .configure(selected_root)
        .map_err(DesktopError::from)?;
    Ok(NotesConfigurationResponse {
        display_name: configuration.display_name,
    })
}

#[tauri::command]
fn refresh_notes(backend: State<'_, BackendState>) -> Result<NotesRefreshResponse, DesktopError> {
    let summary = NotesWorkspace::new(
        backend.database.as_ref(),
        backend.markdown_notes.as_ref(),
        backend.file_manager.as_ref(),
    )
    .refresh()
    .map_err(DesktopError::from)?;
    queue_search_refresh(&backend);
    Ok(notes_refresh_response(summary))
}

#[tauri::command]
fn list_notes(backend: State<'_, BackendState>) -> Result<Vec<NoteListResponse>, DesktopError> {
    NotesWorkspace::new(
        backend.database.as_ref(),
        backend.markdown_notes.as_ref(),
        backend.file_manager.as_ref(),
    )
    .list()
    .map(|notes| notes.into_iter().map(note_list_response).collect())
    .map_err(DesktopError::from)
}

#[tauri::command]
fn create_note(
    title: String,
    book_id: Option<String>,
    backend: State<'_, BackendState>,
) -> Result<NoteDetailResponse, DesktopError> {
    let book_id = book_id
        .as_deref()
        .map(BookId::parse)
        .transpose()
        .map_err(|_| DesktopError::from(NotesError::BookNotFound))?;
    let detail = NotesWorkspace::new(
        backend.database.as_ref(),
        backend.markdown_notes.as_ref(),
        backend.file_manager.as_ref(),
    )
    .create(&title, book_id)
    .map_err(DesktopError::from)?;
    queue_search_refresh(&backend);
    Ok(note_detail_response(detail))
}

#[tauri::command]
fn read_note(
    note_id: String,
    backend: State<'_, BackendState>,
) -> Result<NoteDetailResponse, DesktopError> {
    let note_id =
        NoteId::parse(&note_id).map_err(|_| DesktopError::from(NotesError::NoteNotFound))?;
    NotesWorkspace::new(
        backend.database.as_ref(),
        backend.markdown_notes.as_ref(),
        backend.file_manager.as_ref(),
    )
    .read(note_id)
    .map(note_detail_response)
    .map_err(DesktopError::from)
}

#[tauri::command]
fn save_note(
    note_id: String,
    body: String,
    backend: State<'_, BackendState>,
) -> Result<NoteDetailResponse, DesktopError> {
    let note_id =
        NoteId::parse(&note_id).map_err(|_| DesktopError::from(NotesError::NoteNotFound))?;
    let detail = NotesWorkspace::new(
        backend.database.as_ref(),
        backend.markdown_notes.as_ref(),
        backend.file_manager.as_ref(),
    )
    .save(note_id, &body)
    .map_err(DesktopError::from)?;
    queue_search_refresh(&backend);
    Ok(note_detail_response(detail))
}

#[tauri::command]
fn open_note_external(
    note_id: String,
    backend: State<'_, BackendState>,
) -> Result<(), DesktopError> {
    let note_id =
        NoteId::parse(&note_id).map_err(|_| DesktopError::from(NotesError::NoteNotFound))?;
    NotesWorkspace::new(
        backend.database.as_ref(),
        backend.markdown_notes.as_ref(),
        backend.file_manager.as_ref(),
    )
    .open_note(note_id)
    .map_err(DesktopError::from)
}

#[tauri::command]
fn open_notes_root(backend: State<'_, BackendState>) -> Result<(), DesktopError> {
    NotesWorkspace::new(
        backend.database.as_ref(),
        backend.markdown_notes.as_ref(),
        backend.file_manager.as_ref(),
    )
    .open_root()
    .map_err(DesktopError::from)
}

#[tauri::command]
fn search_library(
    query: String,
    scope: Option<String>,
    backend: State<'_, BackendState>,
) -> Result<Vec<SearchResultResponse>, DesktopError> {
    SearchLibrary::new(backend.database.as_ref(), backend.markdown_notes.as_ref())
        .execute(&query, scope.as_deref())
        .map(|results| results.into_iter().map(search_result_response).collect())
        .map_err(DesktopError::from)
}

#[tauri::command]
fn rebuild_search_index(
    backend: State<'_, BackendState>,
) -> Result<SearchRebuildResponse, DesktopError> {
    SearchLibrary::new(backend.database.as_ref(), backend.markdown_notes.as_ref())
        .rebuild()
        .map(search_rebuild_response)
        .map_err(DesktopError::from)
}

#[tauri::command]
fn get_search_diagnostics(
    backend: State<'_, BackendState>,
) -> Result<SearchDiagnosticsResponse, DesktopError> {
    SearchLibrary::new(backend.database.as_ref(), backend.markdown_notes.as_ref())
        .diagnostics()
        .map(search_diagnostics_response)
        .map_err(DesktopError::from)
}

pub(crate) fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let logging_guard: LoggingGuard = initialize_logging(&app_data_dir)?;
            let database = Arc::new(SqliteDatabase::initialize(&app_data_dir)?);
            let resource_pdfium = app
                .path()
                .resource_dir()?
                .join("resources/pdfium/windows-x86_64");
            let development_pdfium =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/pdfium/windows-x86_64");
            let pdfium_directory = if resource_pdfium.is_dir() {
                resource_pdfium
            } else {
                development_pdfium
            };
            let backend = BackendState {
                database,
                scanner: Arc::new(FilesystemScanner::new()),
                thumbnails: Arc::new(ThumbnailService::new(
                    app_data_dir.join("cache"),
                    pdfium_directory,
                )),
                file_manager: Arc::new(SystemFileManager::new()),
                markdown_notes: Arc::new(MarkdownNotesStore::new()),
                active_scan: Arc::new(Mutex::new(None)),
            };
            tracing::info!(
                event = "application_started",
                operation_id = %uuid::Uuid::new_v4(),
                os = std::env::consts::OS,
                architecture = std::env::consts::ARCH,
                "application foundation initialized"
            );
            app.manage(logging_guard);
            app.manage(backend);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_application_status,
            configure_library,
            get_library_configuration,
            initialize_library,
            rescan_library,
            repair_library,
            cancel_library_scan,
            list_library_books,
            open_book_location,
            update_book_display_title,
            get_book_detail,
            update_book_detail,
            force_book_cover,
            relink_missing_book,
            get_notes_configuration,
            configure_notes_root,
            refresh_notes,
            list_notes,
            create_note,
            read_note,
            save_note,
            open_note_external,
            open_notes_root,
            search_library,
            rebuild_search_index,
            get_search_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Book Library");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_application_errors_to_stable_safe_envelopes() {
        let database_error = DesktopError::from(ApplicationError::DatabaseUnavailable);
        let library_error = DesktopError::from(LibraryError::RootUnreadable);
        let source_error = DesktopError::from(SourceLocationError::SourceUnavailable);
        assert_eq!(database_error.code, "database_unavailable");
        assert_eq!(library_error.code, "library_root_unreadable");
        assert_eq!(source_error.code, "book_source_unavailable");
        assert!(!library_error.message.contains('\\'));
        assert!(!library_error.message.contains('/'));
        assert!(!source_error.message.contains('\\'));
        assert!(!source_error.message.contains('/'));
    }
}
