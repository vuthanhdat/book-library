# Feature Catalog

## Status

This catalog is the authoritative inventory of product capabilities, implementation status, and milestone ownership.

The repository contains a shared Windows 11 x64 and macOS Intel x64
implementation through M4. No feature may be marked `In Progress` or `Completed`
until implementation work exists on an active branch and satisfies the status
rules below.

## Feature status

| Status | Meaning |
|---|---|
| Draft | The capability is being explored and is not implementation-ready. |
| Planned | The capability is accepted for a future milestone but not scheduled for the active sprint. |
| Ready | Scope, dependencies, acceptance criteria, and blocking decisions are sufficient to start. |
| In Progress | Implementation exists on an active branch or pull request. |
| Completed | Implementation is merged, required checks pass, and documentation matches the code. |
| Deferred | Explicitly outside the committed roadmap or postponed until a trigger is met. |

Design documents alone never justify `In Progress` or `Completed`.

## Milestones

| Milestone | Outcome |
|---|---|
| M0 — Engineering foundation | Runnable Windows 11 x64 and macOS Intel x64 Tauri/React/Rust shell, architecture boundaries, SQLite migrations, typed health flow, tests, and CI. |
| M1 — Library MVP | Configure one root, scan PDFs and image folders, reconcile catalog records, generate thumbnails, and browse the library on both supported platforms. |
| M2 — External reading workflow | Find books live and open their source locations through the OS file manager on both supported platforms. |
| M3 — Knowledge MVP | Create portable Markdown notes, associate them with books/locations, and support Obsidian interoperability. |
| M4 — Search MVP | Search books, notes, bookmarks, tags, and supported extracted text using rebuildable local indexes. |
| M5 — Reliability and release | Watch/recover/rebuild safely, validate large libraries, package Windows and macOS Intel releases, and protect upgrades. |
| M6 — Optional intelligence | Add OCR, dictionary, AI, Anki, and plugin experiments without creating core dependencies. |

## Engineering foundation

| ID | Feature | Status | Milestone |
|---|---|---|---|
| ENG-001 | Cross-platform Tauri 2 + React + TypeScript application scaffold | Completed | M0 |
| ENG-002 | Rust modular-monolith structure and dependency boundaries | Completed | M0 |
| ENG-003 | Typed Tauri command/event contracts | Completed | M0 |
| ENG-004 | SQLite connection and migration runner | Completed | M0 |
| ENG-005 | Domain identifiers, `RelativePath`, and common errors | Completed | M0 |
| ENG-006 | Structured logging and safe diagnostics | Completed | M0 |
| ENG-007 | Windows/macOS Rust, frontend, build, and Markdown CI quality gates | Completed | M0 |
| ENG-008 | Temporary library and SQLite test fixtures | Completed | M0 |

## Library

| ID | Feature | Status | Milestone |
|---|---|---|---|
| LIB-001 | Configure and initialize one library root | Completed | M1 |
| LIB-002 | Recursive scan with progress and cancellation | Completed | M1 |
| LIB-003 | Discover PDF books | Completed | M1 |
| LIB-004 | Discover image-folder books with natural page ordering | Completed | M1 |
| LIB-005 | Idempotent catalog upsert and reconciliation | Completed | M1 |
| LIB-006 | Detect changed and newly added books | Completed | M1 |
| LIB-007 | Mark unavailable/missing books without destructive deletion | Completed | M1 |
| LIB-008 | Extract core book metadata | Completed | M1 |
| LIB-009 | Generate rebuildable thumbnails | Completed | M1 |
| LIB-010 | Browse catalog as grid/list | Completed | M1 |
| LIB-011 | Manual rescan and projection repair | Completed | M1 |
| LIB-012 | Favorite books | Deferred | Post-MVP |
| LIB-013 | Multiple libraries | Deferred | Post-MVP |
| LIB-014 | Realtime catalog filtering by title, path, kind, and status | Completed | M2 |
| LIB-015 | Edit app-local book display title without changing source files | Completed | M2 |
| LIB-016 | Open nearest existing authorized folder for a missing book | Completed | M4 |
| LIB-017 | Explicitly relink a missing source inside the configured library | Completed | M4 |
| LIB-018 | Book Detail with reading status, book tags, linked Markdown notes, and explicit cover retry | Completed | M4 |

## Reader and reading state

