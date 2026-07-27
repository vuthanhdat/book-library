use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use thiserror::Error;

use crate::{
    application::CancellationToken,
    domain::{BookId, BookKind, BookStatus, ContentFingerprint, LibraryId, RelativePath},
};

#[derive(Debug, Error)]
pub(crate) enum LibraryError {
    #[error("the selected library root does not exist")]
    RootMissing,
    #[error("the selected library root is not a readable directory")]
    RootUnreadable,
    #[error("the selected library root could not be resolved safely")]
    RootInvalid,
    #[error("library configuration could not be saved")]
    ConfigurationFailed,
    #[error("no library root is configured")]
    NotConfigured,
    #[error("the library scan failed")]
    ScanFailed,
    #[error("catalog persistence failed")]
    CatalogFailed,
    #[error("thumbnail generation failed")]
    ThumbnailFailed,
}

#[derive(Debug, Clone)]
pub(crate) struct LibraryConfigurationState {
    pub(crate) id: LibraryId,
    pub(crate) root: PathBuf,
    pub(crate) display_name: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ScanReason {
    Initial,
    Manual,
    Repair,
}

impl ScanReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Manual => "manual",
            Self::Repair => "repair",
        }
    }
}

fn thumbnail_timeout(reason: ScanReason) -> Duration {
    if matches!(reason, ScanReason::Repair) {
        Duration::from_secs(30)
    } else {
        Duration::from_secs(1)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScanProgress {
    pub(crate) visited_entries: u64,
    pub(crate) discovered_books: u64,
    pub(crate) current_relative_path: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ScanIssue {
    pub(crate) relative_path: Option<String>,
    pub(crate) severity: &'static str,
    pub(crate) code: &'static str,
    pub(crate) message: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct DiscoveredBook {
    pub(crate) kind: BookKind,
    pub(crate) status: BookStatus,
    pub(crate) relative_path: RelativePath,
    pub(crate) path_key: String,
    pub(crate) title: String,
    pub(crate) fingerprint: ContentFingerprint,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) modified_at_ms: Option<i64>,
    pub(crate) page_count: Option<u32>,
    pub(crate) image_pages: Vec<RelativePath>,
}

#[derive(Debug, Clone)]
pub(crate) struct ScanResult {
    pub(crate) books: Vec<DiscoveredBook>,
    pub(crate) issues: Vec<ScanIssue>,
    pub(crate) cancelled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ThumbnailOutcome {
    pub(crate) cache_relative_path: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: &'static str,
    pub(crate) source_fingerprint: String,
    pub(crate) page_count: Option<u32>,
}

#[derive(Debug, Clone)]
pub(crate) struct CatalogReconciliation {
    pub(crate) added: u64,
    pub(crate) updated: u64,
    pub(crate) missing: u64,
    pub(crate) thumbnail_targets: Vec<(BookId, DiscoveredBook)>,
}

#[derive(Debug, Clone)]
pub(crate) struct ScanSummary {
    pub(crate) job_id: String,
    pub(crate) discovered: u64,
    pub(crate) added: u64,
    pub(crate) updated: u64,
    pub(crate) missing: u64,
    pub(crate) issues: u64,
    pub(crate) thumbnails_recovered: u64,
    pub(crate) thumbnails_generated: u64,
    pub(crate) thumbnail_failures: u64,
    pub(crate) cancelled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct BookListItem {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) kind: String,
    pub(crate) relative_path: String,
    pub(crate) status: String,
    pub(crate) page_count: Option<u32>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) modified_at_ms: Option<i64>,
    pub(crate) thumbnail_cache_path: Option<String>,
    pub(crate) thumbnail_status: String,
}

pub(crate) trait LibraryRepository {
    fn save_configuration(
        &self,
        root: &Path,
        display_name: &str,
    ) -> Result<LibraryConfigurationState, LibraryError>;
    fn configuration(&self) -> Result<Option<LibraryConfigurationState>, LibraryError>;
    fn start_scan(&self, library_id: LibraryId, reason: ScanReason)
    -> Result<String, LibraryError>;
    fn reconcile(
        &self,
        library_id: LibraryId,
        job_id: &str,
        result: &ScanResult,
    ) -> Result<CatalogReconciliation, LibraryError>;
    fn save_thumbnail(
        &self,
        book_id: BookId,
        outcome: &ThumbnailOutcome,
    ) -> Result<(), LibraryError>;
    fn save_thumbnail_failure(
        &self,
        book_id: BookId,
        error_code: &'static str,
    ) -> Result<(), LibraryError>;
    fn finish_scan(&self, summary: &ScanSummary) -> Result<(), LibraryError>;
    fn recover_thumbnails(&self) -> Result<u64, LibraryError>;
    fn invalidate_thumbnails(&self) -> Result<(), LibraryError>;
    fn list_books(&self) -> Result<Vec<BookListItem>, LibraryError>;
    fn thumbnail_bytes(&self, cache_relative_path: &str) -> Result<Vec<u8>, LibraryError>;
}

pub(crate) trait LibraryScanner {
    fn scan(
        &self,
        root: &Path,
        reason: ScanReason,
        cancellation: &CancellationToken,
        progress: &mut dyn FnMut(ScanProgress),
    ) -> Result<ScanResult, LibraryError>;
}

pub(crate) trait ThumbnailGenerator: Sync {
    fn generate(
        &self,
        root: &Path,
        book_id: BookId,
        book: &DiscoveredBook,
    ) -> Result<ThumbnailOutcome, LibraryError>;

    fn generate_with_timeout(
        &self,
        root: &Path,
        book_id: BookId,
        book: &DiscoveredBook,
        _timeout: std::time::Duration,
    ) -> Result<ThumbnailOutcome, LibraryError> {
        self.generate(root, book_id, book)
    }
}

pub(crate) struct ConfigureLibrary<'a, Repository> {
    repository: &'a Repository,
}

impl<'a, Repository: LibraryRepository> ConfigureLibrary<'a, Repository> {
    pub(crate) fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }

    pub(crate) fn execute(
        &self,
        selected_root: impl AsRef<Path>,
    ) -> Result<LibraryConfigurationState, LibraryError> {
        let root = selected_root.as_ref();
        if !root.exists() {
            return Err(LibraryError::RootMissing);
        }
        if !root.is_dir() {
            return Err(LibraryError::RootUnreadable);
        }
        let canonical = root.canonicalize().map_err(|_| LibraryError::RootInvalid)?;
        std::fs::read_dir(&canonical).map_err(|_| LibraryError::RootUnreadable)?;
        let display_name = canonical
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("Library");

        self.repository.save_configuration(&canonical, display_name)
    }
}

pub(crate) struct ReconcileCatalog<'a, Repository, Scanner, Thumbnails> {
    repository: &'a Repository,
    scanner: &'a Scanner,
    thumbnails: &'a Thumbnails,
}

