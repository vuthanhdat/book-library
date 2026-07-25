# Book Library Implementation Plan

## Status

- **Current milestone:** M0 — Engineering foundation
- **Current sprint:** [Sprint 01](sprint-01.md)
- **Implementation state:** not started; the repository currently contains specifications only
- **Feature status source:** [Feature catalog](../docs/01-product/feature-catalog.md)
- **Technical decision source:** [Accepted ADRs](../docs/adr/README.md)

This plan defines delivery order and milestone gates. It does not override product requirements, feature status, or ADRs.

## Planning baseline

The committed architecture assumes:

- Windows 11 desktop first;
- Tauri 2 shell with React and TypeScript presentation;
- domain and application behavior implemented in a Rust modular monolith;
- SQLite, caches, thumbnails, and logs stored in OS application data;
- user books and Markdown notes remain canonical on the filesystem;
- persisted content references use normalized relative paths;
- PDF files and image folders are the first book types;
- OCR, dictionary, AI, Anki, and plugins remain optional.

## Delivery model

Build vertical slices in dependency order. A milestone closes only when its user outcome works end to end, not when isolated components or documents exist.

```mermaid
flowchart LR
    M0["M0 Engineering foundation"] --> M1["M1 Library MVP"]
    M1 --> M2["M2 Reading MVP"]
    M2 --> M3["M3 Knowledge MVP"]
    M3 --> M4["M4 Search MVP"]
    M4 --> M5["M5 Reliability and release"]
    M5 --> M6["M6 Optional intelligence"]
```

Cross-cutting constraints apply to every milestone:

- source books are not modified;
- Markdown note text is not made database-only;
- long-running work reports progress and isolates failures;
- errors are typed and user-safe at the desktop boundary;
- migrations and recovery behavior evolve with stored state;
- documentation describes the actual branch, not intended future work.

## M0 — Engineering foundation

### Outcome

A clean Windows 11 checkout can build and launch a minimal desktop application that proves the architecture, database, typed boundary, tests, and CI.

### Required slice

- scaffold Tauri 2 with React and TypeScript;
- configure Tailwind CSS and shadcn/ui only to the extent needed for the shell;
- create Rust modules `domain`, `application`, `infrastructure`, and `desktop`;
- implement identifiers, `RelativePath`, common errors, and focused tests;
- initialize SQLite in OS app data with a forward-only migration runner;
- expose one typed health/status use case through a thin Tauri command;
- render startup, healthy, failure, and no-library-configured UI states;
- add structured local logging without content leakage;
- add temporary filesystem/SQLite test fixtures;
- add CI for Rust, frontend, build, tests, formatting, linting, and Markdown links.

### Risk-reduction spikes

M0 must produce documented outcomes for:

1. PDFium Rust binding, licensing, native DLL packaging, and page-transfer strategy on Windows;
2. Google Drive Desktop behavior for local files, online-only placeholders, unavailable files, and watcher event bursts;
3. SQLite library, migration mechanism, connection model, and validated journal mode.

Database location and Rust module structure are already decided by ADR-005 and ADR-006 and are not spike questions.

### Exit gate

- app launches on Windows 11 from a clean checkout;
- initial migration creates and reopens the database safely;
- React receives a typed health response from a real application use case;
- domain tests reject absolute and escaping paths and preserve Unicode paths;
- no React module accesses SQLite or source folders directly;
- CI executes all established quality gates;
- spike outcomes are recorded as ADRs or technical reports.

## M1 — Library MVP

### Outcome

The user can select one real folder, scan it, discover PDFs and image-folder books, generate covers, and browse an idempotent catalog.

### Required slice

- select, validate, save, and change one machine-local library root;
- recursively scan with progress, cancellation, exclusions, and per-entry issues;
- discover PDFs and eligible image folders using deterministic natural sorting;
- define explicit policy for mixed folders and nested image folders;
- upsert catalog records without duplicates;
- preserve user metadata while updating derived metadata;
- mark unavailable books instead of deleting history;
- extract basic title, kind, page count, size, and timestamps when available;
- generate rebuildable thumbnails outside source folders;
- browse catalog as a virtualized grid/list;
- support manual rescan and initialization summary.

### Decisions required before implementation

- stable identity during rename: path-only identity or fingerprint-assisted matching;
- exact image-folder eligibility and nested-chapter policy;
- thumbnail format, dimensions, and cache invalidation policy.

### Exit gate

- a real mixed library can be initialized;
- repeated scans are idempotent;
- all persisted book/page paths are relative;
- missing, unreadable, unsupported, and changed candidates produce deterministic outcomes;
- source folders are byte-for-byte unaffected by scanning and thumbnail generation;
- scan failures do not prevent successful books from appearing.

## M2 — Reading MVP

### Outcome

The user can open PDF and image-folder books, navigate smoothly, close the app, and resume from the saved location.

### Required slice

