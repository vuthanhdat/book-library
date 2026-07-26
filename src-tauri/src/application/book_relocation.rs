use std::{fs, io::Read, path::Path};

use thiserror::Error;

use crate::domain::{BookId, BookKind, RelativePath};

use super::BookSourceLocation;

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum BookRelocationError {
    #[error("the selected book does not exist")]
    BookNotFound,
    #[error("the selected replacement has the wrong type")]
    WrongSourceType,
    #[error("the selected replacement is outside the configured library")]
    OutsideLibrary,
    #[error("the selected replacement path is invalid")]
    InvalidPath,
    #[error("another catalog book already uses the selected path")]
    PathConflict,
    #[error("the catalog could not be updated")]
    RepositoryFailed,
}

pub(crate) trait BookRelocationRepository {
    fn relocation_source(
        &self,
        book_id: BookId,
    ) -> Result<Option<BookSourceLocation>, BookRelocationError>;
    fn update_source_path(
        &self,
        book_id: BookId,
        relative_path: &RelativePath,
        path_key: &str,
    ) -> Result<(), BookRelocationError>;
}

pub(crate) struct RelinkMissingBook<'a, Repository> {
    repository: &'a Repository,
}

impl<'a, Repository: BookRelocationRepository> RelinkMissingBook<'a, Repository> {
    pub(crate) fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }

    pub(crate) fn execute(
        &self,
        book_id: BookId,
        selected_path: impl AsRef<Path>,
    ) -> Result<RelativePath, BookRelocationError> {
        let source = self
            .repository
            .relocation_source(book_id)?
            .ok_or(BookRelocationError::BookNotFound)?;
        let root = source
            .library_root
            .canonicalize()
            .map_err(|_| BookRelocationError::InvalidPath)?;
        let selected = selected_path
            .as_ref()
            .canonicalize()
            .map_err(|_| BookRelocationError::InvalidPath)?;
        if !selected.starts_with(&root) {
            return Err(BookRelocationError::OutsideLibrary);
        }
        match source.kind {
            BookKind::PdfFile if !is_pdf(&selected) => {
                return Err(BookRelocationError::WrongSourceType);
            }
            BookKind::ImageFolder if !is_image_book_folder(&selected) => {
                return Err(BookRelocationError::WrongSourceType);
            }
            _ => {}
        }
        let relative = selected
            .strip_prefix(&root)
            .map_err(|_| BookRelocationError::OutsideLibrary)?;
        let relative_text = relative.to_str().ok_or(BookRelocationError::InvalidPath)?;
        let relative_path =
            RelativePath::new(relative_text).map_err(|_| BookRelocationError::InvalidPath)?;
        let path_key = if cfg!(target_os = "windows") {
            relative_path.as_str().to_lowercase()
        } else {
            relative_path.as_str().to_owned()
        };
        self.repository
            .update_source_path(book_id, &relative_path, &path_key)?;
        Ok(relative_path)
    }
}

fn is_pdf(path: &Path) -> bool {
    if !path.is_file()
        || !path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("pdf"))
    {
        return false;
    }
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut signature = [0_u8; 5];
    file.read_exact(&mut signature).is_ok() && &signature == b"%PDF-"
}

fn is_image_book_folder(path: &Path) -> bool {
    path.is_dir()
        && fs::read_dir(path).is_ok_and(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                entry.file_type().is_ok_and(|kind| kind.is_file())
                    && entry
                        .path()
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|extension| {
                            ["jpg", "jpeg", "png", "webp"]
                                .iter()
                                .any(|supported| extension.eq_ignore_ascii_case(supported))
                        })
            })
        })
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use tempfile::TempDir;

    use super::*;

    struct FakeRepository {
        source: BookSourceLocation,
        updated: Mutex<Option<RelativePath>>,
    }

    impl BookRelocationRepository for FakeRepository {
        fn relocation_source(
            &self,
            _book_id: BookId,
        ) -> Result<Option<BookSourceLocation>, BookRelocationError> {
            Ok(Some(self.source.clone()))
        }

        fn update_source_path(
            &self,
            _book_id: BookId,
            relative_path: &RelativePath,
            _path_key: &str,
        ) -> Result<(), BookRelocationError> {
            *self.updated.lock().unwrap() = Some(relative_path.clone());
            Ok(())
        }
    }

    #[test]
    fn relinks_only_matching_sources_inside_the_library() {
        let library = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let replacement = library.path().join("Moved").join("本.pdf");
        std::fs::create_dir_all(replacement.parent().unwrap()).unwrap();
        std::fs::write(&replacement, b"%PDF-1.7").unwrap();
        let repository = FakeRepository {
            source: BookSourceLocation {
                library_root: library.path().to_path_buf(),
                kind: BookKind::PdfFile,
                relative_path: RelativePath::new("Old/本.pdf").unwrap(),
                status: "missing".to_owned(),
            },
            updated: Mutex::new(None),
        };
        let use_case = RelinkMissingBook::new(&repository);

        assert_eq!(
            use_case.execute(BookId::new(), &replacement).unwrap(),
            RelativePath::new("Moved/本.pdf").unwrap()
        );
        assert_eq!(
            repository.updated.lock().unwrap().as_ref(),
            Some(&RelativePath::new("Moved/本.pdf").unwrap())
        );
        assert_eq!(
            use_case.execute(BookId::new(), outside.path()),
            Err(BookRelocationError::OutsideLibrary)
        );
    }
}
