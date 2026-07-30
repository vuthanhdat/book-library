use std::path::PathBuf;

use thiserror::Error;

use crate::domain::RelativePath;

use super::MarkdownNotes;

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum SearchError {
    #[error("the search query is invalid")]
    InvalidQuery,
    #[error("the search index is unavailable")]
    IndexUnavailable,
    #[error("the search index could not be rebuilt")]
    RebuildFailed,
}

#[derive(Debug, Clone)]
pub(crate) struct SearchDocument {
    pub(crate) source_kind: String,
    pub(crate) source_id: String,
    pub(crate) scope: String,
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) relative_path: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct SearchResultItem {
    pub(crate) source_kind: String,
    pub(crate) source_id: String,
    pub(crate) scope: String,
    pub(crate) title: String,
    pub(crate) snippet: String,
    pub(crate) relative_path: String,
    pub(crate) status: String,
    pub(crate) rank: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct SearchRebuildSummary {
    pub(crate) indexed: u64,
    pub(crate) failed: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct SearchDiagnostics {
    pub(crate) documents: u64,
    pub(crate) failed_jobs: u64,
    pub(crate) last_rebuild_at: Option<String>,
}

pub(crate) trait SearchRepository {
    fn enqueue_search_rebuild(&self) -> Result<(), SearchError>;
    fn canonical_search_documents(
        &self,
    ) -> Result<(Vec<SearchDocument>, Option<PathBuf>), SearchError>;
    fn replace_search_documents(
        &self,
        documents: &[SearchDocument],
        failed: u64,
    ) -> Result<SearchRebuildSummary, SearchError>;
    fn search(
        &self,
        query: &str,
        scope: Option<&str>,
        limit: u32,
    ) -> Result<Vec<SearchResultItem>, SearchError>;
    fn search_diagnostics(&self) -> Result<SearchDiagnostics, SearchError>;
}

pub(crate) struct SearchLibrary<'a, Repository, Markdown> {
    repository: &'a Repository,
    markdown: &'a Markdown,
}

impl<'a, Repository, Markdown> SearchLibrary<'a, Repository, Markdown>
where
    Repository: SearchRepository,
    Markdown: MarkdownNotes,
{
    pub(crate) fn new(repository: &'a Repository, markdown: &'a Markdown) -> Self {
        Self {
            repository,
            markdown,
        }
    }

    pub(crate) fn rebuild(&self) -> Result<SearchRebuildSummary, SearchError> {
        let (mut documents, notes_root) = self.repository.canonical_search_documents()?;
        let mut failed = 0;
        if let Some(root) = notes_root {
            for document in documents
                .iter_mut()
                .filter(|document| document.source_kind == "note" && document.scope == "notes")
            {
                let Ok(relative_path) = RelativePath::new(&document.relative_path) else {
                    failed += 1;
                    continue;
                };
                match self.markdown.read(&root, &relative_path) {
                    Ok(body) => document.body = body,
                    Err(_) => failed += 1,
                }
            }
        }
        self.repository.replace_search_documents(&documents, failed)
    }

    pub(crate) fn execute(
        &self,
        query: &str,
        scope: Option<&str>,
    ) -> Result<Vec<SearchResultItem>, SearchError> {
        let query = query.trim();
        if query.is_empty() || query.chars().count() > 500 || query.contains('\0') {
            return Err(SearchError::InvalidQuery);
        }
        let scope = match scope {
            None | Some("all") => None,
            Some("books" | "notes" | "tags" | "headings" | "ocr") => scope,
            Some(_) => return Err(SearchError::InvalidQuery),
        };
        self.repository.search(query, scope, 100)
    }

    pub(crate) fn diagnostics(&self) -> Result<SearchDiagnostics, SearchError> {
        self.repository.search_diagnostics()
    }
}
