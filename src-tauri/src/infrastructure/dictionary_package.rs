use std::{fs, io::Read, path::Path};

use serde_json::Value;

use crate::application::StudyError;

const MAX_TSV_BYTES: u64 = 50 * 1024 * 1024;
const MAX_ZIP_BYTES: u64 = 200 * 1024 * 1024;
const MAX_ARCHIVE_FILES: usize = 2_048;
const MAX_ARCHIVE_UNCOMPRESSED_BYTES: u64 = 512 * 1024 * 1024;
const MAX_BANK_BYTES: u64 = 64 * 1024 * 1024;
const MAX_ENTRIES: usize = 500_000;

#[derive(Debug)]
pub(crate) struct ParsedDictionaryPackage {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) checksum: String,
    pub(crate) entries: Vec<ParsedDictionaryEntry>,
    pub(crate) skipped: u64,
}

#[derive(Debug)]
pub(crate) struct ParsedDictionaryEntry {
    pub(crate) expression: String,
    pub(crate) reading: String,
    pub(crate) part_of_speech: String,
    pub(crate) meaning_vi: String,
    pub(crate) han_viet: Option<String>,
}

pub(crate) fn parse_dictionary_package(
    path: &Path,
    name: Option<&str>,
    version: Option<&str>,
) -> Result<ParsedDictionaryPackage, StudyError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    let result = match extension.as_deref() {
        Some("tsv") => parse_tsv(path, name, version),
        Some("zip") => parse_yomitan_zip(path, name, version),
        _ => Err(StudyError::InvalidInput),
    };
    result.map_err(|error| {
        if error == StudyError::InvalidInput {
            StudyError::DictionaryPackageInvalid
        } else {
            error
        }
    })
}

fn parse_tsv(
    path: &Path,
    name: Option<&str>,
    version: Option<&str>,
) -> Result<ParsedDictionaryPackage, StudyError> {
    let name = required_metadata(name, 200)?;
    let version = required_metadata(version, 100)?;
    let metadata = fs::metadata(path).map_err(|_| StudyError::InvalidInput)?;
    if !metadata.is_file() || metadata.len() > MAX_TSV_BYTES {
        return Err(StudyError::InvalidInput);
    }
    let bytes = fs::read(path).map_err(|_| StudyError::InvalidInput)?;
    let content = std::str::from_utf8(&bytes).map_err(|_| StudyError::InvalidInput)?;
    let mut lines = content.lines();
    if lines.next() != Some("expression\treading\tpart_of_speech\tmeaning_vi\than_viet") {
        return Err(StudyError::InvalidInput);
    }
    let mut entries = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if entries.len() >= MAX_ENTRIES || line.chars().count() > 10_000 {
            return Err(StudyError::InvalidInput);
        }
        let columns = line.splitn(5, '\t').collect::<Vec<_>>();
        if columns.len() != 5 {
            return Err(StudyError::InvalidInput);
        }
        entries.push(validated_entry(
            columns[0],
            columns[1],
            columns[2],
            columns[3],
            Some(columns[4]),
            4_000,
        )?);
    }
    finish_package(name, version, fnv1a_checksum(&bytes), entries, 0)
}

fn parse_yomitan_zip(
    path: &Path,
    name_override: Option<&str>,
    version_override: Option<&str>,
) -> Result<ParsedDictionaryPackage, StudyError> {
    let metadata = fs::metadata(path).map_err(|_| StudyError::InvalidInput)?;
    if !metadata.is_file() || metadata.len() > MAX_ZIP_BYTES {
        return Err(StudyError::InvalidInput);
    }
    let checksum = checksum_file(path)?;
    let file = fs::File::open(path).map_err(|_| StudyError::InvalidInput)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| StudyError::InvalidInput)?;
    if archive.is_empty() || archive.len() > MAX_ARCHIVE_FILES {
        return Err(StudyError::InvalidInput);
    }

    let mut total_size = 0u64;
    let mut bank_names = Vec::new();
    for index in 0..archive.len() {
        let member = archive
            .by_index(index)
            .map_err(|_| StudyError::InvalidInput)?;
        total_size = total_size
            .checked_add(member.size())
            .ok_or(StudyError::InvalidInput)?;
        if total_size > MAX_ARCHIVE_UNCOMPRESSED_BYTES {
            return Err(StudyError::InvalidInput);
        }
        let member_name = member.name();
        if is_term_bank_name(member_name) {
            if member.size() > MAX_BANK_BYTES {
                return Err(StudyError::InvalidInput);
            }
            bank_names.push(member_name.to_owned());
        }
    }
    bank_names.sort_by_key(|name| term_bank_number(name).unwrap_or(u32::MAX));
    if bank_names.is_empty() {
        return Err(StudyError::InvalidInput);
    }

    let index: Value = {
        let mut member = archive
            .by_name("index.json")
            .map_err(|_| StudyError::InvalidInput)?;
        if member.size() > 1024 * 1024 {
            return Err(StudyError::InvalidInput);
        }
        serde_json::from_reader(&mut member).map_err(|_| StudyError::InvalidInput)?
    };
    let detected_name = index
        .get("title")
        .and_then(Value::as_str)
        .ok_or(StudyError::InvalidInput)?;
    let detected_version = index
        .get("revision")
        .and_then(Value::as_str)
        .ok_or(StudyError::InvalidInput)?;
    if index
        .get("format")
        .and_then(Value::as_u64)
        .is_none_or(|format| !(1..=3).contains(&format))
    {
        return Err(StudyError::InvalidInput);
    }
    let name = optional_metadata(name_override, detected_name, 200)?;
    let version = optional_metadata(version_override, detected_version, 100)?;

    let mut entries = Vec::new();
    let mut skipped = 0u64;
    for bank_name in bank_names {
        let mut member = archive
            .by_name(&bank_name)
            .map_err(|_| StudyError::InvalidInput)?;
        let bank: Value =
            serde_json::from_reader(&mut member).map_err(|_| StudyError::InvalidInput)?;
        let rows = bank.as_array().ok_or(StudyError::InvalidInput)?;
        if entries
            .len()
            .checked_add(rows.len())
            .is_none_or(|count| count > MAX_ENTRIES)
        {
            return Err(StudyError::InvalidInput);
        }
        for row in rows {
            let columns = row.as_array().ok_or(StudyError::InvalidInput)?;
            if columns.len() < 6 {
                return Err(StudyError::InvalidInput);
            }
            let expression = columns[0].as_str().ok_or(StudyError::InvalidInput)?;
            let reading = columns[1].as_str().ok_or(StudyError::InvalidInput)?;
            let part_of_speech = columns[2].as_str().unwrap_or_default();
            let Some(meaning) = glossary_text(&columns[5])? else {
                skipped += 1;
                continue;
            };
            entries.push(validated_entry(
                expression,
                if reading.trim().is_empty() {
                    expression
                } else {
                    reading
                },
                if part_of_speech.trim().is_empty() {
                    "không xác định"
                } else {
                    part_of_speech
                },
                &meaning,
                None,
                10_000,
            )?);
        }
    }

    finish_package(name, version, checksum, entries, skipped)
}

