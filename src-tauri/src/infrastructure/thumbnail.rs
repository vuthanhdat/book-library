use std::{
    path::{Path, PathBuf},
    sync::{Mutex, mpsc},
    thread,
    time::Duration,
};

use image::{DynamicImage, GenericImageView, ImageFormat};
use pdfium_render::prelude::{PdfRenderConfig, Pdfium};

use crate::{
    application::{DiscoveredBook, LibraryError, ThumbnailGenerator, ThumbnailOutcome},
    domain::{BookId, BookKind},
};

const MAX_WIDTH: u32 = 320;
const MAX_HEIGHT: u32 = 448;
const GENERATION_TIMEOUT: Duration = Duration::from_secs(1);
static PDFIUM_RENDER_LOCK: Mutex<()> = Mutex::new(());

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
    ) -> Result<(DynamicImage, Option<u32>), LibraryError> {
        match book.kind {
            BookKind::ImageFolder => {
                let first_page = book
                    .image_pages
                    .first()
                    .ok_or(LibraryError::ThumbnailFailed)?;
                let image = image::open(root.join(first_page.as_str()))
                    .map_err(|_| LibraryError::ThumbnailFailed)?;
                Ok((image, book.page_count))
            }
            BookKind::PdfFile => {
                let _render_guard = PDFIUM_RENDER_LOCK
                    .lock()
                    .map_err(|_| LibraryError::ThumbnailFailed)?;
                let bindings = Pdfium::bind_to_library(
                    Pdfium::pdfium_platform_library_name_at_path(&self.pdfium_directory),
                )
                .map_err(|_| LibraryError::ThumbnailFailed)?;
                let pdfium = Pdfium::new(bindings);
                let source_path = root.join(book.relative_path.as_str());
                let document = pdfium
                    .load_pdf_from_file(&source_path, None)
                    .map_err(|_| LibraryError::ThumbnailFailed)?;
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
    ) -> Result<ThumbnailOutcome, LibraryError> {
        let (source, page_count) = self.render_source(root, book)?;
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
        let fingerprint_key = book.fingerprint.as_str().replace(':', "_");
        let relative = format!("thumbnails/{book_id}-{fingerprint_key}.png");
        let destination = self.cache_root.join(&relative);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|_| LibraryError::ThumbnailFailed)?;
        }
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
        let service = Self::new(self.cache_root.clone(), self.pdfium_directory.clone());
        let root = root.to_path_buf();
        let book = book.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let result = service.generate_without_timeout(&root, book_id, &book);
            let _ = sender.send(result);
        });
        receiver
            .recv_timeout(GENERATION_TIMEOUT)
            .map_err(|_| LibraryError::ThumbnailFailed)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BookStatus, ContentFingerprint, RelativePath};
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
            fingerprint: ContentFingerprint::new("images:2:1:1").unwrap(),
            size_bytes: None,
            modified_at_ms: None,
            page_count: Some(2),
            image_pages: vec![
                RelativePath::new("book/page1.png").unwrap(),
                RelativePath::new("book/page2.png").unwrap(),
            ],
        };
        let service = ThumbnailService::new(app_data.path().to_path_buf(), PathBuf::new());
        let outcome = service
            .generate(library.path(), BookId::new(), &book)
            .unwrap();

        assert!(outcome.width <= MAX_WIDTH);
        assert!(outcome.height <= MAX_HEIGHT);
        assert!(app_data.path().join(outcome.cache_relative_path).is_file());
        assert_eq!(std::fs::read_dir(library.path()).unwrap().count(), 1);
    }
}
