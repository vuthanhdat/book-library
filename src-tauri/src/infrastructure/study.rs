use std::{
    ffi::OsString,
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::application::{
    CancellationToken, LearningDraft, LearningExporter, OcrBlock, OcrProvider, OcrRecognition,
    StudyAssistant, StudyError, normalize_japanese_ocr_text,
};

pub(crate) struct TesseractOcrProvider {
    executable: PathBuf,
}

impl TesseractOcrProvider {
    pub(crate) fn discover() -> Self {
        let executable = resolve_tesseract_executable(
            std::env::var_os("BOOK_LIBRARY_TESSERACT"),
            known_tesseract_paths(),
        );
        Self { executable }
    }
}

fn resolve_tesseract_executable(
    explicit: Option<OsString>,
    known_paths: impl IntoIterator<Item = PathBuf>,
) -> PathBuf {
    explicit
        .map(PathBuf::from)
        .or_else(|| known_paths.into_iter().find(|path| path.is_file()))
        .unwrap_or_else(|| PathBuf::from("tesseract"))
}

fn known_tesseract_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let mut paths = Vec::new();
        for variable in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(root) = std::env::var_os(variable) {
                paths.push(
                    PathBuf::from(root)
                        .join("Tesseract-OCR")
                        .join("tesseract.exe"),
                );
            }
        }
        if let Some(root) = std::env::var_os("LOCALAPPDATA") {
            paths.push(
                PathBuf::from(root)
                    .join("Programs")
                    .join("Tesseract-OCR")
                    .join("tesseract.exe"),
            );
        }
        paths
    }

    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

impl OcrProvider for TesseractOcrProvider {
    fn available(&self) -> bool {
        Command::new(&self.executable)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    fn recognize(
        &self,
        image_path: &Path,
        cancellation: &CancellationToken,
    ) -> Result<OcrRecognition, StudyError> {
        let mut child = Command::new(&self.executable)
            .arg(image_path)
            .args(["stdout", "-l", "jpn+eng", "tsv"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| StudyError::OcrFailed)?;
        let mut stdout = child.stdout.take().ok_or(StudyError::OcrFailed)?;
        let output_reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).map(|_| bytes)
        });
        loop {
            if cancellation.is_cancelled() {
                let _ = child.kill();
                let _ = child.wait();
                let _ = output_reader.join();
                return Err(StudyError::Cancelled);
            }
            match child.try_wait().map_err(|_| StudyError::OcrFailed)? {
                Some(status) => {
                    if !status.success() {
                        return Err(StudyError::OcrFailed);
                    }
                    break;
                }
                None => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
        let output = output_reader.join().map_err(|_| StudyError::OcrFailed)?;
        let tsv = String::from_utf8(output.map_err(|_| StudyError::OcrFailed)?)
            .map_err(|_| StudyError::OcrFailed)?;
        parse_tesseract_tsv(&tsv)
    }
}

fn parse_tesseract_tsv(tsv: &str) -> Result<OcrRecognition, StudyError> {
    let mut blocks = Vec::new();
    for line in tsv.lines().skip(1) {
        let columns = line.splitn(12, '\t').collect::<Vec<_>>();
        if columns.len() != 12 || columns[0] != "5" {
            continue;
        }
        let text = columns[11].trim();
        if text.is_empty() {
            continue;
        }
        let confidence = columns[10].parse::<f32>().unwrap_or(0.0).clamp(0.0, 100.0) / 100.0;
        blocks.push(OcrBlock {
            block_index: blocks.len() as u32,
            text: text.to_owned(),
            confidence,
            x: columns[6].parse().unwrap_or(0),
            y: columns[7].parse().unwrap_or(0),
            width: columns[8].parse().unwrap_or(0),
            height: columns[9].parse().unwrap_or(0),
        });
    }
    if blocks.is_empty() {
        return Err(StudyError::OcrFailed);
    }
    let confidence = blocks.iter().map(|block| block.confidence).sum::<f32>() / blocks.len() as f32;
    let text = normalize_japanese_ocr_text(
        &blocks
            .iter()
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    Ok(OcrRecognition {
        text,
        confidence,
        provider_id: "tesseract-cli".to_owned(),
        provider_version: "system".to_owned(),
        blocks,
    })
}

pub(crate) struct TsvLearningExporter;

impl TsvLearningExporter {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl LearningExporter for TsvLearningExporter {
    fn export_tsv(&self, path: &Path, drafts: &[LearningDraft]) -> Result<(), StudyError> {
        if path.exists() {
            return Err(StudyError::ExportFailed);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| StudyError::ExportFailed)?;
        }
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|_| StudyError::ExportFailed)?;
        file.write_all(b"\xEF\xBB\xBFfront\tback\ttags\tsource\n")
            .map_err(|_| StudyError::ExportFailed)?;
        for draft in drafts {
            let source = match (&draft.book_relative_path, draft.page_index) {
                (Some(path), Some(page)) => format!("{path}#page={}", page + 1),
                (Some(path), None) => path.clone(),
                _ => draft.source_id.clone(),
            };
            let row = format!(
                "{}\t{}\t{}\t{}\n",
                tsv_field(&draft.front),
                tsv_field(&draft.back),
                tsv_field(&draft.tags.join(" ")),
                tsv_field(&source),
            );
            file.write_all(row.as_bytes())
                .map_err(|_| StudyError::ExportFailed)?;
        }
        file.sync_all().map_err(|_| StudyError::ExportFailed)
    }
}

fn tsv_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "")
        .replace('\n', "<br>")
}

