use std::{path::PathBuf, time::Duration};

use thiserror::Error;

use crate::domain::BookId;

use super::{DiscoveredBook, LibraryRepository, ThumbnailGenerator};

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum BookDetailError {
    #[error("the requested book does not exist")]
    BookNotFound,
    #[error("the reading status is invalid")]
    InvalidReadingStatus,
    #[error("the supplied tags are invalid")]
    InvalidTags,
    #[error("the book source is not available for a cover")]
    SourceUnavailable,
    #[error("the cover could not be generated")]
    CoverFailed,
    #[error("book detail persistence failed")]
    RepositoryFailed,
}

#[derive(Debug, Clone)]
pub(crate) struct LinkedBookNote {
    pub(crate) id: String,
    pub(crate) title: String,
}

#[derive(Debug, Clone)]
pub(crate) struct BookDetailRecord {
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
    pub(crate) reading_status: String,
    pub(crate) tags: Vec<String>,
    pub(crate) notes: Vec<LinkedBookNote>,
}

#[derive(Debug, Clone)]
pub(crate) struct BookThumbnailTarget {
    pub(crate) root: PathBuf,
    pub(crate) book: DiscoveredBook,
}

pub(crate) trait BookDetailRepository {
    fn book_detail(&self, book_id: BookId) -> Result<Option<BookDetailRecord>, BookDetailError>;
    fn update_book_detail(
        &self,
        book_id: BookId,
        reading_status: &str,
        tags: &[String],
    ) -> Result<bool, BookDetailError>;
    fn book_thumbnail_target(
        &self,
        book_id: BookId,
    ) -> Result<Option<BookThumbnailTarget>, BookDetailError>;
}

pub(crate) struct GetBookDetail<'a, Repository> {
    repository: &'a Repository,
}

impl<'a, Repository: BookDetailRepository> GetBookDetail<'a, Repository> {
    pub(crate) fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }

    pub(crate) fn execute(&self, book_id: BookId) -> Result<BookDetailRecord, BookDetailError> {
        self.repository
            .book_detail(book_id)?
            .ok_or(BookDetailError::BookNotFound)
    }
}

pub(crate) struct UpdateBookDetail<'a, Repository> {
    repository: &'a Repository,
}

impl<'a, Repository: BookDetailRepository> UpdateBookDetail<'a, Repository> {
    pub(crate) fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }

    pub(crate) fn execute(
        &self,
        book_id: BookId,
        reading_status: &str,
        tags: Vec<String>,
    ) -> Result<(), BookDetailError> {
        if !matches!(reading_status, "unread" | "reading" | "read") {
            return Err(BookDetailError::InvalidReadingStatus);
        }
        let mut normalized = tags
            .into_iter()
            .map(|tag| tag.trim().trim_start_matches('#').to_owned())
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>();
        normalized.sort();
        normalized.dedup();
        if normalized.len() > 100
            || normalized
                .iter()
                .any(|tag| tag.chars().count() > 64 || tag.chars().any(char::is_whitespace))
        {
            return Err(BookDetailError::InvalidTags);
        }
        if !self
            .repository
            .update_book_detail(book_id, reading_status, &normalized)?
        {
            return Err(BookDetailError::BookNotFound);
        }
        Ok(())
    }
}

pub(crate) struct ForceBookCover<'a, Repository, Generator> {
    repository: &'a Repository,
    generator: &'a Generator,
}

impl<'a, Repository, Generator> ForceBookCover<'a, Repository, Generator>
where
    Repository: BookDetailRepository + LibraryRepository,
    Generator: ThumbnailGenerator,
{
    pub(crate) fn new(repository: &'a Repository, generator: &'a Generator) -> Self {
        Self {
            repository,
            generator,
        }
    }

    pub(crate) fn execute(&self, book_id: BookId) -> Result<(), BookDetailError> {
        let target = self
            .repository
            .book_thumbnail_target(book_id)?
            .ok_or(BookDetailError::BookNotFound)?;
        if !matches!(target.book.status.as_str(), "available" | "unavailable") {
            return Err(BookDetailError::SourceUnavailable);
        }
        let outcome = self
            .generator
            .generate_with_timeout(&target.root, book_id, &target.book, Duration::from_secs(30))
            .map_err(|_| BookDetailError::CoverFailed)?;
        self.repository
            .save_thumbnail(book_id, &outcome)
            .map_err(|_| BookDetailError::RepositoryFailed)
    }
}
