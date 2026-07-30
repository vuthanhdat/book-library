# Sprint 06 — Offline Japanese study and optional intelligence

## Status

- **State:** In Progress
- **Milestone:** M6 — Optional intelligence
- **Platforms:** Windows implementation and automated gates first; macOS Intel
  validation required before feature completion
- **Feature IDs:** OCR-001, OCR-002, DICT-001 through DICT-003, AI-001 through
  AI-004, ANKI-001, PLUGIN-001
- **Sequencing:** explicitly authorized by the maintainer before M5 completion;
  absent M5 source remains `Planned`

## Goal

Provide a useful offline Japanese study workflow: manually look up Japanese,
explicitly OCR one authorized book page, search and reuse derived text, create
reviewable learning drafts, and export approved cards without making core
library or Markdown workflows depend on any optional runtime.

## Delivered source scope

1. Migration 6 adds disabled module settings, dictionary package/entry indexes,
   lookup history, OCR pages/blocks, learning drafts, Anki export history, and
   assistant outputs.
2. A small original CC0 Japanese–Vietnamese starter package supports immediate
   offline lookup.
3. Bounded transactional importers accept UTF-8 TSV and user-provided Yomitan
   ZIP packages while preserving package metadata and license provenance.
4. Application-owned study use cases validate module enablement, text, tags,
   page identity, draft approval, and export state.
5. Longest-known-term suggestions share the same lookup path as manual and OCR
   text.
6. One PDF or image page is resolved by book ID, canonicalized beneath the
   library root, and materialized to app-data PNG without reader state.
7. A Tesseract CLI adapter parses Japanese/English TSV text, confidence, and
   word bounds; the job supports explicit cancellation and retries
   low-confidence pages with the Japanese vertical model before selecting the
   better result.
8. OCR pages and blocks are rebuildable SQLite projections and join explicit
   FTS5 rebuilds under the `ocr` scope.
9. Dictionary and OCR context create explicit learning drafts. Only approved
   drafts export to a new UTF-8 TSV file.
10. A disabled offline assistant provider produces visibly labeled explanation,
    translation, summary, and flashcard drafts.
11. Trusted module manifests expose identity, compatibility, capabilities, and
    permissions.
12. The React Study workspace controls modules, lookup, OCR, draft review, TSV
    export, assistant drafts, and manifest inspection. Its wide-screen layout
    uses the available desktop width, and selecting OCR text triggers dictionary
    lookup directly.

## Acceptance currently passing

- core startup and existing M0–M4 tests pass with every M6 module disabled;
- migration 6 applies idempotently and seeds only app-data;
- Japanese starter lookup and user TSV/Yomitan ZIP imports preserve Unicode;
- unsafe/oversized dictionary rows are rejected before a transaction commits;
- Yomitan members are read without extraction and large-package token
  candidates are filtered without loading the whole dictionary into memory;
- OCR source containment is rechecked after canonicalization;
- OCR parser tests preserve Japanese text, confidence, and bounds;
- cancellation terminates the child OCR process without changing source files;
- OCR content appears in canonical documents used to rebuild FTS5;
- drafts require explicit approval before export;
- TSV export preserves UTF-8, escapes tabs/newlines, and refuses overwrite;
- frontend Study workspace renders enabled, disabled, and unavailable module
  states;
- Rust tests, Clippy, TypeScript checks, frontend tests, production build, and
  Markdown links pass on the current Windows workspace.

## Remaining before M6 completion

- select and record a representative Japanese OCR corpus with horizontal,
  vertical, furigana, manga, low-contrast, and mixed-script fixtures;
- install or bundle a redistributable Japanese OCR runtime and validate real
  image-folder and scanned-PDF results;
- measure accuracy, latency, cancellation, memory, and package size;
- decide whether the first release bundles Tesseract or replaces it with a
  neural provider;
- validate the full Study workflow and packaging on a real macOS Intel machine;
- add secure OS-backed secret storage before any remote AI provider;
- complete M5 recovery/diagnostic/package work before calling the combined
  product a dependable public release.

## Explicit non-goals

- automatic OCR during normal scans;
- general embedded reader navigation, reading progress, or PDF annotations;
- hidden network requests or remote provider defaults;
- representing the starter dictionary as a comprehensive Japanese dictionary;
- automatic Markdown writes or Anki export;
- untrusted plugin loading or a marketplace.