impl<'a, Repository, Scanner, Thumbnails> ReconcileCatalog<'a, Repository, Scanner, Thumbnails>
where
    Repository: LibraryRepository,
    Scanner: LibraryScanner,
    Thumbnails: ThumbnailGenerator,
{
    pub(crate) fn new(
        repository: &'a Repository,
        scanner: &'a Scanner,
        thumbnails: &'a Thumbnails,
    ) -> Self {
        Self {
            repository,
            scanner,
            thumbnails,
        }
    }

    pub(crate) fn execute(
        &self,
        reason: ScanReason,
        cancellation: &CancellationToken,
        progress: &mut dyn FnMut(ScanProgress),
    ) -> Result<ScanSummary, LibraryError> {
        let configuration = self
            .repository
            .configuration()?
            .ok_or(LibraryError::NotConfigured)?;
        let thumbnails_recovered = if matches!(reason, ScanReason::Repair) {
            let recovered = self.repository.recover_thumbnails()?;
            self.repository.invalidate_thumbnails()?;
            recovered
        } else {
            0
        };
        let job_id = self.repository.start_scan(configuration.id, reason)?;
        let scan = self
            .scanner
            .scan(&configuration.root, reason, cancellation, progress)?;
        let reconciliation = self
            .repository
            .reconcile(configuration.id, &job_id, &scan)?;
        let mut generated = 0;
        let mut thumbnail_failures = 0;

        if !scan.cancelled {
            let thumbnail_timeout = thumbnail_timeout(reason);
            for batch in reconciliation.thumbnail_targets.chunks(8) {
                if cancellation.is_cancelled() {
                    break;
                }
                let results = std::thread::scope(|scope| {
                    let workers = batch
                        .iter()
                        .map(|(book_id, book)| {
                            scope.spawn(|| {
                                (
                                    *book_id,
                                    self.thumbnails.generate_with_timeout(
                                        &configuration.root,
                                        *book_id,
                                        book,
                                        thumbnail_timeout,
                                    ),
                                )
                            })
                        })
                        .collect::<Vec<_>>();
                    workers
                        .into_iter()
                        .zip(batch.iter())
                        .map(|(worker, (book_id, _))| {
                            worker
                                .join()
                                .unwrap_or((*book_id, Err(LibraryError::ThumbnailFailed)))
                        })
                        .collect::<Vec<_>>()
                });
                for (book_id, result) in results {
                    match result {
                        Ok(outcome) => {
                            self.repository.save_thumbnail(book_id, &outcome)?;
                            generated += 1;
                        }
                        Err(_) => {
                            self.repository
                                .save_thumbnail_failure(book_id, "thumbnail_failed")?;
                            thumbnail_failures += 1;
                        }
                    }
                }
            }
        }

        let summary = ScanSummary {
            job_id,
            discovered: scan.books.len() as u64,
            added: reconciliation.added,
            updated: reconciliation.updated,
            missing: reconciliation.missing,
            issues: scan.issues.len() as u64,
            thumbnails_recovered,
            thumbnails_generated: generated,
            thumbnail_failures,
            cancelled: scan.cancelled || cancellation.is_cancelled(),
        };
        self.repository.finish_scan(&summary)?;
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repair_allows_cloud_hydration_without_slowing_normal_scans() {
        assert_eq!(
            thumbnail_timeout(ScanReason::Repair),
            Duration::from_secs(30)
        );
        assert_eq!(
            thumbnail_timeout(ScanReason::Initial),
            Duration::from_secs(1)
        );
        assert_eq!(
            thumbnail_timeout(ScanReason::Manual),
            Duration::from_secs(1)
        );
    }
}
