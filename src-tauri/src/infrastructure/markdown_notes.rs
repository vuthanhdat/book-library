use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::{
    application::{MarkdownNotes, NoteProjection, NotesError, ParsedHeading, ParsedNoteLink},
    domain::RelativePath,
};

pub(crate) struct MarkdownNotesStore;

impl MarkdownNotesStore {
    pub(crate) fn new() -> Self {
        Self
    }

    fn projection(root: &Path, path: &Path, body: &str) -> Result<NoteProjection, NotesError> {
        let root = root
            .canonicalize()
            .map_err(|_| NotesError::RootUnavailable)?;
        let path = path.canonicalize().map_err(|_| NotesError::ReadFailed)?;
        let relative_text = path
            .strip_prefix(&root)
            .ok()
            .and_then(Path::to_str)
            .ok_or(NotesError::InvalidNotePath)?;
        let relative_path =
            RelativePath::new(relative_text).map_err(|_| NotesError::InvalidNotePath)?;
        let metadata = fs::metadata(&path).map_err(|_| NotesError::ReadFailed)?;
        let modified_at_ms = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .and_then(|value| i64::try_from(value.as_millis()).ok());
        let headings = parse_headings(body);
        let title = headings
            .iter()
            .find(|heading| heading.level == 1)
            .map(|heading| heading.text.clone())
            .or_else(|| {
                path.file_stem()
                    .and_then(|value| value.to_str())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "Untitled note".to_owned());
        let book_relative_path = parse_frontmatter_value(body, "book_relative_path")
            .and_then(|value| RelativePath::new(value).ok());
        let path_key = if cfg!(target_os = "windows") {
            relative_path.as_str().to_lowercase()
        } else {
            relative_path.as_str().to_owned()
        };
        Ok(NoteProjection {
            relative_path,
            path_key,
            title,
            fingerprint: format!(
                "note:{}:{}",
                metadata.len(),
                modified_at_ms.unwrap_or_default()
            ),
            size_bytes: metadata.len(),
            modified_at_ms,
            book_relative_path,
            headings,
            tags: parse_tags(body),
            links: parse_links(body),
        })
    }

    fn scan_directory(
        root: &Path,
        directory: &Path,
        notes: &mut Vec<NoteProjection>,
        issues: &mut u64,
    ) {
        let entries = match fs::read_dir(directory) {
            Ok(value) => value,
            Err(_) => {
                *issues += 1;
                return;
            }
        };
        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(value) => value,
                Err(_) => {
                    *issues += 1;
                    continue;
                }
            };
            if file_type.is_symlink() {
                *issues += 1;
            } else if file_type.is_dir() {
                Self::scan_directory(root, &entry.path(), notes, issues);
            } else if file_type.is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case("md"))
            {
                match fs::read_to_string(entry.path()) {
                    Ok(body) => match Self::projection(root, &entry.path(), &body) {
                        Ok(note) => notes.push(note),
                        Err(_) => *issues += 1,
                    },
                    Err(_) => *issues += 1,
                }
            }
        }
    }

    fn safe_filename(title: &str) -> String {
        let mut name = title
            .chars()
            .map(|character| {
                if matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                ) || character.is_control()
                {
                    '-'
                } else {
                    character
                }
            })
            .collect::<String>();
        name = name.trim_matches([' ', '.']).trim().to_owned();
        if name.is_empty() {
            name = "Note".to_owned();
        }
        name.chars().take(100).collect()
    }

    fn write_new(path: &Path, body: &str) -> Result<(), NotesError> {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|_| NotesError::WriteFailed)?;
        file.write_all(body.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|_| NotesError::WriteFailed)
    }

    fn replace_atomically(path: &Path, body: &str) -> Result<(), NotesError> {
        let parent = path.parent().ok_or(NotesError::InvalidNotePath)?;
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(NotesError::InvalidNotePath)?;
        let temporary = parent.join(format!("{filename}.{}.tmp", uuid::Uuid::new_v4()));
        Self::write_new(&temporary, body)?;

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::ffi::OsStrExt;

            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
            }
            const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
            const MOVEFILE_WRITE_THROUGH: u32 = 0x8;
            let existing: Vec<u16> = temporary.as_os_str().encode_wide().chain(Some(0)).collect();
            let new: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            // SAFETY: both paths are valid null-terminated UTF-16 strings.
            let moved = unsafe {
                MoveFileExW(
                    existing.as_ptr(),
                    new.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            };
            if moved == 0 {
                return Err(NotesError::WriteFailed);
            }
        }

        #[cfg(not(target_os = "windows"))]
        fs::rename(&temporary, path).map_err(|_| NotesError::WriteFailed)?;

        Ok(())
    }
}

impl MarkdownNotes for MarkdownNotesStore {
    fn scan(&self, root: &Path) -> Result<(Vec<NoteProjection>, u64), NotesError> {
        let root = root
            .canonicalize()
            .map_err(|_| NotesError::RootUnavailable)?;
        let mut notes = Vec::new();
        let mut issues = 0;
        Self::scan_directory(&root, &root, &mut notes, &mut issues);
        Ok((notes, issues))
    }

