# Book Library Implementation Plan

## 1. Planning basis

This plan converts the existing product, architecture, library, reader, notes, search, AI, and ADR documents into an executable delivery sequence.

The plan follows these constraints:

- Windows 11 desktop first.
- Tauri 2 shell with React and TypeScript UI.
- Rust application and infrastructure layers.
- SQLite stores metadata, indexes, jobs, and reading state.
- The filesystem is the source of truth for books and Markdown notes.
- Persisted filesystem references use relative paths only.
- PDF and image-folder books are the first supported book types.
- AI, OCR, dictionary, Anki, and plugin capabilities remain optional and are not MVP dependencies.

## 2. Delivery strategy

Build vertical slices that become usable at the end of every milestone. Establish the application shell, domain contracts, migration system, and error model first. Then deliver library initialization, browsing, readers, notes, search, and optional intelligence in dependency order.

```mermaid
flowchart LR
    M0["M0 Foundation"] --> M1["M1 Library MVP"]
    M1 --> M2["M2 Reading MVP"]
    M2 --> M3["M3 Knowledge MVP"]
    M3 --> M4["M4 Search MVP"]
    M4 --> M5["M5 Reliability and Release"]
    M5 --> M6["M6 Optional Intelligence"]
```

## 3. Milestones

### M0 — Engineering foundation

Goal: produce a runnable desktop shell with enforceable architecture boundaries.

Deliverables:

- Tauri 2 application scaffold.
- React, TypeScript, Tailwind, and Shadcn UI setup.
- Rust workspace or module layout aligned with the documented dependency map.
- SQLite connection, migration runner, and transaction abstraction.
- Typed Tauri command and event contracts.
- Common domain error model and user-safe error mapping.
- Logging and local diagnostics.
- Test fixtures for temporary library folders.
- CI for Rust, TypeScript, tests, formatting, and Markdown validation.

Exit criteria:

- App starts on Windows 11.
- A migration creates and upgrades a local SQLite database.
- React can call one typed health/status command.
- Architecture and test conventions are documented in `AGENTS.md` or contributor guidance.

### M1 — Library MVP

Goal: choose a folder, scan it, discover books, generate covers, persist metadata, and browse the catalog.

Deliverables:

- Library-root selection and validation.
- Relative-path value object and path normalization.
- Recursive scanner with cancellation and progress reporting.
- PDF detector.
- Image-folder detector using natural page sorting.
- Discovery and upsert workflow.
- Missing-book reconciliation without destructive deletion.
- Metadata extraction for title, kind, page count, file size, and timestamps.
- Thumbnail generation and cache.
- Library grid/list UI with cover, title, type, and path.
- Manual rescan.
- Initialization summary and scan-issue report.

Exit criteria:

- A real folder containing PDFs and image folders can be initialized.
- Re-running initialization does not duplicate books.
- Renamed, changed, missing, unsupported, and unreadable entries produce deterministic outcomes.
- Source files are never modified.
- All stored paths are relative.

### M2 — Reading MVP

Goal: open both supported book types and resume reading reliably.

Deliverables:

- Generic reader session contract.
- PDFium integration spike followed by production adapter.
- PDF page rendering, navigation, zoom, fit modes, and rotation.
- Image-folder single-page and continuous modes.
- Lazy page loading and bounded cache.
- Current reading state with debounced persistence.
- Recent books and resume-reading UI.
- Bookmarks with optional title and note.
- Graceful missing, corrupt, password-protected, and unreadable-book states.

Exit criteria:

- PDF and image-folder books open from relative paths.
- Closing and reopening restores the last location.
- Large books do not load all pages into memory.
- Reader failures do not crash the desktop app.

### M3 — Knowledge MVP

Goal: create durable Markdown notes associated with books and reading locations.

Deliverables:

- Configurable notes root.
- Markdown note creation and conservative editing.
- Book-note and bookmark-note associations.
- CommonMark parsing with YAML frontmatter, headings, tags, Markdown links, and wiki links.
- SQLite note projection that can be rebuilt.
- Open note externally and open notes folder in Obsidian.
- Basic backlinks view.
- File watcher support for externally edited notes.

Exit criteria:

- User-authored note content lives in Markdown files, not SQLite.
- Notes remain readable in Obsidian and a normal text editor.
- Deleting the SQLite note projection does not destroy note content.

