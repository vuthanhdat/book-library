use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::{BookId, NoteId, RelativePath};

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum NotesError {
    #[error("the notes root does not exist or is not readable")]
    RootUnavailable,
    #[error("the notes root path is invalid")]
    RootInvalid,
    #[error("no notes root is configured")]
    NotConfigured,
    #[error("the requested note does not exist")]
    NoteNotFound,
    #[error("the requested note path is invalid")]
    InvalidNotePath,
    #[error("the note title is invalid")]
    InvalidTitle,
    #[error("the note body is invalid")]
    InvalidBody,
    #[error("the requested book does not exist")]
    BookNotFound,
    #[error("the Markdown file could not be read")]
    ReadFailed,
    #[error("the Markdown file could not be saved")]
    WriteFailed,
    #[error("the notes projection could not be saved")]
    RepositoryFailed,
    #[error("the external application could not be opened")]
    LaunchFailed,
}

#[derive(Debug, Clone)]
pub(crate) struct NotesConfiguration {
    pub(crate) root: PathBuf,
    pub(crate) display_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedNoteLink {
    pub(crate) target_ref: String,
    pub(crate) link_text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedHeading {
    pub(crate) level: u8,
    pub(crate) text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct NoteProjection {
    pub(crate) relative_path: RelativePath,
    pub(crate) path_key: String,
    pub(crate) title: String,
    pub(crate) fingerprint: String,
    pub(crate) size_bytes: u64,
    pub(crate) modified_at_ms: Option<i64>,
    pub(crate) book_relative_path: Option<RelativePath>,
    pub(crate) headings: Vec<ParsedHeading>,
    pub(crate) tags: Vec<String>,
    pub(crate) links: Vec<ParsedNoteLink>,
}

#[derive(Debug, Clone)]
pub(crate) struct NoteRecord {
    pub(crate) relative_path: RelativePath,
    pub(crate) status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct NoteListItem {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) relative_path: String,
    pub(crate) status: String,
    pub(crate) book_id: Option<String>,
    pub(crate) book_title: Option<String>,
    pub(crate) modified_at_ms: Option<i64>,
}

#[derive(Debug, Clone)]
pub(crate) struct NoteBacklink {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) relative_path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct NoteDetail {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) relative_path: String,
    pub(crate) body: String,
    pub(crate) book_id: Option<String>,
    pub(crate) book_title: Option<String>,
    pub(crate) backlinks: Vec<NoteBacklink>,
}

#[derive(Debug, Clone)]
pub(crate) struct NotesRefreshSummary {
    pub(crate) discovered: u64,
    pub(crate) added: u64,
    pub(crate) updated: u64,
    pub(crate) missing: u64,
    pub(crate) issues: u64,
}

pub(crate) trait NotesRepository {
    fn save_notes_configuration(&self, root: &Path) -> Result<NotesConfiguration, NotesError>;
    fn notes_configuration(&self) -> Result<Option<NotesConfiguration>, NotesError>;
    fn reconcile_notes(
        &self,
        notes: &[NoteProjection],
        issues: u64,
    ) -> Result<NotesRefreshSummary, NotesError>;
    fn upsert_note(&self, note: &NoteProjection) -> Result<NoteId, NotesError>;
    fn note_record(&self, note_id: NoteId) -> Result<Option<NoteRecord>, NotesError>;
    fn book_relative_path(&self, book_id: BookId) -> Result<Option<RelativePath>, NotesError>;
    fn list_notes(&self) -> Result<Vec<NoteListItem>, NotesError>;
    fn note_detail_projection(
        &self,
        note_id: NoteId,
        body: String,
    ) -> Result<Option<NoteDetail>, NotesError>;
}

pub(crate) trait MarkdownNotes {
    fn scan(&self, root: &Path) -> Result<(Vec<NoteProjection>, u64), NotesError>;
    fn create(
        &self,
        root: &Path,
        title: &str,
        book_relative_path: Option<&RelativePath>,
    ) -> Result<(NoteProjection, String), NotesError>;
    fn read(&self, root: &Path, relative_path: &RelativePath) -> Result<String, NotesError>;
    fn save(
        &self,
        root: &Path,
        relative_path: &RelativePath,
        body: &str,
    ) -> Result<NoteProjection, NotesError>;
    fn resolve(&self, root: &Path, relative_path: &RelativePath) -> Result<PathBuf, NotesError>;
}

pub(crate) trait ExternalPathOpener {
    fn open_path(&self, path: &Path) -> Result<(), NotesError>;
}

pub(crate) struct NotesWorkspace<'a, Repository, Markdown, Opener> {
    repository: &'a Repository,
    markdown: &'a Markdown,
    opener: &'a Opener,
}

impl<'a, Repository, Markdown, Opener> NotesWorkspace<'a, Repository, Markdown, Opener>
where
    Repository: NotesRepository,
    Markdown: MarkdownNotes,
    Opener: ExternalPathOpener,
{
    pub(crate) fn new(
        repository: &'a Repository,
        markdown: &'a Markdown,
        opener: &'a Opener,
    ) -> Self {
        Self {
            repository,
            markdown,
            opener,
        }
    }

    pub(crate) fn configure(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<NotesConfiguration, NotesError> {
        let root = root.as_ref();
        if !root.is_dir() || std::fs::read_dir(root).is_err() {
            return Err(NotesError::RootUnavailable);
        }
        let root = root.canonicalize().map_err(|_| NotesError::RootInvalid)?;
        self.repository.save_notes_configuration(&root)
    }

    pub(crate) fn configuration(&self) -> Result<Option<NotesConfiguration>, NotesError> {
        self.repository.notes_configuration()
    }

    pub(crate) fn refresh(&self) -> Result<NotesRefreshSummary, NotesError> {
        let configuration = self.configuration()?.ok_or(NotesError::NotConfigured)?;
        let (notes, issues) = self.markdown.scan(&configuration.root)?;
        self.repository.reconcile_notes(&notes, issues)
    }

    pub(crate) fn list(&self) -> Result<Vec<NoteListItem>, NotesError> {
        self.repository.list_notes()
    }

    pub(crate) fn create(
        &self,
        title: &str,
        book_id: Option<BookId>,
    ) -> Result<NoteDetail, NotesError> {
        let title = validate_title(title)?;
        let configuration = self.configuration()?.ok_or(NotesError::NotConfigured)?;
        let book_path = book_id
            .map(|id| {
                self.repository
                    .book_relative_path(id)?
                    .ok_or(NotesError::BookNotFound)
            })
            .transpose()?;
        let (projection, body) =
            self.markdown
                .create(&configuration.root, title, book_path.as_ref())?;
        let note_id = self.repository.upsert_note(&projection)?;
        self.repository
            .note_detail_projection(note_id, body)?
            .ok_or(NotesError::NoteNotFound)
    }

    pub(crate) fn read(&self, note_id: NoteId) -> Result<NoteDetail, NotesError> {
        let configuration = self.configuration()?.ok_or(NotesError::NotConfigured)?;
        let record = self
            .repository
            .note_record(note_id)?
            .ok_or(NotesError::NoteNotFound)?;
        if record.status == "missing" {
            return Err(NotesError::NoteNotFound);
        }
        let body = self
            .markdown
            .read(&configuration.root, &record.relative_path)?;
        self.repository
            .note_detail_projection(note_id, body)?
            .ok_or(NotesError::NoteNotFound)
    }

    pub(crate) fn save(&self, note_id: NoteId, body: &str) -> Result<NoteDetail, NotesError> {
        if body.len() > 4 * 1024 * 1024 || body.contains('\0') {
            return Err(NotesError::InvalidBody);
        }
        let configuration = self.configuration()?.ok_or(NotesError::NotConfigured)?;
        let record = self
            .repository
            .note_record(note_id)?
            .ok_or(NotesError::NoteNotFound)?;
        let projection = self
            .markdown
            .save(&configuration.root, &record.relative_path, body)?;
        self.repository.upsert_note(&projection)?;
        self.read(note_id)
    }

    pub(crate) fn open_note(&self, note_id: NoteId) -> Result<(), NotesError> {
        let configuration = self.configuration()?.ok_or(NotesError::NotConfigured)?;
        let record = self
            .repository
            .note_record(note_id)?
            .ok_or(NotesError::NoteNotFound)?;
        let path = self
            .markdown
            .resolve(&configuration.root, &record.relative_path)?;
        self.opener.open_path(&path)
    }

    pub(crate) fn open_root(&self) -> Result<(), NotesError> {
        let configuration = self.configuration()?.ok_or(NotesError::NotConfigured)?;
        self.opener.open_path(&configuration.root)
    }
}

fn validate_title(title: &str) -> Result<&str, NotesError> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 200 || title.chars().any(char::is_control) {
        return Err(NotesError::InvalidTitle);
    }
    Ok(title)
}
