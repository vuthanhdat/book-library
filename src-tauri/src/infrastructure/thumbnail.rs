use std::{
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, OnceLock, mpsc, mpsc::RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

use image::{DynamicImage, GenericImageView, ImageFormat};
use pdfium_render::prelude::{PdfRenderConfig, Pdfium};

use crate::{
    application::{
        DiscoveredBook, LibraryError, ThumbnailGenerator, ThumbnailOutcome, ThumbnailProgressStage,
    },
    domain::{BookId, BookKind},
};

const MAX_WIDTH: u32 = 320;
const MAX_HEIGHT: u32 = 448;
const GENERATION_TIMEOUT: Duration = Duration::from_secs(1);
static PDFIUM_RENDER_LOCK: Mutex<()> = Mutex::new(());
static PDFIUM: OnceLock<Option<Pdfium>> = OnceLock::new();

fn acquire_render_lock(lock: &Mutex<()>) -> MutexGuard<'_, ()> {
    lock.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn shared_pdfium(pdfium_directory: &Path) -> Result<&'static Pdfium, LibraryError> {
    PDFIUM
        .get_or_init(|| {
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
                pdfium_directory,
            ))
            .ok()
            .map(Pdfium::new)
        })
        .as_ref()
        .ok_or(LibraryError::ThumbnailFailed)
}

enum ThumbnailWorkerEvent {
    Progress(ThumbnailProgressStage),
    Finished(Result<ThumbnailOutcome, LibraryError>),
}

pub(crate) struct ThumbnailService {
    cache_root: PathBuf,
    pdfium_directory: PathBuf,
}

impl ThumbnailService {
    pub(crate) fn new(cache_root: PathBuf, pdfium_directory: PathBuf) -> Self {
        Self {
            cache_root,
            pdfium_directory,
        }
    }

    fn render_source(
        &self,
        root: &Path,
        book: &DiscoveredBook,
        progress: &mpsc::SyncSender<ThumbnailWorkerEvent>,
    ) -> Result<(DynamicImage, Option<u32>), LibraryError> {
        let _ = progress.send(ThumbnailWorkerEvent::Progress(
            ThumbnailProgressStage::OpeningSource,
        ));
        match book.kind {
            BookKind::ImageFolder => {
                let first_page = book
                    .image_pages
                    .first()
                    .ok_or(LibraryError::ThumbnailFailed)?;
                let image = image::open(root.join(first_page.as_str()))
                    .map_err(|_| LibraryError::ThumbnailFailed)?;
                let _ = progress.send(ThumbnailWorkerEvent::Progress(
                    ThumbnailProgressStage::RenderingFirstPage,
                ));
                Ok((image, book.page_count))
            }
            BookKind::PdfFile => {
                // Pdfium's Rust bindings are process-global and may only be
                // initialized once. Keep that instance alive for the process
                // while this recoverable lock serializes native rendering.
                let _render_guard = acquire_render_lock(&PDFIUM_RENDER_LOCK);
                let pdfium = shared_pdfium(&self.pdfium_directory)?;
                let source_path = root.join(book.relative_path.as_str());
                let document = pdfium
                    .load_pdf_from_file(&source_path, None)
                    .map_err(|_| LibraryError::ThumbnailFailed)?;
                let _ = progress.send(ThumbnailWorkerEvent::Progress(
                    ThumbnailProgressStage::RenderingFirstPage,
                ));
                let page_count = u32::try_from(document.pages().len())
                    .map_err(|_| LibraryError::ThumbnailFailed)?;
                let page = document
                    .pages()
                    .get(0)
                    .map_err(|_| LibraryError::ThumbnailFailed)?;
                let image = page
                    .render_with_config(
                        &PdfRenderConfig::new()
                            .set_target_width(MAX_WIDTH as i32)
                            .render_form_data(true),
                    )
                    .map_err(|_| LibraryError::ThumbnailFailed)?
                    .as_image()
                    .map_err(|_| LibraryError::ThumbnailFailed)?;
                Ok((image, Some(page_count)))
            }
        }
    }

