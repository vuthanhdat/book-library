use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::{BookId, BookKind, RelativePath};

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum SourceLocationError {
    #[error("the requested book identifier is invalid")]
    InvalidBookId,
    #[error("the requested book does not exist")]
    BookNotFound,
    #[error("the requested book source is missing")]
    SourceMissing,
    #[error("the requested book source is unavailable")]
    SourceUnavailable,
    #[error("the requested source path is invalid")]
    InvalidSourcePath,
    #[error("the catalog source could not be read")]
    RepositoryFailed,
    #[error("the operating system file manager could not be started")]
    LaunchFailed,
}

#[derive(Debug, Clone)]
pub(crate) struct BookSourceLocation {
    pub(crate) library_root: PathBuf,
    pub(crate) kind: BookKind,
    pub(crate) relative_path: RelativePath,
    pub(crate) status: String,
}

pub(crate) trait BookLocationRepository {
    fn book_source_location(
        &self,
        book_id: BookId,
    ) -> Result<Option<BookSourceLocation>, SourceLocationError>;
}

pub(crate) trait FileManager {
    fn open_directory(&self, directory: &Path) -> Result<(), SourceLocationError>;
}

pub(crate) struct OpenBookLocation<'a, Repository, Manager> {
    repository: &'a Repository,
    file_manager: &'a Manager,
}

impl<'a, Repository, Manager> OpenBookLocation<'a, Repository, Manager>
where
    Repository: BookLocationRepository,
    Manager: FileManager,
{
    pub(crate) fn new(repository: &'a Repository, file_manager: &'a Manager) -> Self {
        Self {
            repository,
            file_manager,
        }
    }

    pub(crate) fn execute(&self, book_id: BookId) -> Result<(), SourceLocationError> {
        let source = self
            .repository
            .book_source_location(book_id)?
            .ok_or(SourceLocationError::BookNotFound)?;
        match source.status.as_str() {
            "missing" => return Err(SourceLocationError::SourceMissing),
            "unsupported" | "error" => return Err(SourceLocationError::SourceUnavailable),
            "available" | "unavailable" => {}
            _ => return Err(SourceLocationError::SourceUnavailable),
        }

        let root = source
            .library_root
            .canonicalize()
            .map_err(|_| SourceLocationError::SourceUnavailable)?;
        let source_path = root.join(source.relative_path.as_str());
        let requested_directory = match source.kind {
            BookKind::PdfFile => source_path
                .parent()
                .ok_or(SourceLocationError::InvalidSourcePath)?,
            BookKind::ImageFolder => source_path.as_path(),
        };
        let directory = requested_directory
            .canonicalize()
            .map_err(|_| SourceLocationError::SourceUnavailable)?;
        if !directory.starts_with(&root) || !directory.is_dir() {
            return Err(SourceLocationError::InvalidSourcePath);
        }

        self.file_manager.open_directory(&directory)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use tempfile::TempDir;

    use super::*;

    struct FakeRepository {
        source: Option<BookSourceLocation>,
    }

    impl BookLocationRepository for FakeRepository {
        fn book_source_location(
            &self,
            _book_id: BookId,
        ) -> Result<Option<BookSourceLocation>, SourceLocationError> {
            Ok(self.source.clone())
        }
    }

    #[derive(Default)]
    struct RecordingFileManager {
        opened: Mutex<Vec<PathBuf>>,
    }

    impl FileManager for RecordingFileManager {
        fn open_directory(&self, directory: &Path) -> Result<(), SourceLocationError> {
            self.opened.lock().unwrap().push(directory.to_path_buf());
            Ok(())
        }
    }

    #[test]
    fn pdf_opens_its_parent_and_image_book_opens_its_own_folder() {
        let library = TempDir::new().unwrap();
        let pdf_parent = library.path().join("PDF");
        let image_folder = library.path().join("Images").join("Book").join("pages");
        std::fs::create_dir_all(&pdf_parent).unwrap();
        std::fs::create_dir_all(&image_folder).unwrap();
        let manager = RecordingFileManager::default();

        for (kind, relative_path) in [
            (BookKind::PdfFile, "PDF/Book.pdf"),
            (BookKind::ImageFolder, "Images/Book/pages"),
        ] {
            OpenBookLocation::new(
                &FakeRepository {
                    source: Some(BookSourceLocation {
                        library_root: library.path().to_path_buf(),
                        kind,
                        relative_path: RelativePath::new(relative_path).unwrap(),
                        status: "available".to_owned(),
                    }),
                },
                &manager,
            )
            .execute(BookId::new())
            .unwrap();
        }

        let opened = manager.opened.lock().unwrap();
        assert_eq!(opened[0], pdf_parent.canonicalize().unwrap());
        assert_eq!(opened[1], image_folder.canonicalize().unwrap());
    }

    #[test]
    fn missing_and_unknown_books_do_not_launch_the_file_manager() {
        let manager = RecordingFileManager::default();
        let missing = OpenBookLocation::new(
            &FakeRepository {
                source: Some(BookSourceLocation {
                    library_root: PathBuf::from("unused"),
                    kind: BookKind::PdfFile,
                    relative_path: RelativePath::new("Book.pdf").unwrap(),
                    status: "missing".to_owned(),
                }),
            },
            &manager,
        )
        .execute(BookId::new());
        let unknown = OpenBookLocation::new(&FakeRepository { source: None }, &manager)
            .execute(BookId::new());

        assert_eq!(missing, Err(SourceLocationError::SourceMissing));
        assert_eq!(unknown, Err(SourceLocationError::BookNotFound));
        assert!(manager.opened.lock().unwrap().is_empty());
    }
}
