use thiserror::Error;

use crate::domain::BookId;

const MAX_DISPLAY_TITLE_CHARACTERS: usize = 512;

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum BookMetadataError {
    #[error("the requested book identifier is invalid")]
    InvalidBookId,
    #[error("the requested book does not exist")]
    BookNotFound,
    #[error("the display title is invalid")]
    InvalidTitle,
    #[error("the catalog metadata could not be saved")]
    RepositoryFailed,
}

pub(crate) trait BookMetadataRepository {
    fn update_display_title(&self, book_id: BookId, title: &str)
    -> Result<bool, BookMetadataError>;
}

pub(crate) struct UpdateBookDisplayTitle<'a, Repository> {
    repository: &'a Repository,
}

impl<'a, Repository: BookMetadataRepository> UpdateBookDisplayTitle<'a, Repository> {
    pub(crate) fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }

    pub(crate) fn execute(
        &self,
        book_id: BookId,
        title: &str,
    ) -> Result<String, BookMetadataError> {
        let title = title.trim();
        if title.is_empty()
            || title.chars().count() > MAX_DISPLAY_TITLE_CHARACTERS
            || title.chars().any(char::is_control)
        {
            return Err(BookMetadataError::InvalidTitle);
        }
        if !self.repository.update_display_title(book_id, title)? {
            return Err(BookMetadataError::BookNotFound);
        }
        Ok(title.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct RecordingRepository {
        saved: Mutex<Vec<String>>,
    }

    impl BookMetadataRepository for RecordingRepository {
        fn update_display_title(
            &self,
            _book_id: BookId,
            title: &str,
        ) -> Result<bool, BookMetadataError> {
            self.saved.lock().unwrap().push(title.to_owned());
            Ok(true)
        }
    }

    #[test]
    fn trims_and_preserves_a_valid_unicode_title() {
        let repository = RecordingRepository::default();
        let title = UpdateBookDisplayTitle::new(&repository)
            .execute(
                BookId::new(),
                "  「私」が主語になる人生のつくり方 脳の自動操縦から抜け出す7つの講義  ",
            )
            .unwrap();

        assert_eq!(
            title,
            "「私」が主語になる人生のつくり方 脳の自動操縦から抜け出す7つの講義"
        );
        assert_eq!(repository.saved.lock().unwrap().as_slice(), [title]);
    }

    #[test]
    fn rejects_empty_control_character_and_oversized_titles() {
        let repository = RecordingRepository::default();
        let use_case = UpdateBookDisplayTitle::new(&repository);

        for invalid in ["   ".to_owned(), "line\nbreak".to_owned(), "a".repeat(513)] {
            assert_eq!(
                use_case.execute(BookId::new(), &invalid),
                Err(BookMetadataError::InvalidTitle)
            );
        }
        assert!(repository.saved.lock().unwrap().is_empty());
    }
}