pub(crate) struct BuiltinStudyAssistant;

impl BuiltinStudyAssistant {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl StudyAssistant for BuiltinStudyAssistant {
    fn available(&self) -> bool {
        true
    }

    fn generate(&self, kind: &str, context: &str) -> Result<String, StudyError> {
        let content = match kind {
            "explain" => format!(
                "Bản nháp học tập offline\n\nNgữ cảnh: {context}\n\nHãy kiểm tra cách đọc, từ loại và ngữ pháp bằng từ điển trước khi lưu."
            ),
            "translate" => format!(
                "Bản nháp dịch chưa được xác minh cho ngữ cảnh:\n{context}\n\nỨng dụng không tự coi nội dung này là bản dịch chính xác."
            ),
            "summarize" => format!(
                "Tóm tắt nháp: nội dung đã chọn gồm {} ký tự. Hãy chỉnh sửa trước khi đưa vào ghi chú.",
                context.chars().count()
            ),
            "flashcard" => format!(
                "Mặt trước: {context}\nMặt sau: Bổ sung cách đọc, nghĩa và câu giải thích đã kiểm chứng."
            ),
            _ => return Err(StudyError::InvalidInput),
        };
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn explicit_tesseract_path_wins_and_known_install_is_a_fallback() {
        let fixture = TempDir::new().unwrap();
        let known = fixture.path().join("known-tesseract");
        std::fs::write(&known, b"fixture").unwrap();

        assert_eq!(
            resolve_tesseract_executable(
                Some(OsString::from("configured-tesseract")),
                [known.clone()]
            ),
            PathBuf::from("configured-tesseract")
        );
        assert_eq!(resolve_tesseract_executable(None, [known.clone()]), known);
        assert_eq!(
            resolve_tesseract_executable(None, [fixture.path().join("missing")]),
            PathBuf::from("tesseract")
        );
    }

    #[test]
    fn parses_word_boxes_from_tesseract_tsv() {
        let result = parse_tesseract_tsv(
            "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n\
             5\t1\t1\t1\t1\t1\t10\t20\t30\t40\t95.0\t日本語\n\
             5\t1\t1\t1\t1\t2\t45\t20\t20\t40\t80.0\t本",
        )
        .unwrap();
        assert_eq!(result.text, "日本語本");
        assert_eq!(result.blocks.len(), 2);
        assert!(result.confidence > 0.8);
    }

    #[test]
    fn tsv_export_is_utf8_and_does_not_overwrite() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("cards.tsv");
        let draft = LearningDraft {
            id: "draft".to_owned(),
            source_kind: "dictionary_lookup".to_owned(),
            source_id: "日本語".to_owned(),
            book_relative_path: Some("日本語/book.pdf".to_owned()),
            page_index: Some(0),
            front: "日本語".to_owned(),
            back: "tiếng Nhật\nにほんご".to_owned(),
            tags: vec!["japanese".to_owned()],
            status: "approved".to_owned(),
        };
        let exporter = TsvLearningExporter::new();
        exporter.export_tsv(&path, &[draft]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("日本語"));
        assert!(content.contains("<br>"));
        assert!(exporter.export_tsv(&path, &[]).is_err());
    }
}
