# Book Library Implementation Plan

## Status

- **Completed milestones:** M0–M4 on Windows 11 x64 and macOS Intel x64
- **Latest completed sprint:** [Sprint 05](sprint-05.md)
- **Current implementation milestone:** M6 — Optional intelligence
- **Active sprint:** [Sprint 06](sprint-06.md)
- **M5 status:** Still planned in source/status terms; the maintainer explicitly
  authorized M6 implementation before the M5 exit gate
- **Implementation state:** The cross-platform engineering foundation, library,
  external reading, Markdown knowledge, missing-source recovery, Book Detail,
  and offline search workflows are complete
- **Feature status source:** [Feature catalog](../docs/01-product/feature-catalog.md)
- **Technical decision source:** [Accepted ADRs](../docs/adr/README.md)

This plan defines delivery order and milestone gates. It does not override product requirements, feature status, or ADRs.

## Planning baseline

The committed architecture assumes:

- Windows 11 x64 and macOS Intel x64 are required desktop platforms from M0;
- one Tauri 2 shell with React and TypeScript presentation is shared by both platforms;
- domain and application behavior is implemented once in a platform-independent Rust modular monolith;
- SQLite, caches, thumbnails, and logs are stored in each operating system's application-data location;
- user books and Markdown notes remain canonical on the filesystem;
- persisted content references use normalized relative paths without unconditional lowercasing;
- PDF files and image folders are the first book types;
- OCR, dictionary, AI, Anki, and plugins remain optional;
- Apple Silicon and universal macOS binaries are deferred until explicitly added to the roadmap.

See [ADR-007](../docs/adr/ADR-007-supported-desktop-platforms.md).

## Delivery model

Build vertical slices in dependency order. A milestone closes only when its user outcome works end to end, not when isolated components or documents exist.