fn glossary_text(value: &Value) -> Result<Option<String>, StudyError> {
    fn append(value: &Value, output: &mut Vec<String>, depth: usize) -> Result<(), StudyError> {
        if depth > 16 {
            return Err(StudyError::InvalidInput);
        }
        match value {
            Value::String(text) => {
                let text = text.trim();
                if !text.is_empty() {
                    output.push(text.to_owned());
                }
            }
            Value::Array(values) => {
                for value in values {
                    append(value, output, depth + 1)?;
                }
            }
            Value::Object(object) => {
                if let Some(content) = object.get("content") {
                    append(content, output, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    let mut parts = Vec::new();
    append(value, &mut parts, 0)?;
    let result = parts.join("\n");
    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

fn validated_entry(
    expression: &str,
    reading: &str,
    part_of_speech: &str,
    meaning_vi: &str,
    han_viet: Option<&str>,
    maximum_meaning_chars: usize,
) -> Result<ParsedDictionaryEntry, StudyError> {
    let expression = expression.trim();
    let reading = reading.trim();
    let part_of_speech = part_of_speech.trim();
    let meaning_vi = meaning_vi.trim();
    let han_viet = han_viet.map(str::trim).filter(|value| !value.is_empty());
    let values = [expression, reading, part_of_speech, meaning_vi];
    if values.iter().any(|value| value.is_empty())
        || values.iter().any(|value| value.contains('\0'))
        || expression.chars().count() > 200
        || reading.chars().count() > 200
        || part_of_speech.chars().count() > 200
        || meaning_vi.chars().count() > maximum_meaning_chars
        || han_viet.is_some_and(|value| value.contains('\0') || value.chars().count() > 500)
    {
        return Err(StudyError::InvalidInput);
    }
    Ok(ParsedDictionaryEntry {
        expression: expression.to_owned(),
        reading: reading.to_owned(),
        part_of_speech: part_of_speech.to_owned(),
        meaning_vi: meaning_vi.to_owned(),
        han_viet: han_viet.map(str::to_owned),
    })
}

fn required_metadata(value: Option<&str>, maximum: usize) -> Result<String, StudyError> {
    let value = value.ok_or(StudyError::InvalidInput)?;
    optional_metadata(Some(value), "", maximum)
}

fn optional_metadata(
    value: Option<&str>,
    fallback: &str,
    maximum: usize,
) -> Result<String, StudyError> {
    let value = value.unwrap_or(fallback).trim();
    if value.is_empty()
        || value.chars().count() > maximum
        || value.contains('\0')
        || value.chars().any(char::is_control)
    {
        Err(StudyError::InvalidInput)
    } else {
        Ok(value.to_owned())
    }
}

fn finish_package(
    name: String,
    version: String,
    checksum: String,
    entries: Vec<ParsedDictionaryEntry>,
    skipped: u64,
) -> Result<ParsedDictionaryPackage, StudyError> {
    if entries.is_empty() {
        return Err(StudyError::InvalidInput);
    }
    Ok(ParsedDictionaryPackage {
        name,
        version,
        checksum,
        entries,
        skipped,
    })
}

fn is_term_bank_name(name: &str) -> bool {
    name.starts_with("term_bank_")
        && name.ends_with(".json")
        && !name.contains('/')
        && !name.contains('\\')
        && term_bank_number(name).is_some()
}

fn term_bank_number(name: &str) -> Option<u32> {
    name.strip_prefix("term_bank_")?
        .strip_suffix(".json")?
        .parse()
        .ok()
}

fn checksum_file(path: &Path) -> Result<String, StudyError> {
    let mut file = fs::File::open(path).map_err(|_| StudyError::InvalidInput)?;
    let mut hash = 0xcbf29ce484222325u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| StudyError::InvalidInput)?;
        if count == 0 {
            break;
        }
        for byte in &buffer[..count] {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    Ok(format!("fnv1a64:{hash:016x}"))
}

fn fnv1a_checksum(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}
