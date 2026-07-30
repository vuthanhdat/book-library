# ADR-014: Start M6 with an isolated offline Japanese study baseline

- **Status:** Accepted
- **Date:** 2026-07-30

## Context

M6 needs a useful Japanese-learning path without making core catalog, notes, or
search behavior depend on a model runtime, network provider, proprietary data,
or an embedded reader. A full Japanese neural OCR and dictionary distribution
would add large native/model assets and redistribution obligations before
representative cross-platform measurements exist.

The maintainer authorized M6 implementation before the normal M5 sequencing
gate. This authorization does not provide missing M5 source or validation.

## Decision

Implement the first M6 baseline as disabled-by-default optional modules:

- ship a small original CC0 Japanese–Vietnamese starter dictionary and support
  explicit import of versioned user-provided UTF-8 TSV packages; ADR-015 later
  extends this boundary to user-provided Yomitan ZIP packages;
- provide deterministic longest-known-term suggestions that the user can
  correct, rather than claiming full morphological analysis;
- materialize only one explicitly selected authorized PDF/image page for OCR;
- use a `TesseractOcrProvider` CLI adapter for the first OCR integration, with
  `BOOK_LIBRARY_TESSERACT` as an optional executable override;
- report OCR as unavailable when the executable or Japanese language data is
  absent; do not fail startup or normal scans;
- store OCR text/blocks as rebuildable app-data projections and include them in
  explicit FTS5 rebuilds;
- create explicit learning drafts and export approved drafts to a new UTF-8 TSV
  file without overwriting an existing file;
- provide an offline built-in study-draft provider behind the AI port while
  leaving remote providers and secret storage unimplemented;
- expose trusted built-in module manifests with declared capabilities and
  permissions.

## Considered options

### Bundle a neural Japanese OCR runtime immediately

Deferred until representative horizontal, vertical, furigana, manga, and scan
fixtures establish accuracy, latency, model size, license, and packaging
requirements on Windows x64 and macOS Intel x64.

### Require a remote OCR or AI service

Rejected because it would break offline-first behavior and add content upload,
secret, privacy, and availability dependencies.

### Treat the starter dictionary as comprehensive

Rejected. It is explicitly a functional starter and import-format fixture. The
UI must not imply that a missing entry means a Japanese term does not exist.

### Store generated learning content directly in Markdown

Rejected. OCR, assistant outputs, and flashcards remain drafts until an explicit
write or export action.

## Consequences

- Dictionary lookup, draft review, and TSV export work with no network or
  external runtime.
- OCR is operational only when a compatible Tesseract executable with Japanese
  data is installed; this keeps failure isolated but prevents OCR features from
  reaching `Completed` until runtime packaging and both-platform evidence exist.
- The built-in assistant proves provider boundaries and draft ownership but is
  not represented as a general language model.
- Imported dictionary packages remain the user's responsibility; the app stores
  their declared license/provenance and does not grant redistribution rights.

## Implementation constraints

- Source paths are resolved from book identifiers and rechecked after
  canonicalization beneath the configured library root.
- Normal scan, search, notes, and startup flows never launch OCR.
- Optional modules remain disabled by default.
- No note body, OCR text, selected context, or API secret is logged.
- Tesseract output is parsed as UTF-8 TSV with per-block confidence and bounds.
- Package imports are bounded, validated, transactional, and never read from a
  source-book directory implicitly.
- Anki export uses a user-selected new `.tsv` file and refuses silent overwrite.

## Revisit when

Revisit the OCR provider after the M6 corpus benchmark selects a distributable
Japanese neural runtime or packaged Tesseract baseline for both supported
platforms. Add a separate ADR before introducing remote providers, secret
storage, or automatic content transmission.
