use std::{
    collections::VecDeque,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::Read,
    path::Path,
    sync::mpsc,
    thread,
    time::Duration,
    time::UNIX_EPOCH,
};

use crate::{
    application::{
        DiscoveredBook, LibraryError, LibraryScanner, ScanIssue, ScanProgress, ScanResult,
    },
    domain::{BookKind, BookStatus, ContentFingerprint, RelativePath},
};

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];
const DIRECTORY_BATCH_SIZE: usize = 8;
const DIRECTORY_READ_TIMEOUT: Duration = Duration::from_millis(500);
const PDF_PROBE_TIMEOUT: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy)]
struct CandidateMetadata {
    size: u64,
    modified_at_ms: Option<i64>,
    unavailable: bool,
}

#[derive(Debug)]
struct EntryInfo {
    path: std::path::PathBuf,
    is_directory: bool,
    is_file: bool,
    is_symlink: bool,
}

pub(crate) struct FilesystemScanner;

impl FilesystemScanner {
    pub(crate) fn new() -> Self {
        Self
    }

    fn is_hidden_or_system(path: &Path) -> bool {
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy())
            .unwrap_or_default();
        name.starts_with('.')
            || name.eq_ignore_ascii_case("$RECYCLE.BIN")
            || name.eq_ignore_ascii_case("System Volume Information")
    }

    fn relative_path(root: &Path, path: &Path) -> Result<RelativePath, LibraryError> {
        let relative = path
            .strip_prefix(root)
            .map_err(|_| LibraryError::ScanFailed)?;
        let text = relative.to_str().ok_or(LibraryError::ScanFailed)?;
        RelativePath::new(text).map_err(|_| LibraryError::ScanFailed)
    }

    fn path_key(path: &RelativePath) -> String {
        if cfg!(target_os = "windows") {
            path.as_str().to_lowercase()
        } else {
            path.as_str().to_owned()
        }
    }

    fn modified_ms(metadata: &fs::Metadata) -> Option<i64> {
        metadata
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| i64::try_from(duration.as_millis()).ok())
    }

    #[cfg(target_os = "windows")]
    fn candidate_metadata(path: &Path) -> Option<CandidateMetadata> {
        use std::{ffi::c_void, os::windows::ffi::OsStrExt};

        #[repr(C)]
        struct FileTime {
            low: u32,
            high: u32,
        }

        #[repr(C)]
        struct FileAttributeData {
            attributes: u32,
            creation_time: FileTime,
            last_access_time: FileTime,
            last_write_time: FileTime,
            size_high: u32,
            size_low: u32,
        }

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetFileAttributesExW(
                file_name: *const u16,
                info_level: i32,
                file_information: *mut c_void,
            ) -> i32;
        }

        const FILE_ATTRIBUTE_OFFLINE: u32 = 0x0000_1000;
        const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x0004_0000;
        const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;
        const WINDOWS_TO_UNIX_EPOCH_MS: u64 = 11_644_473_600_000;

        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut data = FileAttributeData {
            attributes: 0,
            creation_time: FileTime { low: 0, high: 0 },
            last_access_time: FileTime { low: 0, high: 0 },
            last_write_time: FileTime { low: 0, high: 0 },
            size_high: 0,
            size_low: 0,
        };
        // SAFETY: `wide` is null-terminated and `data` is a correctly sized,
        // writable WIN32_FILE_ATTRIBUTE_DATA-compatible buffer.
        let succeeded =
            unsafe { GetFileAttributesExW(wide.as_ptr(), 0, (&raw mut data).cast::<c_void>()) };
        if succeeded == 0 {
            return None;
        }
        let file_time =
            (u64::from(data.last_write_time.high) << 32) | u64::from(data.last_write_time.low);
        let modified_at_ms = file_time
            .checked_div(10_000)
            .and_then(|value| value.checked_sub(WINDOWS_TO_UNIX_EPOCH_MS))
            .and_then(|value| i64::try_from(value).ok());
        Some(CandidateMetadata {
            size: (u64::from(data.size_high) << 32) | u64::from(data.size_low),
            modified_at_ms,
            unavailable: data.attributes
                & (FILE_ATTRIBUTE_OFFLINE
                    | FILE_ATTRIBUTE_RECALL_ON_OPEN
                    | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS)
                != 0,
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn candidate_metadata(path: &Path) -> Option<CandidateMetadata> {
        let metadata = fs::metadata(path).ok()?;
        Some(CandidateMetadata {
            size: metadata.len(),
            modified_at_ms: Self::modified_ms(&metadata),
            unavailable: false,
        })
    }

    fn title_from_path(path: &Path, kind: BookKind) -> String {
        let source = match kind {
            BookKind::PdfFile => path.file_stem(),
            BookKind::ImageFolder => {
                let folder_name = path.file_name();
                if folder_name.is_some_and(|name| {
                    name.to_str()
                        .is_some_and(|value| value.eq_ignore_ascii_case("pages"))
                }) {
                    path.parent().and_then(Path::file_name).or(folder_name)
                } else {
                    folder_name
                }
            }
        };
        source
            .and_then(|name| name.to_str())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("Untitled")
            .to_owned()
    }

    fn is_pdf(path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
    }

    fn is_image(path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                IMAGE_EXTENSIONS
                    .iter()
                    .any(|supported| extension.eq_ignore_ascii_case(supported))
            })
    }

    fn has_pdf_signature(path: &Path) -> std::io::Result<bool> {
        let mut file = File::open(path)?;
        let mut signature = [0_u8; 5];
        Ok(file.read_exact(&mut signature).is_ok() && &signature == b"%PDF-")
    }

    fn read_directory(path: &Path) -> Result<Vec<EntryInfo>, &'static str> {
        let owned_path = path.to_path_buf();
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let result = fs::read_dir(&owned_path).map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| {
                        let file_type = entry.file_type().ok()?;
                        Some(EntryInfo {
                            path: entry.path(),
                            is_directory: file_type.is_dir(),
                            is_file: file_type.is_file(),
                            is_symlink: file_type.is_symlink(),
                        })
                    })
                    .collect()
            });
            let _ = sender.send(result);
        });
        match receiver.recv_timeout(DIRECTORY_READ_TIMEOUT) {
            Ok(Ok(entries)) => Ok(entries),
            Ok(Err(_)) => Err("unreadable_directory"),
            Err(_) => Err("directory_unavailable"),
        }
    }

    fn probe_pdf(path: &Path) -> Result<(bool, u64, Option<i64>), &'static str> {
        let owned_path = path.to_path_buf();
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let result = (|| {
                let valid_signature = Self::has_pdf_signature(&owned_path).ok()?;
                let metadata = fs::metadata(&owned_path).ok()?;
                Some((
                    valid_signature,
                    metadata.len(),
                    Self::modified_ms(&metadata),
                ))
            })();
            let _ = sender.send(result);
        });
        match receiver.recv_timeout(PDF_PROBE_TIMEOUT) {
            Ok(Some(probe)) => Ok(probe),
            Ok(None) => Err("unreadable_file"),
            Err(_) => Err("file_unavailable"),
        }
    }

    fn contained(root: &Path, candidate: &Path) -> bool {
        // WalkDir is rooted at the canonical authorized root and links are never
        // followed. A lexical descendant check avoids hydrating Drive placeholders.
        candidate.starts_with(root)
    }

    fn pdf_candidate(root: &Path, path: &Path) -> Result<Option<DiscoveredBook>, ScanIssue> {
        if !Self::contained(root, path) {
            return Err(ScanIssue {
                relative_path: None,
                severity: "warning",
                code: "root_escape",
                message: "An entry resolved outside the authorized library root.",
            });
        }
        if Self::candidate_metadata(path).is_some_and(|metadata| metadata.unavailable) {
            return Err(ScanIssue {
                relative_path: Self::relative_path(root, path)
                    .ok()
                    .map(|value| value.to_string()),
                severity: "warning",
                code: "file_unavailable",
                message: "A PDF candidate is currently available online only.",
            });
        }
        let (valid_signature, size, modified_at_ms) = match Self::probe_pdf(path) {
            Ok(value) => value,
            Err(code) => {
                return Err(ScanIssue {
                    relative_path: Self::relative_path(root, path)
                        .ok()
                        .map(|value| value.to_string()),
                    severity: "warning",
                    code,
                    message: "A PDF candidate is temporarily unavailable or unreadable.",
                });
            }
        };
        if !valid_signature {
            return Err(ScanIssue {
                relative_path: Self::relative_path(root, path)
                    .ok()
                    .map(|value| value.to_string()),
                severity: "warning",
                code: "invalid_pdf_signature",
                message: "A .pdf file did not contain a PDF signature.",
            });
        }
        let relative_path = Self::relative_path(root, path).map_err(|_| ScanIssue {
            relative_path: None,
            severity: "warning",
            code: "invalid_relative_path",
            message: "A candidate path could not be represented safely.",
        })?;
        let fingerprint_text = format!("pdf:{}:{}", size, modified_at_ms.unwrap_or_default());
        let fingerprint = ContentFingerprint::new(fingerprint_text).map_err(|_| ScanIssue {
            relative_path: Some(relative_path.to_string()),
            severity: "warning",
            code: "fingerprint_failed",
            message: "A candidate fingerprint could not be created.",
        })?;

        Ok(Some(DiscoveredBook {
            kind: BookKind::PdfFile,
            status: BookStatus::Available,
            path_key: Self::path_key(&relative_path),
            title: Self::title_from_path(path, BookKind::PdfFile),
            relative_path,
            fingerprint,
            size_bytes: Some(size),
            modified_at_ms,
            page_count: None,
            image_pages: Vec::new(),
        }))
    }

    fn unavailable_pdf_candidate(root: &Path, path: &Path) -> Result<DiscoveredBook, LibraryError> {
        let relative_path = Self::relative_path(root, path)?;
        let fingerprint =
            ContentFingerprint::new(format!("pdf-unavailable:{}", relative_path.as_str()))
                .map_err(|_| LibraryError::ScanFailed)?;
        Ok(DiscoveredBook {
            kind: BookKind::PdfFile,
            status: BookStatus::Unavailable,
            path_key: Self::path_key(&relative_path),
            title: Self::title_from_path(path, BookKind::PdfFile),
            relative_path,
            fingerprint,
            size_bytes: None,
            modified_at_ms: None,
            page_count: None,
            image_pages: Vec::new(),
        })
    }

    fn image_folder_candidate(
        root: &Path,
        folder: &Path,
        mut image_paths: Vec<std::path::PathBuf>,
    ) -> Result<Option<DiscoveredBook>, ScanIssue> {
        if folder == root || !Self::contained(root, folder) {
            return Ok(None);
        }
        if image_paths.len() < 2 {
            return Ok(None);
        }

        image_paths.sort_by(|left, right| {
            let left_name = left
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            let right_name = right
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            natord::compare(&left_name.to_lowercase(), &right_name.to_lowercase())
                .then_with(|| left_name.cmp(right_name))
        });

        let mut total_size = 0_u64;
        let mut latest_modified = None;
        let mut pages = Vec::with_capacity(image_paths.len());
        let mut all_available = true;
        let mut names_hasher = DefaultHasher::new();
        for image in &image_paths {
            let relative = match Self::relative_path(root, image) {
                Ok(value) => value,
                Err(_) => {
                    all_available = false;
                    continue;
                }
            };
            relative.as_str().hash(&mut names_hasher);
            pages.push(relative);
            match Self::candidate_metadata(image) {
                Some(metadata) if !metadata.unavailable => {
                    total_size = total_size.saturating_add(metadata.size);
                    latest_modified = latest_modified.max(metadata.modified_at_ms);
                }
                _ => {
                    all_available = false;
                }
            }
        }
        if pages.len() < 2 {
            return Ok(None);
        }

        let relative_path = Self::relative_path(root, folder).map_err(|_| ScanIssue {
            relative_path: None,
            severity: "warning",
            code: "invalid_relative_path",
            message: "An image-folder path could not be represented safely.",
        })?;
        let fingerprint = ContentFingerprint::new(format!(
            "images:{}:{}:{}:{:016x}",
            pages.len(),
            total_size,
            latest_modified.unwrap_or_default(),
            names_hasher.finish()
        ))
        .map_err(|_| ScanIssue {
            relative_path: Some(relative_path.to_string()),
            severity: "warning",
            code: "fingerprint_failed",
            message: "An image-folder fingerprint could not be created.",
        })?;

        Ok(Some(DiscoveredBook {
            kind: BookKind::ImageFolder,
            status: if all_available {
                BookStatus::Available
            } else {
                BookStatus::Unavailable
            },
            path_key: Self::path_key(&relative_path),
            title: Self::title_from_path(folder, BookKind::ImageFolder),
            relative_path,
            fingerprint,
            size_bytes: Some(total_size),
            modified_at_ms: latest_modified,
            page_count: u32::try_from(pages.len()).ok(),
            image_pages: pages,
        }))
    }
}