### M4 — Search MVP

Goal: provide fast local search across books, metadata, notes, bookmarks, and supported extracted text.

Deliverables:

- FTS5 schema and migrations.
- Search-document projection and indexing queue.
- Book, note, bookmark, and tag indexing.
- Global search UI with filters and result types.
- Reindex and repair commands.
- Incremental updates after scans and note edits.
- Search diagnostics for failed documents.

Exit criteria:

- Search works fully offline.
- Indexes can be deleted and rebuilt from canonical sources.
- Search updates after book and Markdown changes without a full rescan.

### M5 — Reliability and first release

Goal: make the product safe for everyday use on a real library.

Deliverables:

- Filesystem watcher with event debouncing and targeted reconciliation.
- Startup recovery for interrupted scan, thumbnail, and index jobs.
- Database backup and rebuild workflow.
- Cache cleanup and orphan detection.
- Performance tests using large fixture libraries.
- Windows installer and upgrade validation.
- Accessibility and keyboard-navigation pass.
- Exportable diagnostic report.
- Release checklist and user documentation.

Exit criteria:

- Interrupted work resumes or fails safely.
- Database and caches are rebuildable.
- Installer upgrades preserve settings, metadata, notes, and reading state.
- Core workflows are covered by automated integration tests.

### M6 — Optional intelligence

Goal: add value without making network or AI dependencies part of the core.

Candidate slices:

- OCR jobs and per-page text storage.
- Offline Japanese dictionary using JMdict/KANJIDIC data.
- AI provider abstraction for explain, translate, summarize, and flashcard drafting.
- Explicit user acceptance before AI output becomes a canonical note.
- Anki-compatible export.
- Plugin manifest and permission model proof of concept.

Exit criteria:

- All optional modules can be disabled without affecting library, reader, notes, or search.
- Network access and API keys are isolated behind provider modules.

## 4. Cross-cutting workstreams

### Data and migrations

- Define initial schema only for the current milestone.
- Add forward-only migrations with backup guidance.
- Enforce unique `(library_id, relative_path)` identities.
- Prohibit persisted absolute book and note paths.
- Treat thumbnails, OCR text, and FTS records as rebuildable derivatives.

### Testing

- Unit tests for value objects, natural sorting, discovery policies, and reader-independent state.
- Integration tests with temporary filesystem fixtures and SQLite.
- Golden tests for Markdown parsing and path normalization.
- End-to-end smoke tests for initialize, browse, open, resume, note, and search.

### Performance

- Batch database writes during scan.
- Bound concurrency for metadata and thumbnail jobs.
- Avoid full-file hashing by default; begin with size and modified time, then hash when identity is ambiguous.
- Virtualize large library and page lists.

### Security and safety

- Validate selected roots and prevent relative-path traversal.
- Do not execute content from the library.
- Avoid destructive source-file operations in the MVP.
- Store secrets only for optional providers and use OS-supported secure storage.

## 5. Architecture spikes before implementation

Resolve these questions early and record outcomes as ADRs:

1. PDFium Rust binding, native binary packaging, and render transfer strategy on Windows.
2. SQLite location: application data versus a hidden directory inside the library root.
3. Notes-root default and whether it may be outside the library root.
4. Stable identity across file rename: path identity only versus fingerprint-assisted matching.
5. Initial Rust module layout: workspace crates versus modules within one crate.
6. Filesystem watcher reliability on Google Drive Desktop placeholders and sync events.

## 6. Definition of done

A feature is done only when:

- Acceptance criteria pass.
- Domain and application tests exist for critical rules.
- Error and cancellation behavior is implemented.
- Migrations and rollback/recovery notes are included when data changes.
- User-visible progress is reported for long-running operations.
- Documentation and relevant ADRs are updated.
- No absolute source path is persisted.
- Source books and user Markdown notes are not destructively modified.

## 7. Recommended release scope

Version `0.1.0` should include M0 through M2: foundation, library initialization and browsing, PDF/image reading, bookmarks, and resume reading.

Version `0.2.0` should include Markdown notes and Obsidian interoperability.

Version `0.3.0` should include FTS5 search and reliability hardening.

Optional intelligence should begin only after the core product is dependable for daily reading.