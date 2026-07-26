use std::fmt;

use super::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RelativePath(String);

impl RelativePath {
    pub(crate) fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(DomainError::EmptyRelativePath);
        }

        let normalized = value.replace('\\', "/");
        if Self::looks_absolute(&normalized) {
            return Err(DomainError::AbsolutePath);
        }

        let mut segments = Vec::new();
        for segment in normalized.split('/') {
            match segment {
                "" => return Err(DomainError::EmptyPathSegment),
                "." => {}
                ".." => {
                    if segments.pop().is_none() {
                        return Err(DomainError::PathTraversal);
                    }
                }
                safe => segments.push(safe),
            }
        }

        if segments.is_empty() {
            return Err(DomainError::EmptyRelativePath);
        }

        Ok(Self(segments.join("/")))
    }

    fn looks_absolute(value: &str) -> bool {
        value.starts_with('/') || value.starts_with("//") || value.as_bytes().get(1) == Some(&b':')
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RelativePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_windows_separators_and_dot_segments() {
        let path = RelativePath::new(r"Series\.\Volume 01\book.pdf").unwrap();
        assert_eq!(path.as_str(), "Series/Volume 01/book.pdf");
    }

    #[test]
    fn normalizes_safe_parent_segments() {
        let path = RelativePath::new("Series/Drafts/../book.pdf").unwrap();
        assert_eq!(path.as_str(), "Series/book.pdf");
    }

    #[test]
    fn preserves_unicode_spelling_and_case() {
        let path = RelativePath::new("日本語/Ärger/Book.PDF").unwrap();
        assert_eq!(path.as_str(), "日本語/Ärger/Book.PDF");
        assert_ne!(path, RelativePath::new("日本語/ärger/book.pdf").unwrap());
    }

    #[test]
    fn rejects_absolute_and_escaping_paths() {
        for invalid in [
            "C:/Books/book.pdf",
            "C:\\Books\\book.pdf",
            "//server/share/book.pdf",
            "\\\\server\\share\\book.pdf",
            "/books/book.pdf",
            "\\books\\book.pdf",
            "../book.pdf",
            "series/../../book.pdf",
        ] {
            assert!(RelativePath::new(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn defines_empty_dot_and_empty_segment_behavior() {
        assert_eq!(RelativePath::new(""), Err(DomainError::EmptyRelativePath));
        assert_eq!(RelativePath::new("."), Err(DomainError::EmptyRelativePath));
        assert_eq!(
            RelativePath::new("series//book.pdf"),
            Err(DomainError::EmptyPathSegment)
        );
    }
}