- generic reader session and location contracts;
- production PDFium adapter based on the M0 spike;
- PDF open/render/navigation/zoom/fit/rotation;
- image-folder single-page and continuous modes;
- lazy loading and bounded page cache;
- debounced current reading state;
- recent books and resume reading;
- create, edit, delete, list, and navigate bookmarks;
- user-readable missing, corrupt, unsupported, and password-protected states.

### Exit gate

- both book kinds open from relative paths;
- reopen restores the expected page/location;
- large books do not load all pages into memory;
- reader failures remain isolated and do not crash the app;
- bookmarks survive temporary source unavailability.

## M3 — Knowledge MVP

### Outcome

The user can create durable Markdown notes linked to books and reading locations and continue using those notes in normal editors or Obsidian.

### Required slice

- configure a notes root;
- create and conservatively edit Markdown files;
- associate notes with books, bookmarks, and locations;
- parse CommonMark, YAML frontmatter, headings, tags, Markdown links, and wiki links;
- build a rebuildable SQLite note projection;
- open a note or notes folder externally;
- show basic backlinks;
- reconcile externally edited note files.

### Decisions required before implementation

- default notes-root policy and whether it may be outside the library root;
- internal editor scope versus external-editor-first workflow;
- portable representation of book/location links in Markdown.

### Exit gate

- user-authored note text exists in Markdown, not only SQLite;
- note projections can be deleted and rebuilt without losing note text;
- external edits are detected without destructive rewriting;
- files remain useful in Obsidian and plain text editors.

## M4 — Search MVP

### Outcome

The user can search books, notes, bookmarks, tags, and supported extracted text entirely offline.

### Required slice

- FTS5 schema and search-document projection;
- book, note, bookmark, and tag indexing;
- queued incremental updates after scans and note edits;
- global search UI with result-type filters;
- reindex, repair, and failed-document diagnostics.

### Exit gate

- canonical sources can rebuild all search data;
- changed content appears without a full-library rescan;
- failed documents do not block successful indexing;
- deleting FTS tables does not delete canonical data.

## M5 — Reliability and first release

### Outcome

The application is safe for everyday use on a real library and can be installed and upgraded on Windows.

### Required slice

- debounced filesystem watcher and targeted reconciliation;
- recovery for interrupted scan, thumbnail, and index jobs;
- database backup, integrity check, and rebuild workflow;
- cache cleanup and orphan detection;
- large-library performance suite;
- Windows installer and upgrade validation;
- accessibility and keyboard-navigation pass;
- exportable diagnostic report and release checklist.

### Exit gate

- interrupted work resumes or fails safely;
- database and rebuildable caches can be repaired;
- installer upgrades preserve settings, catalog metadata, notes, and reading state;
- core workflows pass automated integration tests on release configuration.

## M6 — Optional intelligence

### Outcome

Optional modules add OCR and language/AI workflows without changing the reliability or offline availability of the core product.

Candidate slices:

- OCR jobs and per-page text storage;
- offline Japanese dictionary data;
- provider abstraction for explain, translate, summarize, and flashcard drafts;
- explicit acceptance before generated text becomes a canonical note;
- Anki-compatible export;
- trusted in-process module manifest proof of concept.

### Exit gate

- disabling all optional modules leaves M1–M5 workflows unchanged;
- network access and secrets remain isolated behind provider adapters;
- generated content is distinguishable from user-authored canonical content.

## Cross-cutting workstreams

### Data and migrations

- add schema only when required by the current milestone;
- use forward-only migrations and document recovery for risky state changes;
- enforce unique library-relative identities;
- keep thumbnails, OCR text, and FTS data rebuildable;
- never synchronize a live SQLite database as a substitute for metadata sync.

### Testing

- domain unit tests for value objects and policies;
- application tests with fake ports;
- infrastructure tests with temporary folders and SQLite;
- golden tests for path normalization, natural sorting, and Markdown parsing;
- end-to-end smoke tests for each completed milestone outcome.

### Performance

- batch catalog writes during scans;
- bound background concurrency;
- avoid full-file hashing unless identity is ambiguous;
- virtualize large lists and lazy-load pages;
- establish measurable fixture sizes before optimizing.

### Security and privacy

- validate roots and prevent traversal;
- do not execute library content;
- store optional-provider secrets using OS-supported secure storage;
- avoid logging user content or unnecessary absolute paths;
- keep network access absent from the core.

## Release mapping

- `0.1.0`: M0–M2 — engineering foundation, library management, PDF/image reading, progress, and bookmarks.
- `0.2.0`: M3 — Markdown notes and Obsidian interoperability.
- `0.3.0`: M4–M5 — local search, recovery, performance, packaging, and first dependable release.
- later versions: M6 optional intelligence after the core is dependable.

## Definition of done

A feature is done only when acceptance criteria pass, critical tests exist, error/cancellation/recovery behavior is implemented, persistence changes include migration guidance, user-owned files remain safe, and authoritative documentation matches the merged code.