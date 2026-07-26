//! Application use cases and infrastructure port contracts.

mod book_detail;
mod book_metadata;
mod book_relocation;
mod library;
mod notes;
#[allow(dead_code)]
mod operation;
mod search;
mod source_location;
mod status;

pub(crate) use book_detail::{
    BookDetailError, BookDetailRecord, BookDetailRepository, BookThumbnailTarget, ForceBookCover,
    GetBookDetail, LinkedBookNote, UpdateBookDetail,
};
pub(crate) use book_metadata::{BookMetadataError, BookMetadataRepository, UpdateBookDisplayTitle};
pub(crate) use book_relocation::{
    BookRelocationError, BookRelocationRepository, RelinkMissingBook,
};
pub(crate) use library::{
    BookListItem, CatalogReconciliation, ConfigureLibrary, DiscoveredBook,
    LibraryConfigurationState, LibraryError, LibraryRepository, LibraryScanner, ReconcileCatalog,
    ScanIssue, ScanProgress, ScanReason, ScanResult, ScanSummary, ThumbnailGenerator,
    ThumbnailOutcome,
};
pub(crate) use notes::{
    ExternalPathOpener, MarkdownNotes, NoteBacklink, NoteDetail, NoteListItem, NoteProjection,
    NoteRecord, NotesConfiguration, NotesError, NotesRefreshSummary, NotesRepository,
    NotesWorkspace, ParsedHeading, ParsedNoteLink,
};
#[allow(unused_imports)]
pub(crate) use operation::{CancellationToken, EventEnvelope, OperationId};
pub(crate) use search::{
    SearchDiagnostics, SearchDocument, SearchError, SearchLibrary, SearchRebuildSummary,
    SearchRepository, SearchResultItem,
};
pub(crate) use source_location::{
    BookLocationRepository, BookSourceLocation, FileManager, OpenBookLocation, SourceLocationError,
};
pub(crate) use status::{
    ApplicationError, DatabaseHealth, GetApplicationStatus, LibraryConfiguration,
};
