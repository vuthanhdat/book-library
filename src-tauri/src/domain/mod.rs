mod book;
mod error;
mod id;
mod relative_path;

pub(crate) use book::{BookKind, BookStatus, ContentFingerprint};
pub(crate) use error::DomainError;
pub(crate) use id::{BookId, LibraryId};
pub(crate) use relative_path::RelativePath;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_domain_foundation_is_constructible() {
        let _library_id = LibraryId::new();
        let _book_id = BookId::new();
        let _kind = BookKind::PdfFile;
        let _other_kind = BookKind::ImageFolder;
        let _status = BookStatus::Available;
        let _unavailable = BookStatus::Unavailable;
        let _missing = BookStatus::Missing;
        let _unsupported = BookStatus::Unsupported;
        let _error = BookStatus::Error;
        let _fingerprint = ContentFingerprint::new("size:42:modified:7").unwrap();
        let _path = RelativePath::new("日本語/book.pdf").unwrap();
    }
}