impl LibraryScanner for FilesystemScanner {
    fn scan(
        &self,
        root: &Path,
        cancellation: &crate::application::CancellationToken,
        progress: &mut dyn FnMut(ScanProgress),
    ) -> Result<ScanResult, LibraryError> {
        let root = root.canonicalize().map_err(|_| LibraryError::RootInvalid)?;
        let mut books = Vec::new();
        let mut issues = Vec::new();
        let mut visited_entries = 0_u64;
        let mut directories = VecDeque::from([root.clone()]);

        while !directories.is_empty() {
            if cancellation.is_cancelled() {
                return Ok(ScanResult {
                    books,
                    issues,
                    cancelled: true,
                });
            }
            let batch = (0..DIRECTORY_BATCH_SIZE)
                .filter_map(|_| directories.pop_front())
                .collect::<Vec<_>>();
            let read_results = thread::scope(|scope| {
                let workers = batch
                    .iter()
                    .map(|directory| scope.spawn(|| Self::read_directory(directory)))
                    .collect::<Vec<_>>();
                workers
                    .into_iter()
                    .map(|worker| worker.join().unwrap_or(Err("directory_unavailable")))
                    .collect::<Vec<_>>()
            });

            for (directory, read_result) in batch.into_iter().zip(read_results) {
                let entries = match read_result {
                    Ok(value) => value,
                    Err(code) => {
                        let is_root = directory == root;
                        if is_root {
                            return Err(LibraryError::RootUnreadable);
                        }
                        issues.push(ScanIssue {
                            relative_path: Self::relative_path(&root, &directory)
                                .ok()
                                .map(|value| value.to_string()),
                            severity: "warning",
                            code,
                            message: "A directory is temporarily unavailable or unreadable.",
                        });
                        continue;
                    }
                };
                let image_paths = entries
                    .iter()
                    .filter(|entry| {
                        entry.is_file && !entry.is_symlink && Self::is_image(&entry.path)
                    })
                    .map(|entry| entry.path.clone())
                    .collect();
                match Self::image_folder_candidate(&root, &directory, image_paths) {
                    Ok(Some(book)) => books.push(book),
                    Ok(None) => {}
                    Err(issue) => issues.push(issue),
                }

                for entry in entries {
                    if cancellation.is_cancelled() {
                        return Ok(ScanResult {
                            books,
                            issues,
                            cancelled: true,
                        });
                    }
                    if Self::is_hidden_or_system(&entry.path) {
                        continue;
                    }
                    visited_entries += 1;
                    if entry.is_symlink {
                        issues.push(ScanIssue {
                            relative_path: Self::relative_path(&root, &entry.path)
                                .ok()
                                .map(|value| value.to_string()),
                            severity: "warning",
                            code: "symlink_skipped",
                            message: "A symbolic link was skipped.",
                        });
                        continue;
                    }
                    if entry.is_directory {
                        directories.push_back(entry.path.clone());
                    } else if entry.is_file && Self::is_pdf(&entry.path) {
                        match Self::pdf_candidate(&root, &entry.path) {
                            Ok(Some(book)) => books.push(book),
                            Ok(None) => {}
                            Err(issue) => {
                                if matches!(issue.code, "file_unavailable" | "unreadable_file")
                                    && let Ok(book) =
                                        Self::unavailable_pdf_candidate(&root, &entry.path)
                                {
                                    books.push(book);
                                }
                                issues.push(issue);
                            }
                        }
                    }

                    if visited_entries == 1 || visited_entries.is_multiple_of(100) {
                        progress(ScanProgress {
                            visited_entries,
                            discovered_books: books.len() as u64,
                            current_relative_path: Self::relative_path(&root, &entry.path)
                                .ok()
                                .map(|value| value.to_string()),
                        });
                    }
                }
            }
        }
        progress(ScanProgress {
            visited_entries,
            discovered_books: books.len() as u64,
            current_relative_path: None,
        });
        Ok(ScanResult {
            books,
            issues,
            cancelled: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::CancellationToken;
    use tempfile::TempDir;

    fn write_pdf(path: &Path) {
        fs::write(path, b"%PDF-1.4\n%%EOF").unwrap();
    }

    #[test]
    fn discovers_pdfs_and_naturally_sorted_direct_image_pages() {
        let root = TempDir::new().unwrap();
        write_pdf(&root.path().join("Book.PDF"));
        let images = root.path().join("Volume");
        fs::create_dir(&images).unwrap();
        for name in ["page10.jpg", "page2.jpg", "page1.jpg"] {
            fs::write(images.join(name), b"image").unwrap();
        }

        let mut progress = |_| {};
        let result = FilesystemScanner::new()
            .scan(root.path(), &CancellationToken::default(), &mut progress)
            .unwrap();

        assert_eq!(result.books.len(), 2);
        let image_book = result
            .books
            .iter()
            .find(|book| book.kind == BookKind::ImageFolder)
            .unwrap();
        let pages: Vec<_> = image_book
            .image_pages
            .iter()
            .map(|path| path.as_str())
            .collect();
        assert_eq!(
            pages,
            ["Volume/page1.jpg", "Volume/page2.jpg", "Volume/page10.jpg"]
        );
    }

    #[test]
    fn rejects_fake_pdfs_and_supports_cancellation() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("fake.pdf"), b"not a pdf").unwrap();
        let cancellation = CancellationToken::default();
        cancellation.cancel();

        let mut progress = |_| {};
        let cancelled = FilesystemScanner::new()
            .scan(root.path(), &cancellation, &mut progress)
            .unwrap();
        assert!(cancelled.cancelled);

        let valid = FilesystemScanner::new()
            .scan(root.path(), &CancellationToken::default(), &mut progress)
            .unwrap();
        assert!(valid.books.is_empty());
        assert_eq!(valid.issues[0].code, "invalid_pdf_signature");
    }

    #[test]
    fn unavailable_pdf_identity_does_not_require_reading_source_bytes() {
        let root = TempDir::new().unwrap();
        let path = root.path().join("Cloud only.pdf");

        let candidate = FilesystemScanner::unavailable_pdf_candidate(root.path(), &path).unwrap();

        assert_eq!(candidate.status, BookStatus::Unavailable);
        assert_eq!(candidate.relative_path.as_str(), "Cloud only.pdf");
        assert_eq!(candidate.size_bytes, None);
        assert_eq!(candidate.modified_at_ms, None);
    }

    #[test]
    fn pages_wrapper_uses_the_unicode_parent_folder_as_its_title() {
        let root = TempDir::new().unwrap();
        let expected_title = "「私」が主語になる人生のつくり方 脳の自動操縦から抜け出す7つの講義";
        let pages = root.path().join(expected_title).join("pages");
        fs::create_dir_all(&pages).unwrap();
        fs::write(pages.join("page-0001.png"), b"image").unwrap();
        fs::write(pages.join("page-0002.png"), b"image").unwrap();

        let mut progress = |_| {};
        let result = FilesystemScanner::new()
            .scan(root.path(), &CancellationToken::default(), &mut progress)
            .unwrap();

        assert_eq!(result.books.len(), 1);
        assert_eq!(result.books[0].title, expected_title);
        assert_eq!(
            result.books[0].relative_path.as_str(),
            format!("{expected_title}/pages")
        );
    }
}
