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
        ApplicationError, CancellationToken, ConfigureLibrary, GetApplicationStatus, LibraryError,
        LibraryRepository, ReconcileCatalog, ScanProgress, ScanReason,
    },
    infrastructure::{
        FilesystemScanner, LoggingGuard, SqliteDatabase, ThumbnailService, initialize_logging,
    },
};

struct BackendState {
    database: Arc<SqliteDatabase>,
    scanner: Arc<FilesystemScanner>,
    thumbnails: Arc<ThumbnailService>,
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
    let result = tauri::async_runtime::spawn_blocking(move || {
        let use_case =
            ReconcileCatalog::new(database.as_ref(), scanner.as_ref(), thumbnails.as_ref());
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
            list_library_books
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
        assert_eq!(database_error.code, "database_unavailable");
        assert_eq!(library_error.code, "library_root_unreadable");
        assert!(!library_error.message.contains('\\'));
        assert!(!library_error.message.contains('/'));
    }
}
