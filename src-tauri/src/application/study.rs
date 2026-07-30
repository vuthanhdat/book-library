use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::BookId;

use super::CancellationToken;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum StudyError {
    #[error("the optional module is disabled")]
    ModuleDisabled,
    #[error("the optional module is unavailable")]
    ModuleUnavailable,
    #[error("the study request is invalid")]
    InvalidInput,
    #[error("the dictionary package is invalid or unsupported")]
    DictionaryPackageInvalid,
    #[error("the requested book or page is unavailable")]
    SourceUnavailable,
    #[error("the local OCR provider failed")]
    OcrFailed,
    #[error("the OCR job was cancelled")]
    Cancelled,
    #[error("the study data could not be read or saved")]
    RepositoryFailed,
    #[error("the export could not be written")]
    ExportFailed,
    #[error("the requested draft does not exist")]
    DraftNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StudyModule {
    pub(crate) id: String,
    pub(crate) enabled: bool,
    pub(crate) available: bool,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DictionaryEntry {
    pub(crate) id: String,
    pub(crate) expression: String,
    pub(crate) reading: String,
    pub(crate) part_of_speech: String,
    pub(crate) meaning_vi: String,
    pub(crate) han_viet: Option<String>,
    pub(crate) package_name: String,
    pub(crate) package_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JapaneseToken {
    pub(crate) surface: String,
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) entries: Vec<DictionaryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DictionaryLookup {
    pub(crate) query: String,
    pub(crate) entries: Vec<DictionaryEntry>,
    pub(crate) tokens: Vec<JapaneseToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DictionaryImportSummary {
    pub(crate) package_id: String,
    pub(crate) imported: u64,
    pub(crate) skipped: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OcrBlock {
    pub(crate) block_index: u32,
    pub(crate) text: String,
    pub(crate) confidence: f32,
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OcrRecognition {
    pub(crate) text: String,
    pub(crate) confidence: f32,
    pub(crate) provider_id: String,
    pub(crate) provider_version: String,
    pub(crate) blocks: Vec<OcrBlock>,
}

#[derive(Debug, Clone)]
pub(crate) struct BookPageSource {
    pub(crate) book_id: BookId,
    pub(crate) title: String,
    pub(crate) page_index: u32,
    pub(crate) page_count: u32,
    pub(crate) source_fingerprint: String,
    pub(crate) library_root: PathBuf,
    pub(crate) source_path: PathBuf,
    pub(crate) kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RenderedStudyPage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) media_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StudyReaderPage {
    pub(crate) book_id: String,
    pub(crate) book_title: String,
    pub(crate) page_index: u32,
    pub(crate) page_count: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) media_type: String,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OcrPageRecord {
    pub(crate) id: String,
    pub(crate) book_id: String,
    pub(crate) book_title: String,
    pub(crate) page_index: u32,
    pub(crate) text: String,
    pub(crate) confidence: f32,
    pub(crate) provider_id: String,
    pub(crate) provider_version: String,
    pub(crate) blocks: Vec<OcrBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LearningDraft {
    pub(crate) id: String,
    pub(crate) source_kind: String,
    pub(crate) source_id: String,
    pub(crate) book_relative_path: Option<String>,
    pub(crate) page_index: Option<u32>,
    pub(crate) front: String,
    pub(crate) back: String,
    pub(crate) tags: Vec<String>,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AiDraft {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) context: String,
    pub(crate) content: String,
    pub(crate) accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrustedModule {
    pub(crate) id: String,
    pub(crate) version: String,
    pub(crate) capabilities: Vec<String>,
    pub(crate) permissions: Vec<String>,
    pub(crate) compatible: bool,
}

pub(crate) trait StudyRepository {
    fn modules(&self) -> Result<Vec<StudyModule>, StudyError>;
    fn set_module_enabled(&self, module_id: &str, enabled: bool) -> Result<(), StudyError>;
    fn module_enabled(&self, module_id: &str) -> Result<bool, StudyError>;
    fn dictionary_lookup(&self, query: &str) -> Result<Vec<DictionaryEntry>, StudyError>;
    fn dictionary_terms(&self, query: &str) -> Result<Vec<String>, StudyError>;
    fn import_dictionary_package(
        &self,
        path: &Path,
        name: Option<&str>,
        version: Option<&str>,
        license_id: &str,
    ) -> Result<DictionaryImportSummary, StudyError>;
    fn save_lookup_history(&self, query: &str) -> Result<(), StudyError>;
    fn clear_lookup_history(&self) -> Result<(), StudyError>;
    fn book_page_source(
        &self,
        book_id: BookId,
        page_index: u32,
    ) -> Result<BookPageSource, StudyError>;
    fn save_ocr_page(
        &self,
        source: &BookPageSource,
        recognition: &OcrRecognition,
    ) -> Result<OcrPageRecord, StudyError>;
    fn list_ocr_pages(&self, book_id: Option<BookId>) -> Result<Vec<OcrPageRecord>, StudyError>;
    fn update_ocr_page_text(&self, page_id: &str, text: &str) -> Result<(), StudyError>;
    #[allow(clippy::too_many_arguments)]
    fn create_learning_draft(
        &self,
        source_kind: &str,
        source_id: &str,
        book_relative_path: Option<&str>,
        page_index: Option<u32>,
        front: &str,
        back: &str,
        tags: &[String],
    ) -> Result<LearningDraft, StudyError>;
    fn list_learning_drafts(&self) -> Result<Vec<LearningDraft>, StudyError>;
    fn approve_learning_draft(&self, draft_id: &str) -> Result<LearningDraft, StudyError>;
    fn approved_learning_drafts(&self) -> Result<Vec<LearningDraft>, StudyError>;
    fn mark_learning_drafts_exported(&self, draft_ids: &[String]) -> Result<(), StudyError>;
    fn save_ai_draft(
        &self,
        kind: &str,
        context: &str,
        content: &str,
    ) -> Result<AiDraft, StudyError>;
    fn list_ai_drafts(&self) -> Result<Vec<AiDraft>, StudyError>;
}

pub(crate) trait PageMaterializer {
    fn materialize(&self, source: &BookPageSource) -> Result<PathBuf, StudyError>;
    fn render(&self, source: &BookPageSource) -> Result<RenderedStudyPage, StudyError>;
}

pub(crate) trait OcrProvider {
    fn available(&self) -> bool;
    fn recognize(
        &self,
        image_path: &Path,
        cancellation: &CancellationToken,
    ) -> Result<OcrRecognition, StudyError>;
}

pub(crate) trait LearningExporter {
    fn export_tsv(&self, path: &Path, drafts: &[LearningDraft]) -> Result<(), StudyError>;
}

pub(crate) trait StudyAssistant {
    fn available(&self) -> bool;
    fn generate(&self, kind: &str, context: &str) -> Result<String, StudyError>;
}

pub(crate) struct StudyWorkspace<'a, Repository, Pages, Ocr, Exporter, Assistant> {
    repository: &'a Repository,
    pages: &'a Pages,
    ocr: &'a Ocr,
    exporter: &'a Exporter,
    assistant: &'a Assistant,
}

impl<'a, Repository, Pages, Ocr, Exporter, Assistant>
    StudyWorkspace<'a, Repository, Pages, Ocr, Exporter, Assistant>
where
    Repository: StudyRepository,
    Pages: PageMaterializer,
    Ocr: OcrProvider,
    Exporter: LearningExporter,
    Assistant: StudyAssistant,
{
    pub(crate) fn new(
        repository: &'a Repository,
        pages: &'a Pages,
        ocr: &'a Ocr,
        exporter: &'a Exporter,
        assistant: &'a Assistant,
    ) -> Self {
        Self {
            repository,
            pages,
            ocr,
            exporter,
            assistant,
        }
    }

    pub(crate) fn modules(&self) -> Result<Vec<StudyModule>, StudyError> {
        let mut modules = self.repository.modules()?;
        for module in &mut modules {
            module.available = match module.id.as_str() {
                "ocr" => self.ocr.available(),
                "ai" => self.assistant.available(),
                _ => true,
            };
            module.status = if !module.enabled {
                "disabled"
            } else if module.available {
                "ready"
            } else {
                "unavailable"
            }
            .to_owned();
        }
        Ok(modules)
    }

    pub(crate) fn set_module_enabled(
        &self,
        module_id: &str,
        enabled: bool,
    ) -> Result<Vec<StudyModule>, StudyError> {
        if !matches!(
            module_id,
            "dictionary" | "ocr" | "anki" | "ai" | "trusted_modules"
        ) {
            return Err(StudyError::InvalidInput);
        }
        self.repository.set_module_enabled(module_id, enabled)?;
        self.modules()
    }

    pub(crate) fn lookup(
        &self,
        query: &str,
        save_history: bool,
    ) -> Result<DictionaryLookup, StudyError> {
        self.require_enabled("dictionary")?;
        let normalized_query = normalize_japanese_ocr_text(query);
        let query = validate_text(&normalized_query, 4_000)?;
        let entries = self.repository.dictionary_lookup(query)?;
        let terms = self.repository.dictionary_terms(query)?;
        let tokens = tokenize_with_terms(query, &terms)
            .into_iter()
            .map(|(surface, start, end)| {
                let token_entries = self.repository.dictionary_lookup(&surface)?;
                Ok(JapaneseToken {
                    surface,
                    start,
                    end,
                    entries: token_entries,
                })
            })
            .collect::<Result<Vec<_>, StudyError>>()?;
        if save_history {
            self.repository.save_lookup_history(query)?;
        }
        Ok(DictionaryLookup {
            query: query.to_owned(),
            entries,
            tokens,
        })
    }

    pub(crate) fn import_dictionary_package(
        &self,
        path: &Path,
        name: Option<&str>,
        version: Option<&str>,
        license_id: &str,
    ) -> Result<DictionaryImportSummary, StudyError> {
        self.require_enabled("dictionary")?;
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase);
        if !matches!(extension.as_deref(), Some("tsv" | "zip")) {
            return Err(StudyError::DictionaryPackageInvalid);
        }
        let name = name.map(|value| validate_text(value, 200)).transpose()?;
        let version = version.map(|value| validate_text(value, 100)).transpose()?;
        if extension.as_deref() == Some("tsv") && (name.is_none() || version.is_none()) {
            return Err(StudyError::InvalidInput);
        }
        let license_id = validate_text(license_id, 100)?;
        self.repository
            .import_dictionary_package(path, name, version, license_id)
    }

    pub(crate) fn clear_lookup_history(&self) -> Result<(), StudyError> {
        self.repository.clear_lookup_history()
    }

    pub(crate) fn ocr_page(
        &self,
        book_id: BookId,
        page_index: u32,
        cancellation: &CancellationToken,
    ) -> Result<OcrPageRecord, StudyError> {
        self.require_enabled("ocr")?;
        if !self.ocr.available() {
            return Err(StudyError::ModuleUnavailable);
        }
        if cancellation.is_cancelled() {
            return Err(StudyError::Cancelled);
        }
        let source = self.repository.book_page_source(book_id, page_index)?;
        let materialized = self.pages.materialize(&source)?;
        if cancellation.is_cancelled() {
            return Err(StudyError::Cancelled);
        }
        let recognition = self.ocr.recognize(&materialized, cancellation)?;
        if cancellation.is_cancelled() {
            return Err(StudyError::Cancelled);
        }
        self.repository.save_ocr_page(&source, &recognition)
    }

    pub(crate) fn reader_page(
        &self,
        book_id: BookId,
        page_index: u32,
    ) -> Result<StudyReaderPage, StudyError> {
        let source = self.repository.book_page_source(book_id, page_index)?;
        let rendered = self.pages.render(&source)?;
        Ok(StudyReaderPage {
            book_id: source.book_id.to_string(),
            book_title: source.title,
            page_index: source.page_index,
            page_count: source.page_count,
            width: rendered.width,
            height: rendered.height,
            media_type: rendered.media_type,
            bytes: rendered.bytes,
        })
    }

    pub(crate) fn list_ocr_pages(
        &self,
        book_id: Option<BookId>,
    ) -> Result<Vec<OcrPageRecord>, StudyError> {
        self.repository.list_ocr_pages(book_id)
    }

    pub(crate) fn trim_ocr_page(&self, page_id: &str) -> Result<OcrPageRecord, StudyError> {
        self.require_enabled("ocr")?;
        validate_identifier(page_id)?;
        let mut page = self
            .repository
            .list_ocr_pages(None)?
            .into_iter()
            .find(|page| page.id == page_id)
            .ok_or(StudyError::SourceUnavailable)?;
        let text = normalize_japanese_ocr_text(&page.text);
        let text = validate_text(&text, 100_000)?;
        self.repository.update_ocr_page_text(page_id, text)?;
        page.text = text.to_owned();
        Ok(page)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_learning_draft(
        &self,
        source_kind: &str,
        source_id: &str,
        book_relative_path: Option<&str>,
        page_index: Option<u32>,
        front: &str,
        back: &str,
        tags: &[String],
    ) -> Result<LearningDraft, StudyError> {
        self.require_enabled("anki")?;
        if !matches!(
            source_kind,
            "dictionary_lookup" | "ocr_page" | "ai_output" | "note"
        ) {
            return Err(StudyError::InvalidInput);
        }
        let front = validate_text(front, 10_000)?;
        let back = validate_text(back, 25_000)?;
        let tags = normalize_tags(tags)?;
        self.repository.create_learning_draft(
            source_kind,
            source_id,
            book_relative_path,
            page_index,
            front,
            back,
            &tags,
        )
    }

    pub(crate) fn list_learning_drafts(&self) -> Result<Vec<LearningDraft>, StudyError> {
        self.repository.list_learning_drafts()
    }

    pub(crate) fn approve_learning_draft(
        &self,
        draft_id: &str,
    ) -> Result<LearningDraft, StudyError> {
        validate_identifier(draft_id)?;
        self.repository.approve_learning_draft(draft_id)
    }

    pub(crate) fn export_approved(&self, path: &Path) -> Result<u64, StudyError> {
        self.require_enabled("anki")?;
        if path.extension().and_then(|value| value.to_str()) != Some("tsv") {
            return Err(StudyError::InvalidInput);
        }
        let drafts = self.repository.approved_learning_drafts()?;
        if drafts.is_empty() {
            return Err(StudyError::InvalidInput);
        }
        self.exporter.export_tsv(path, &drafts)?;
        let ids = drafts
            .iter()
            .map(|draft| draft.id.clone())
            .collect::<Vec<_>>();
        self.repository.mark_learning_drafts_exported(&ids)?;
        Ok(drafts.len() as u64)
    }

    pub(crate) fn assist(&self, kind: &str, context: &str) -> Result<AiDraft, StudyError> {
        self.require_enabled("ai")?;
        if !matches!(kind, "explain" | "translate" | "summarize" | "flashcard") {
            return Err(StudyError::InvalidInput);
        }
        let context = validate_text(context, 25_000)?;
        let content = self.assistant.generate(kind, context)?;
        self.repository.save_ai_draft(kind, context, &content)
    }

    pub(crate) fn list_ai_drafts(&self) -> Result<Vec<AiDraft>, StudyError> {
        self.repository.list_ai_drafts()
    }

    pub(crate) fn trusted_modules(&self) -> Vec<TrustedModule> {
        vec![
            TrustedModule {
                id: "builtin.dictionary.ja-vi".to_owned(),
                version: "1".to_owned(),
                capabilities: vec![
                    "dictionary.lookup".to_owned(),
                    "japanese.analyze".to_owned(),
                ],
                permissions: vec!["app_data.read".to_owned()],
                compatible: true,
            },
            TrustedModule {
                id: "system.tesseract-cli".to_owned(),
                version: "1".to_owned(),
                capabilities: vec!["ocr.page".to_owned()],
                permissions: vec![
                    "source.read_selected_page".to_owned(),
                    "process.spawn".to_owned(),
                ],
                compatible: self.ocr.available(),
            },
            TrustedModule {
                id: "builtin.study-assistant".to_owned(),
                version: "1".to_owned(),
                capabilities: vec!["ai.draft".to_owned()],
                permissions: vec![],
                compatible: self.assistant.available(),
            },
        ]
    }

    fn require_enabled(&self, module_id: &str) -> Result<(), StudyError> {
        match self.repository.module_enabled(module_id)? {
            true => Ok(()),
            false => Err(StudyError::ModuleDisabled),
        }
    }
}

fn validate_text(value: &str, maximum: usize) -> Result<&str, StudyError> {
    let value = value.trim();
    if value.is_empty() || value.contains('\0') || value.chars().count() > maximum {
        Err(StudyError::InvalidInput)
    } else {
        Ok(value)
    }
}

fn validate_identifier(value: &str) -> Result<(), StudyError> {
    if value.is_empty()
        || value.len() > 128
        || value.contains(|character: char| character.is_control())
    {
        Err(StudyError::InvalidInput)
    } else {
        Ok(())
    }
}

pub(crate) fn normalize_japanese_ocr_text(value: &str) -> String {
    value
        .lines()
        .map(normalize_japanese_ocr_line)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_japanese_ocr_line(value: &str) -> String {
    let characters = value.trim().chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(value.len());
    let mut index = 0usize;
    while index < characters.len() {
        let character = characters[index];
        if !character.is_whitespace() {
            output.push(character);
            index += 1;
            continue;
        }

        let previous = output.chars().next_back();
        while index < characters.len() && characters[index].is_whitespace() {
            index += 1;
        }
        let next = characters.get(index).copied();
        let touches_japanese = previous.is_some_and(is_japanese_spacing_character)
            || next.is_some_and(is_japanese_spacing_character);
        if !touches_japanese && previous.is_some() && next.is_some() && !output.ends_with(' ') {
            output.push(' ');
        }
    }
    output
}

fn is_japanese_spacing_character(character: char) -> bool {
    matches!(
        character as u32,
        0x3000..=0x30ff | 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xff00..=0xffef
    )
}

fn normalize_tags(tags: &[String]) -> Result<Vec<String>, StudyError> {
    if tags.len() > 50 {
        return Err(StudyError::InvalidInput);
    }
    let mut normalized = Vec::new();
    for tag in tags {
        let tag = tag.trim().trim_start_matches('#');
        if tag.is_empty()
            || tag.chars().count() > 64
            || tag.chars().any(char::is_whitespace)
            || tag.chars().any(char::is_control)
        {
            return Err(StudyError::InvalidInput);
        }
        if !normalized.iter().any(|current| current == tag) {
            normalized.push(tag.to_owned());
        }
    }
    Ok(normalized)
}

fn tokenize_with_terms(query: &str, terms: &[String]) -> Vec<(String, u32, u32)> {
    let characters = query.char_indices().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut char_index = 0usize;
    while char_index < characters.len() {
        let byte_start = characters[char_index].0;
        let tail = &query[byte_start..];
        let longest = terms
            .iter()
            .filter(|term| tail.starts_with(term.as_str()))
            .max_by_key(|term| term.chars().count());
        if let Some(term) = longest {
            let length = term.chars().count();
            tokens.push((
                term.clone(),
                char_index as u32,
                (char_index + length) as u32,
            ));
            char_index += length;
        } else {
            char_index += 1;
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenization_prefers_the_longest_known_term() {
        let terms = vec![
            "日本".to_owned(),
            "日本語".to_owned(),
            "語".to_owned(),
            "勉強".to_owned(),
        ];
        assert_eq!(
            tokenize_with_terms("日本語を勉強する", &terms),
            vec![("日本語".to_owned(), 0, 3), ("勉強".to_owned(), 4, 6),]
        );
    }

    #[test]
    fn japanese_ocr_spacing_is_removed_without_joining_english_words() {
        assert_eq!(
            normalize_japanese_ocr_text(" はじめ に あなた が AI を 学ぶ 。 "),
            "はじめにあなたがAIを学ぶ。"
        );
        assert_eq!(
            normalize_japanese_ocr_text("Japanese language data"),
            "Japanese language data"
        );
        assert_eq!(normalize_japanese_ocr_text("water と ice"), "waterとice");
    }

    #[test]
    fn draft_tags_are_normalized_without_accepting_unsafe_values() {
        assert_eq!(
            normalize_tags(&["#日本語".to_owned(), "jlpt-n5".to_owned()]).unwrap(),
            vec!["日本語", "jlpt-n5"]
        );
        assert!(normalize_tags(&["two words".to_owned()]).is_err());
    }
}
