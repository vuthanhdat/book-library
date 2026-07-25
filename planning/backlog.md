# Product Backlog

## Purpose and authority

This backlog groups catalog features into deliverable engineering slices and records priority/dependency order.

- [Feature catalog](../docs/01-product/feature-catalog.md) owns feature IDs, milestone assignment, and implementation status.
- [Implementation plan](implementation-plan.md) owns milestone outcomes and gates.
- The active sprint owns detailed tasks and acceptance criteria for work that is ready to start.
- Module specifications own subsystem behavior.

Do not create a second feature ID in this backlog. A backlog item may group several catalog IDs into one vertical slice.

## Priority model

| Priority | Meaning |
|---|---|
| P0 | Required for the current milestone outcome. |
| P1 | Important quality, recovery, or workflow capability within the milestone. |
| P2 | Valuable enhancement after the milestone's core path works. |
| P3 | Optional intelligence or exploratory work. |

Priority is meaningful inside a milestone; it does not move an M6 item ahead of unfinished M1 work.

## Current execution queue — M0

Detailed tasks and acceptance criteria are in [Sprint 01](sprint-01.md).

| Backlog item | Feature IDs | Priority | State | Dependency | Outcome |
|---|---|---|---|---|---|
| M0-01 Application scaffold | ENG-001 | P0 | Ready | None | Tauri/React/TypeScript app launches on Windows 11. |
| M0-02 Architecture modules | ENG-002 | P0 | Ready | M0-01 | Rust modular-monolith structure and composition root exist. |
| M0-03 Domain primitives | ENG-005 | P0 | Ready | M0-02 | Tested IDs, enums, errors, and `RelativePath`. |
| M0-04 SQLite foundation | ENG-004, ENG-008 | P0 | Ready | M0-02, M0-03 | App-data database, migrations, and temporary fixtures work. |
| M0-05 Typed application status | ENG-003 | P0 | Ready | M0-04 | React calls a real use case through a thin typed Tauri command. |
| M0-06 Honest frontend shell | ENG-001, ENG-003 | P0 | Ready | M0-05 | Loading, healthy/no-library, and startup-error states render. |
| M0-07 Logging and diagnostics | ENG-006 | P1 | Ready | M0-04 | Safe structured local logs exist in app data. |
| M0-08 CI quality gates | ENG-007 | P0 | Ready | M0-01–M0-07 | Actual format/lint/test/build/link checks run and fail correctly. |
| M0-09 Technical spikes | Risk reduction | P0 | Ready | M0-01 | PDFium, Drive Desktop, and SQLite implementation choices are documented. |

## M1 — Library MVP backlog

| Backlog item | Feature IDs | Priority | Depends on | Outcome |
|---|---|---|---|---|
| M1-01 Configure library | LIB-001 | P0 | M0 | Select, validate, persist, and change one local root. |
| M1-02 Scanner job | LIB-002 | P0 | M1-01 | Recursive scan reports progress, cancellation, warnings, and failures. |
| M1-03 Candidate discovery | LIB-003, LIB-004 | P0 | M1-02 | PDFs and eligible image folders become deterministic candidates. |
| M1-04 Catalog reconciliation | LIB-005, LIB-006, LIB-007 | P0 | M1-03 | Repeated scans upsert, preserve user metadata, and mark missing items safely. |
| M1-05 Metadata extraction | LIB-008 | P0 | M1-04 | Core metadata is captured with per-book failure isolation. |
| M1-06 Thumbnail pipeline | LIB-009 | P0 | M1-03, M1-04 | Rebuildable covers are generated outside source folders. |
| M1-07 Library browser | LIB-010 | P0 | M1-04, M1-06 | Virtualized grid/list shows catalog and opens a selected book action. |
| M1-08 Rescan and repair | LIB-011 | P1 | M1-02–M1-07 | User can rescan and repair derived catalog/thumbnail state. |

Blocking decisions before M1 implementation:

- Windows path uniqueness and case-only rename policy;
- stable identity during rename;
- image-folder eligibility, mixed-content, and nested-chapter policy;
- thumbnail format and invalidation.

## M2 — Reading MVP backlog

| Backlog item | Feature IDs | Priority | Depends on | Outcome |
|---|---|---|---|---|
| M2-01 Reader contract | READ-001 | P0 | M1 catalog | Shared lifecycle/location model supports both book kinds. |
| M2-02 PDF reader | READ-002, READ-004, READ-006, READ-009 | P0 | M0 PDFium spike, M2-01 | PDF open/render/navigation/fit/rotation and error states work. |
| M2-03 Image reader | READ-003, READ-004, READ-005, READ-006, READ-007 | P0 | M1 image ordering, M2-01 | Single/continuous modes lazy-load ordered pages. |
| M2-04 Reader ergonomics | READ-008 | P1 | M2-02, M2-03 | Fullscreen and keyboard shortcuts work consistently. |
| M2-05 Reading state | PROG-001, PROG-002, PROG-003 | P0 | M2-01 | Progress saves with debouncing and resumes reliably. |
| M2-06 Bookmarks | BOOKMARK-001, BOOKMARK-002, BOOKMARK-003 | P1 | M2-01, M2-05 | Bookmarks persist and navigate to saved locations. |