    fn create(
        &self,
        root: &Path,
        title: &str,
        book_relative_path: Option<&RelativePath>,
    ) -> Result<(NoteProjection, String), NotesError> {
        let suffix = &uuid::Uuid::new_v4().to_string()[..8];
        let path = root.join(format!("{}-{suffix}.md", Self::safe_filename(title)));
        let body = if let Some(book_path) = book_relative_path {
            format!(
                "---\nbook_relative_path: \"{}\"\n---\n\n# {title}\n\n",
                book_path
                    .as_str()
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
            )
        } else {
            format!("# {title}\n\n")
        };
        Self::write_new(&path, &body)?;
        let projection = Self::projection(root, &path, &body)?;
        Ok((projection, body))
    }

    fn read(&self, root: &Path, relative_path: &RelativePath) -> Result<String, NotesError> {
        let path = self.resolve(root, relative_path)?;
        fs::read_to_string(path).map_err(|_| NotesError::ReadFailed)
    }

    fn save(
        &self,
        root: &Path,
        relative_path: &RelativePath,
        body: &str,
    ) -> Result<NoteProjection, NotesError> {
        let path = self.resolve(root, relative_path)?;
        Self::replace_atomically(&path, body)?;
        Self::projection(root, &path, body)
    }

    fn resolve(&self, root: &Path, relative_path: &RelativePath) -> Result<PathBuf, NotesError> {
        let root = root
            .canonicalize()
            .map_err(|_| NotesError::RootUnavailable)?;
        let path = root
            .join(relative_path.as_str())
            .canonicalize()
            .map_err(|_| NotesError::NoteNotFound)?;
        if !path.starts_with(&root) || !path.is_file() {
            return Err(NotesError::InvalidNotePath);
        }
        Ok(path)
    }
}

fn parse_frontmatter_value(body: &str, key: &str) -> Option<String> {
    let mut lines = body.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        if let Some((candidate, value)) = line.split_once(':')
            && candidate.trim() == key
        {
            return Some(value.trim().trim_matches(['"', '\'']).to_owned());
        }
    }
    None
}

fn parse_headings(body: &str) -> Vec<ParsedHeading> {
    body.lines()
        .filter_map(|line| {
            let hashes = line.chars().take_while(|value| *value == '#').count();
            if !(1..=6).contains(&hashes)
                || !line.chars().nth(hashes).is_some_and(char::is_whitespace)
            {
                return None;
            }
            let text = line[hashes..].trim();
            (!text.is_empty()).then(|| ParsedHeading {
                level: hashes as u8,
                text: text.to_owned(),
            })
        })
        .collect()
}

fn parse_tags(body: &str) -> Vec<String> {
    let mut tags = BTreeSet::new();
    for token in body.split_whitespace() {
        if let Some(tag) = token.strip_prefix('#') {
            let tag = tag.trim_matches(|value: char| !value.is_alphanumeric() && value != '_');
            if !tag.is_empty() {
                tags.insert(tag.to_owned());
            }
        }
    }
    tags.into_iter().collect()
}

fn parse_links(body: &str) -> Vec<ParsedNoteLink> {
    let mut links = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("[[") {
        rest = &rest[start + 2..];
        let Some(end) = rest.find("]]") else { break };
        let raw = &rest[..end];
        let (target, label) = raw.split_once('|').unwrap_or((raw, raw));
        if !target.trim().is_empty() {
            links.push(ParsedNoteLink {
                target_ref: target.trim().to_owned(),
                link_text: label.trim().to_owned(),
            });
        }
        rest = &rest[end + 2..];
    }
    let mut rest = body;
    while let Some(label_start) = rest.find('[') {
        rest = &rest[label_start + 1..];
        let Some(separator) = rest.find("](") else {
            break;
        };
        let label = &rest[..separator];
        rest = &rest[separator + 2..];
        let Some(end) = rest.find(')') else { break };
        let target = rest[..end].trim();
        if target.to_ascii_lowercase().ends_with(".md") {
            links.push(ParsedNoteLink {
                target_ref: target.to_owned(),
                link_text: label.to_owned(),
            });
        }
        rest = &rest[end + 1..];
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_parses_and_atomically_saves_portable_markdown() {
        let root = TempDir::new().unwrap();
        let store = MarkdownNotesStore::new();
        let book = RelativePath::new("日本語/本.pdf").unwrap();
        let (created, _) = store.create(root.path(), "読書メモ", Some(&book)).unwrap();
        let updated = "---\nbook_relative_path: \"日本語/本.pdf\"\n---\n\n# 読書メモ\n\n## 要点\n#学習 [[別のノート]] [Link](other.md)\n";

        let projection = store
            .save(root.path(), &created.relative_path, updated)
            .unwrap();

        assert_eq!(projection.title, "読書メモ");
        assert_eq!(projection.book_relative_path, Some(book));
        assert_eq!(projection.headings.len(), 2);
        assert_eq!(projection.tags, ["学習"]);
        assert_eq!(projection.links.len(), 2);
        assert_eq!(
            store.read(root.path(), &created.relative_path).unwrap(),
            updated
        );
    }
}