    fn generate_without_timeout(
        &self,
        root: &Path,
        book_id: BookId,
        book: &DiscoveredBook,
        progress: &mpsc::SyncSender<ThumbnailWorkerEvent>,
    ) -> Result<ThumbnailOutcome, LibraryError> {
        let (source, page_count) = self.render_source(root, book, progress)?;
        let (source_width, source_height) = source.dimensions();
        let scale = f64::min(
            1.0,
            f64::min(
                f64::from(MAX_WIDTH) / f64::from(source_width.max(1)),
                f64::from(MAX_HEIGHT) / f64::from(source_height.max(1)),
            ),
        );
        let width = (f64::from(source_width) * scale).round().max(1.0) as u32;
        let height = (f64::from(source_height) * scale).round().max(1.0) as u32;
        let thumbnail = source.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        // Use a new destination for every attempt. The catalog switches to this
        // file only after the render succeeds, so a failed repair cannot damage
        // the last known-good cover.
        let generation_id = uuid::Uuid::new_v4();
        let relative = format!("thumbnails/{book_id}-{generation_id}.png");
        let destination = self.cache_root.join(&relative);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|_| LibraryError::ThumbnailFailed)?;
        }
        let _ = progress.send(ThumbnailWorkerEvent::Progress(
            ThumbnailProgressStage::SavingCover,
        ));
        thumbnail
            .save_with_format(destination, ImageFormat::Png)
            .map_err(|_| LibraryError::ThumbnailFailed)?;

        Ok(ThumbnailOutcome {
            cache_relative_path: relative,
            width,
            height,
            format: "png",
            source_fingerprint: book.fingerprint.as_str().to_owned(),
            page_count,
        })
    }
}

impl ThumbnailGenerator for ThumbnailService {
    fn generate(
        &self,
        root: &Path,
        book_id: BookId,
        book: &DiscoveredBook,
    ) -> Result<ThumbnailOutcome, LibraryError> {
        self.generate_with_timeout(root, book_id, book, GENERATION_TIMEOUT)
    }

    fn generate_with_timeout(
        &self,
        root: &Path,
        book_id: BookId,
        book: &DiscoveredBook,
        timeout: Duration,
    ) -> Result<ThumbnailOutcome, LibraryError> {
        self.generate_with_progress(root, book_id, book, timeout, &mut |_| {})
    }