## M3 — Knowledge MVP backlog

| Backlog item | Feature IDs | Priority | Depends on | Outcome |
|---|---|---|---|---|
| M3-01 Notes-root configuration | NOTE-001 | P0 | M0 settings | Configure and validate a notes root. |
| M3-02 Markdown file workflow | NOTE-002 | P0 | M3-01 | Create and conservatively edit portable notes. |
| M3-03 Book/location associations | NOTE-003 | P0 | M2 locations, M3-02 | Notes link to books and reading locations. |
| M3-04 Markdown projection | NOTE-004, NOTE-005 | P0 | M3-02 | Parse metadata/links into rebuildable SQLite projections. |
| M3-05 External interoperability | NOTE-006 | P1 | M3-01 | Open notes and folders in normal editors/Obsidian. |
| M3-06 Backlinks | NOTE-007 | P1 | M3-04 | Resolve and display basic backlinks. |
| M3-07 External edit reconciliation | NOTE-008 | P1 | M3-04 | External changes update projections without destructive rewrites. |

## M4 — Search MVP backlog

| Backlog item | Feature IDs | Priority | Depends on | Outcome |
|---|---|---|---|---|
| M4-01 Search schema/projection | SEARCH-001 | P0 | M1 catalog, M3 projections | Rebuildable search documents and FTS5 schema exist. |
| M4-02 Core indexing | SEARCH-002, SEARCH-003, SEARCH-004 | P0 | M4-01 | Books, notes, bookmarks, and tags are searchable offline. |
| M4-03 Global search UI | SEARCH-005 | P0 | M4-02 | Query and result-type filters navigate to source items. |
| M4-04 Incremental index queue | SEARCH-006 | P1 | M4-01, scan/note events | Changed content updates without a full rebuild. |
| M4-05 Repair and diagnostics | SEARCH-007 | P1 | M4-01–M4-04 | User can rebuild and inspect failed documents. |

## M5 — Reliability and release backlog

| Backlog item | Feature IDs | Priority | Depends on | Outcome |
|---|---|---|---|---|
| M5-01 Filesystem reconciliation | REL-001 | P0 | M1 scan policies | Debounced watcher targets affected content and falls back safely. |
| M5-02 Job recovery | REL-002 | P0 | background job framework | Interrupted work resumes or fails explicitly. |
| M5-03 Backup and rebuild | REL-003 | P0 | stable schema | Integrity, backup, and rebuild paths protect local state. |
| M5-04 Cache maintenance | REL-004 | P1 | thumbnails/indexes | Orphans and stale rebuildable artifacts are cleaned safely. |
| M5-05 Performance suite | REL-005 | P0 | M1–M4 | Measured large-library fixtures define release budgets. |
| M5-06 Windows delivery | REL-006 | P0 | all core milestones | Installer and upgrades preserve state and launch reliably. |
| M5-07 Accessibility | REL-007 | P1 | stable UI | Keyboard and accessibility pass covers core workflows. |
| M5-08 Diagnostics | REL-008 | P1 | logging/jobs | User can export a privacy-safe diagnostic report. |

## M6 — Optional intelligence backlog

M6 starts only after the M5 release gate. All items are P3 until explicitly promoted by a milestone revision.

| Backlog item | Feature IDs | Outcome |
|---|---|---|
| M6-01 OCR | OCR-001, OCR-002 | Optional page-level OCR behind a cancellable job port. |
| M6-02 Japanese dictionary | DICT-001, DICT-002, DICT-003 | Offline lookup for Japanese reading workflows. |
| M6-03 AI provider boundary | AI-001 | Optional network/provider abstraction with isolated secrets. |
| M6-04 Reading assistance | AI-002, AI-003, AI-004 | Explain, translate, summarize, and draft cards without automatic canonical writes. |
| M6-05 Anki export | ANKI-001 | Export accepted card data in an Anki-compatible form. |
| M6-06 Trusted module proof | PLUGIN-001 | Validate a minimal in-process manifest/capability model. |

## Deferred backlog

The feature catalog records deferred IDs. Current deferred themes include:

- multi-library support;
- reading statistics and goals;
- graph and semantic search;
- additional ebook/archive formats;
- hosted/web/mobile clients;
- direct Google Drive API integration;
- untrusted plugin sandbox or marketplace;
- in-place PDF annotation modification;
- multi-user collaboration.

## Backlog maintenance rules

- Create issues/tasks from the next incomplete backlog slice, preserving the catalog feature IDs.
- Add detailed acceptance criteria to the active sprint or issue; do not copy full module specifications into this file.
- Split a slice when it cannot be reviewed or delivered safely as one vertical outcome.
- Do not move work between milestones without updating the feature catalog and implementation plan in the same change.
- Do not mark a backlog slice complete while any mapped feature remains unimplemented or required checks fail.