use super::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BookKind {
    PdfFile,
    ImageFolder,
}

impl BookKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::PdfFile => "pdf_file",
            Self::ImageFolder => "image_folder",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BookStatus {
    Available,
    Unavailable,
    Missing,
    Unsupported,
    Error,
}

impl BookStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unavailable => "unavailable",
            Self::Missing => "missing",
            Self::Unsupported => "unsupported",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ContentFingerprint(String);

impl ContentFingerprint {
    pub(crate) fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.is_empty() {
            return Err(DomainError::EmptyFingerprint);
        }
        Ok(Self(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_fingerprint() {
        assert_eq!(
            ContentFingerprint::new(""),
            Err(DomainError::EmptyFingerprint)
        );
    }
}