```mermaid
flowchart LR
    M0["M0 Engineering foundation"] --> M1["M1 Library MVP"]
    M1 --> M2["M2 External reading workflow"]
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
- shared domain/application behavior contains no Windows-only or macOS-only product fork;
- applicable milestone outcomes are validated on Windows 11 x64 and a real macOS Intel x64 machine;
- documentation describes the actual branch, not intended future work.

## M0 — Engineering foundation

### Outcome

Clean Windows 11 x64 and macOS Intel x64 checkouts can build and launch the same minimal desktop application, proving the architecture, database, typed boundary, tests, and platform validation process.

### Required slice

- scaffold Tauri 2 with React and TypeScript;
- configure Tailwind CSS and shadcn/ui only to the extent needed for the shell;
- create Rust modules `domain`, `application`, `infrastructure`, and `desktop`;
- implement identifiers, cross-platform `RelativePath`, common errors, and focused tests;
- initialize SQLite in OS app data with a forward-only migration runner on both platforms;
- expose one typed health/status use case through a thin Tauri command;
- render startup, healthy, failure, and no-library-configured UI states;
- add structured local logging without content leakage;
- add temporary filesystem/SQLite test fixtures;
- add CI for Rust, frontend, builds, tests, formatting, linting, and Markdown links across supported environments;
- document local build and smoke-test commands for Windows 11 x64 and macOS Intel x64.

### Risk-reduction spikes

M0 must produce documented outcomes for:

1. PDFium Rust binding, licensing, native binary packaging, and page-transfer strategy on Windows x64 and macOS Intel x64;
2. Google Drive Desktop behavior for local files, online-only placeholders, unavailable files, permissions, and watcher event bursts on both operating systems;
3. SQLite library, migration mechanism, connection model, and validated journal/concurrency behavior on both operating systems.

Database location, Rust module structure, and supported platforms are already decided by ADR-005, ADR-006, and ADR-007 and are not spike questions.

### Exit gate

- app launches from clean documented setup on Windows 11 x64;
- app launches from clean documented setup on a real macOS Intel x64 machine;
- initial migration creates and reopens the database safely in each OS application-data location;
- React receives a typed health response from a real application use case;
- domain tests reject Windows and POSIX absolute/escaping paths and preserve Unicode paths;
- no React module accesses SQLite or source folders directly;
- CI executes all established quality gates without claiming platform coverage it does not actually run;
- spike outcomes are recorded as ADRs or technical reports for both platforms.

## M1 — Library MVP

### Outcome

The user can select one real folder, scan it, discover PDFs and image-folder books, generate covers, and browse an idempotent catalog on Windows and macOS Intel.

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
- support manual rescan and initialization summary;
- handle platform-specific hidden files, symlinks, permissions, case collisions, and Google Drive availability behind shared scanner ports.

### Accepted implementation policy

M1 identity, case comparison, symlink containment, image-folder eligibility,
and thumbnail policy are settled by
[ADR-008](../docs/adr/ADR-008-m1-library-policies.md).

### Exit gate

- a real mixed library can be initialized on both supported platforms;
- repeated scans are idempotent;
- all persisted book/page paths are relative;
- missing, unreadable, unsupported, changed, and platform-specific unavailable candidates produce deterministic outcomes;
- source folders are byte-for-byte unaffected by scanning and thumbnail generation;
- scan failures do not prevent successful books from appearing.

## M2 — External reading workflow

### Outcome

The user can find a cataloged book immediately and open its source directory in
Windows Explorer or macOS Finder, then choose an external reading application.

### Required slice

- application use case resolving book ID to an authorized directory;
- platform file-manager adapter behind a shared port;
- PDF parent-directory and image-folder selection rules;
- typed missing, unavailable, invalid-ID, and launch errors;
- explicit catalog actions in grid and list views;
- live Unicode-aware filtering of catalog metadata;
- no source content modification or network dependency.

### Exit gate

- PDF and image-folder locations open correctly on both supported platforms;
- React passes only a book ID and never opens an arbitrary path;
- missing or unresolved records produce a recoverable state;
- live search remains responsive on the validated large catalog;
- external-open and search code do not modify source content;
- full embedded reading, progress, and bookmarks remain explicitly deferred under
  ADR-009.

## M3 — Knowledge MVP

### Outcome

The user can create durable Markdown notes linked to cataloged books and continue
using those notes in normal editors or Obsidian.

### Required slice

- configure a notes root;
- create and conservatively edit Markdown files;
- associate notes with books;
- parse CommonMark, YAML frontmatter, headings, tags, Markdown links, and wiki links;
- build a rebuildable SQLite note projection;
- open a note or notes folder externally;
- show basic backlinks;
- reconcile externally edited note files.

### Decisions required before implementation

- default notes-root policy and whether it may be outside the library root;
- internal editor scope versus external-editor-first workflow;
- portable representation of book links in Markdown.

### Exit gate

- user-authored note text exists in Markdown, not only SQLite;
- note projections can be deleted and rebuilt without losing note text;
- external edits are detected without destructive rewriting;
- files remain useful in Obsidian and plain text editors on both supported platforms.

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
- deleting FTS tables does not delete canonical data;
- equivalent fixtures produce consistent searchable outcomes on both supported platforms.

## M5 — Reliability and first release

### Outcome

The application is safe for everyday use on a real library and can be installed and upgraded on Windows 11 x64 and macOS Intel x64.

### Required slice

- debounced filesystem watcher and targeted reconciliation on both operating systems;
- recovery for interrupted scan, thumbnail, and index jobs;
- database backup, integrity check, and rebuild workflow;
- cache cleanup and orphan detection;
- large-library performance suite;
- Windows installer and upgrade validation;
- macOS Intel `.app`/`.dmg` packaging, native-library signing, code signing, notarization when required for distribution, and upgrade validation;
- accessibility and keyboard-navigation pass;
- exportable diagnostic report and per-platform release checklist.

### Exit gate

- interrupted work resumes or fails safely on both supported platforms;
- database and rebuildable caches can be repaired;
- installer or application upgrades preserve settings, catalog metadata, notes, and reading state;
- Windows release artifacts pass a clean-machine install/upgrade test;
- macOS Intel release artifacts pass Gatekeeper/distribution checks appropriate to the chosen release method;
- core workflows pass automated integration tests and real release-configuration smoke tests on both targets.

## M6 — Optional intelligence

### Outcome

Optional modules turn selected book pages and Japanese text into an offline
lookup, note, search, and flashcard workflow without changing the reliability or
offline availability of the core product.

### Entry gate and sequencing exception

- The normal roadmap requires the M5 release, backup, job-recovery, diagnostics,
  and cross-platform packaging gates first. For Sprint 06, the maintainer
  explicitly waived this sequencing dependency for M6 implementation only. The
  waiver does not mark any unimplemented M5 feature complete and does not waive
  M5 before a dependable public release;
- a provider/data licensing spike selects distributable Japanese dictionary,
  Kanji, Vietnamese/Hán-Việt, tokenizer, and OCR inputs;
- any new native runtime or model packaging choice is accepted in an ADR before
  production implementation;
- optional-module settings can disable every M6 adapter without migration or
  startup failure in the core.

### Delivery order

#### M6-A — Offline Japanese dictionary

- import versioned local dictionary packages into rebuildable app-data indexes;
- normalize entries behind application-owned dictionary and Japanese-analysis
  ports;
- support manual word, reading, and Kanji lookup before any OCR dependency;
- include reading, part of speech, senses, Kanji metadata, and licensed
  Vietnamese/Hán-Việt data when available;
- keep lookup history disabled by default and provide explicit clear-history
  behavior.

#### M6-B — Explicit page OCR

- render or load one explicitly selected PDF/image page for OCR or the bounded
  Study Reader without starting whole-book background work;
- run a cancellable local OCR job only after a user request;
- persist derived page text, confidence, blocks, bounding boxes, provider
  version, and source fingerprint in app data;
- enqueue completed OCR text into the existing rebuildable FTS5 pipeline;
- allow retry, delete, and rebuild without modifying the source book.

#### M6-C — Japanese learning workflow

- select an OCR block or paste Japanese text into the same lookup use case;
- open a bounded single-page Study Reader for cataloged PDF and image-folder
  books while retaining external opening;
- navigate explicitly requested pages, zoom the presentation, and keep local
  dictionary results adjacent to selectable saved OCR text;
- show token boundaries as suggestions and allow the user to correct the lookup
  term;
- create an editable Markdown-note insertion or flashcard draft with book
  relative path, page index, dictionary provenance, and optional bounded image
  crop;
- export only explicitly approved drafts as UTF-8 TSV for Anki import.

#### M6-D — Optional AI and module proof

- add an isolated provider boundary and secure settings for explicitly
  configured local or remote models;
- keep conversations ephemeral by default and show the exact context before any
  remote request;
- return explanations, translations, summaries, and cards as drafts that require
  explicit acceptance;
- validate a minimal trusted in-process module manifest without exposing an
  untrusted plugin marketplace.

### Exit gate

- disabling all optional modules leaves M1–M5 workflows unchanged;
- manual Japanese lookup, one-page OCR, OCR-to-lookup, note/card drafting, and
  TSV export pass on Windows 11 x64 and macOS Intel x64;
- normal library scanning never runs OCR, hydrates cloud files, or loads
  dictionary/model runtimes implicitly;
- OCR, dictionary, and AI failures are isolated per provider/job and cannot
  corrupt canonical books or Markdown notes;
- deleting OCR text and dictionary indexes removes no canonical content and both
  can be rebuilt from their declared sources;
- packaged dictionary/model licenses, versions, checksums, and update behavior
  are documented;
- network access and secrets remain isolated behind provider adapters;
- remote requests require explicit configuration and visible user context;
- generated content is distinguishable from user-authored canonical content and
  is never written or exported without explicit acceptance.

## Cross-cutting workstreams

### Data and migrations

- add schema only when required by the current milestone;
- use forward-only migrations and document recovery for risky state changes;
- enforce unique library-relative identities without assuming every filesystem is case-insensitive;
- keep thumbnails, OCR text, and FTS data rebuildable;
- never synchronize a live SQLite database as a substitute for metadata sync.

### Testing

- domain unit tests for value objects and policies;
- application tests with fake ports;
- infrastructure tests with temporary folders and SQLite;
- golden tests for Windows/POSIX path normalization, natural sorting, and Markdown parsing;
- platform-specific tests for app-data, symlinks, permissions, watchers, Google Drive behavior, and native PDFium loading;
- end-to-end smoke tests on Windows 11 x64 and a real macOS Intel x64 machine for each completed milestone outcome.

### Performance

- batch catalog writes during scans;
- bound background concurrency;
- avoid full-file hashing unless identity is ambiguous;
- virtualize large lists and lazy-load pages;
- establish measurable fixture sizes before optimizing;
- record platform-specific performance differences instead of hiding them behind one threshold.

### Security and privacy

- validate roots and prevent traversal or symlink escape;
- do not execute library content;
- store optional-provider secrets using OS-supported secure storage;
- avoid logging user content or unnecessary absolute paths;
- keep network access absent from the core;
- sign and notarize distributed native artifacts according to the target platform's release model.

## Release mapping

- `0.1.0`: M0–M2 — dual-platform engineering foundation, library management,
  live catalog filtering, and authorized source-location opening through the OS
  file manager.
- `0.2.0`: M3 — Markdown notes and Obsidian interoperability.
- `0.3.0`: M4–M5 — local search, recovery, performance, Windows/macOS Intel packaging, and first dependable release.
- later versions: M6 optional intelligence and any explicitly approved additional platform targets after the core is dependable.

## Definition of done

A feature is done only when acceptance criteria pass on the applicable supported platforms, critical tests exist, error/cancellation/recovery behavior is implemented, persistence changes include migration guidance, user-owned files remain safe, and authoritative documentation matches the merged code.