    fn generate_with_progress(
        &self,
        root: &Path,
        book_id: BookId,
        book: &DiscoveredBook,
        timeout: Duration,
        progress: &mut dyn FnMut(ThumbnailProgressStage),
    ) -> Result<ThumbnailOutcome, LibraryError> {
        let service = Self::new(self.cache_root.clone(), self.pdfium_directory.clone());
        let root = root.to_path_buf();
        let book = book.clone();
        let (sender, receiver) = mpsc::sync_channel(4);
        thread::spawn(move || {
            let result = service.generate_without_timeout(&root, book_id, &book, &sender);
            let _ = sender.send(ThumbnailWorkerEvent::Finished(result));
        });

        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match receiver.recv_timeout(remaining) {
                Ok(ThumbnailWorkerEvent::Progress(stage)) => progress(stage),
                Ok(ThumbnailWorkerEvent::Finished(result)) => {
                    if result.is_ok() {
                        progress(ThumbnailProgressStage::Completed);
                    }
                    return result;
                }
                Err(RecvTimeoutError::Timeout) => {
                    return Err(LibraryError::ThumbnailTimedOut);
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(LibraryError::ThumbnailFailed);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BookStatus, ContentFingerprint, RelativePath};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn image_thumbnail_is_bounded_and_stored_outside_library() {
        let library = TempDir::new().unwrap();
        let app_data = TempDir::new().unwrap();
        std::fs::create_dir(library.path().join("book")).unwrap();
        let source = DynamicImage::new_rgb8(800, 1200);
        source.save(library.path().join("book/page1.png")).unwrap();
        source.save(library.path().join("book/page2.png")).unwrap();
        let book = DiscoveredBook {
            kind: BookKind::ImageFolder,
            status: BookStatus::Available,
            relative_path: RelativePath::new("book").unwrap(),
            path_key: "book".to_owned(),
            title: "book".to_owned(),
            fingerprint: ContentFingerprint::new(
                "pdf-unavailable:very/long/日本語の書名/日本語の書名.pdf",
            )
            .unwrap(),
            size_bytes: None,
            modified_at_ms: None,
            page_count: Some(2),
            image_pages: vec![
                RelativePath::new("book/page1.png").unwrap(),
                RelativePath::new("book/page2.png").unwrap(),
            ],
        };
        let service = ThumbnailService::new(app_data.path().to_path_buf(), PathBuf::new());
        let mut stages = Vec::new();
        let outcome = service
            .generate_with_progress(
                library.path(),
                BookId::new(),
                &book,
                Duration::from_secs(1),
                &mut |stage| stages.push(stage),
            )
            .unwrap();

        assert_eq!(
            stages,
            [
                ThumbnailProgressStage::OpeningSource,
                ThumbnailProgressStage::RenderingFirstPage,
                ThumbnailProgressStage::SavingCover,
                ThumbnailProgressStage::Completed,
            ]
        );
        assert!(outcome.width <= MAX_WIDTH);
        assert!(outcome.height <= MAX_HEIGHT);
        assert!(app_data.path().join(&outcome.cache_relative_path).is_file());
        assert_eq!(
            Path::new(&outcome.cache_relative_path).components().count(),
            2
        );
        assert_eq!(std::fs::read_dir(library.path()).unwrap().count(), 1);
    }

    #[test]
    fn a_panicked_worker_does_not_disable_later_render_lock_users() {
        let lock = Arc::new(Mutex::new(()));
        let worker_lock = Arc::clone(&lock);
        let _ = std::thread::spawn(move || {
            let _guard = worker_lock.lock().unwrap();
            panic!("intentional test panic while holding the render lock");
        })
        .join();

        let _recovered_guard = acquire_render_lock(lock.as_ref());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn pdfium_instance_renders_two_covers_in_one_process() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let fixture_root = manifest.join("../tests/fixtures").canonicalize().unwrap();
        let pdfium_directory = manifest.join("resources/pdfium/windows-x86_64");
        let app_data = TempDir::new().unwrap();
        let source_path = fixture_root.join("pdfium-smoke.pdf");
        let metadata = std::fs::metadata(&source_path).unwrap();
        let book = DiscoveredBook {
            kind: BookKind::PdfFile,
            status: BookStatus::Available,
            relative_path: RelativePath::new("pdfium-smoke.pdf").unwrap(),
            path_key: "pdfium-smoke.pdf".to_owned(),
            title: "PDFium smoke".to_owned(),
            fingerprint: ContentFingerprint::new(format!("pdf:{}:1", metadata.len())).unwrap(),
            size_bytes: Some(metadata.len()),
            modified_at_ms: None,
            page_count: None,
            image_pages: Vec::new(),
        };
        let service =
            ThumbnailService::new(app_data.path().to_path_buf(), pdfium_directory.clone());

        let first = service
            .generate_with_timeout(&fixture_root, BookId::new(), &book, Duration::from_secs(5))
            .unwrap();
        let second = service
            .generate_with_timeout(&fixture_root, BookId::new(), &book, Duration::from_secs(5))
            .unwrap();

        assert!(app_data.path().join(first.cache_relative_path).is_file());
        assert!(app_data.path().join(second.cache_relative_path).is_file());
    }
}
