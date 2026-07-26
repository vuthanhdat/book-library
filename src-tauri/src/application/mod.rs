//! Application use cases and infrastructure port contracts.

mod library;
#[allow(dead_code)]
mod operation;
mod status;

pub(crate) use library::{
    BookListItem, CatalogReconciliation, ConfigureLibrary, DiscoveredBook,
    LibraryConfigurationState, LibraryError, LibraryRepository, LibraryScanner, ReconcileCatalog,
    ScanIssue, ScanProgress, ScanReason, ScanResult, ScanSummary, ThumbnailGenerator,
    ThumbnailOutcome,
};
#[allow(unused_imports)]
pub(crate) use operation::{CancellationToken, EventEnvelope, OperationId};
pub(crate) use status::{
    ApplicationError, DatabaseHealth, GetApplicationStatus, LibraryConfiguration,
};