| ID | Feature | Status | Milestone |
|---|---|---|---|
| READ-001 | Open a book's source location in the OS file manager | Completed | M2 |
| READ-002 | Open and render PDF books through PDFium adapter | Deferred | Post-MVP |
| READ-003 | Open image-folder books in an embedded reader | Deferred | Post-MVP |
| READ-004 | Next/previous page and direct page navigation | Deferred | Post-MVP |
| READ-005 | Single-page and continuous image reading modes | Deferred | Post-MVP |
| READ-006 | Zoom, fit width, fit height, and rotation | Deferred | Post-MVP |
| READ-007 | Lazy loading and bounded page cache | Deferred | Post-MVP |
| READ-008 | Fullscreen and keyboard shortcuts | Deferred | Post-MVP |
| READ-009 | User-readable source-location availability errors | Completed | M2 |
| PROG-001 | Debounced automatic progress save | Deferred | Post-MVP |
| PROG-002 | Resume from last reading location | Deferred | Post-MVP |
| PROG-003 | Recent books | Deferred | Post-MVP |
| BOOKMARK-001 | Add bookmark at current location | Deferred | Post-MVP |
| BOOKMARK-002 | Edit/delete bookmark | Deferred | Post-MVP |
| BOOKMARK-003 | Per-book bookmark list and navigation | Deferred | Post-MVP |
| PROG-004 | Reading statistics and goals | Deferred | Post-MVP |

## Notes and knowledge

| ID | Feature | Status | Milestone |
|---|---|---|---|
| NOTE-001 | Configure notes root | Completed | M3 |
| NOTE-002 | Create and conservatively edit Markdown notes | Completed | M3 |
| NOTE-003 | Associate notes with books | Completed | M3 |
| NOTE-004 | Parse headings, tags, links, and YAML frontmatter | Completed | M3 |
| NOTE-005 | Rebuildable SQLite note projection | Completed | M3 |
| NOTE-006 | Open note/folder in external editor or Obsidian | Completed | M3 |
| NOTE-007 | Basic backlinks | Completed | M3 |
| NOTE-008 | Reconcile externally edited notes | Completed | M3 |
| NOTE-009 | Graph view | Deferred | Post-MVP |

## Search

| ID | Feature | Status | Milestone |
|---|---|---|---|
| SEARCH-001 | FTS5 schema and search-document projection | Completed | M4 |
| SEARCH-002 | Search books and bibliographic metadata | Completed | M4 |
| SEARCH-003 | Search notes | Completed | M4 |
| SEARCH-004 | Search tags | Completed | M4 |
| SEARCH-005 | Global search UI with result-type filters | Completed | M4 |
| SEARCH-006 | Incremental indexing after scans/note edits | Completed | M4 |
| SEARCH-007 | Reindex, repair, and failed-document diagnostics | Completed | M4 |
| SEARCH-008 | Semantic/vector search | Deferred | Post-MVP |

## Reliability and release

| ID | Feature | Status | Milestone |
|---|---|---|---|
| REL-001 | Debounced filesystem watcher and targeted reconciliation | Planned | M5 |
| REL-002 | Persistent job recovery after restart | Planned | M5 |
| REL-003 | Database backup and rebuild workflow | Planned | M5 |
| REL-004 | Cache cleanup and orphan detection | Planned | M5 |
| REL-005 | Large-library performance suite | Planned | M5 |
| REL-006 | Windows installer plus macOS Intel app/DMG packaging, signing, and upgrade validation | Planned | M5 |
| REL-007 | Accessibility and keyboard-navigation pass | Planned | M5 |
| REL-008 | Exportable diagnostic report | Planned | M5 |

## Optional intelligence

| ID | Feature | Status | Milestone |
|---|---|---|---|
| OCR-001 | OCR image pages | In Progress | M6 |
| OCR-002 | OCR scanned PDF pages | In Progress | M6 |
| DICT-001 | Offline Japanese dictionary lookup | In Progress | M6 |
| DICT-002 | Vietnamese meaning and Sino-Vietnamese reading | In Progress | M6 |
| DICT-003 | Kanji lookup | In Progress | M6 |
| DICT-004 | Pitch-accent data | Deferred | Post-MVP |
| AI-001 | Provider abstraction and secure configuration | In Progress | M6 |
| AI-002 | Explain and translate selected content | In Progress | M6 |
| AI-003 | Summary and note-draft suggestions | In Progress | M6 |
| AI-004 | Flashcard drafting with explicit user acceptance | In Progress | M6 |
| ANKI-001 | Anki-compatible export | In Progress | M6 |
| PLUGIN-001 | Trusted in-process module manifest proof of concept | In Progress | M6 |
| PLUGIN-002 | Untrusted plugin sandbox/marketplace | Deferred | Post-MVP |

## Explicit non-goals for the committed roadmap

- hosted web application;
- online account requirement;
- Google Drive API integration;
- multi-user collaboration;
- DRM circumvention;
- ebook store or marketplace;
- social features;
- in-place modification of PDF annotations;
- mobile clients;
- Apple Silicon or universal macOS binaries before an explicit roadmap decision.

## Status maintenance

- The active sprint may move a `Ready` feature to `In Progress` when a real implementation branch begins.
- A merged implementation may move to `Completed` only after acceptance criteria and required checks pass.
- When a milestone changes, update this catalog and the implementation plan in the same pull request.
- Deferred capabilities need an explicit trigger before returning to `Planned`.
