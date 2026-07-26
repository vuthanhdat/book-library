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

## Engineering foundation — M0

Detailed tasks and acceptance criteria are in [Sprint 01](sprint-01.md).

Backlog `Done` means the item outcome and its item-level checks pass in the current
branch. The feature catalog remains the authority for feature status and does not
move to `Completed` until implementation is merged and all required platform gates pass.
The current execution order completes the Windows 11 x64 implementation first, then
runs the required macOS Intel x64 validation before closing cross-platform items.

| Backlog item | Feature IDs | Priority | State | Dependency | Outcome |
|---|---|---|---|---|---|
| M0-01 Cross-platform application scaffold | ENG-001 | P0 | In Progress | None | The same Tauri/React/TypeScript app launches on Windows 11 x64 and macOS Intel x64. |
| M0-02 Architecture modules | ENG-002 | P0 | Done | M0-01 | Rust modular-monolith structure and composition root exist without platform-specific domain/application forks. |
| M0-03 Domain primitives | ENG-005 | P0 | Done | M0-02 | Tested IDs, enums, errors, and cross-platform `RelativePath`. |
| M0-04 SQLite foundation | ENG-004, ENG-008 | P0 | In Progress | M0-02, M0-03 | OS app-data database, migrations, and temporary fixtures work on both supported platforms. |
| M0-05 Typed application status | ENG-003 | P0 | Done | M0-04 | React calls a real use case through a thin typed Tauri command. |
| M0-06 Honest frontend shell | ENG-001, ENG-003 | P0 | In Progress | M0-05 | Loading, healthy/no-library, and startup-error states render consistently on both platforms. |
| M0-07 Logging and diagnostics | ENG-006 | P1 | In Progress | M0-04 | Safe structured local logs exist in each OS application-data location. |
| M0-08 Cross-platform CI quality gates | ENG-007 | P0 | In Progress | M0-01–M0-07 | Actual format/lint/test/build/link checks run and fail correctly for compatible Windows and macOS environments. |
| M0-09 Cross-platform technical spikes | Risk reduction | P0 | In Progress | M0-01 | PDFium, Drive Desktop, and SQLite findings are documented for Windows x64 and macOS Intel x64. |

### Current Windows evidence

| Item | Windows evidence | Remaining before `Done` |
|---|---|---|
| M0-01 | Shared Tauri executable builds and launches. | Real macOS Intel launch/build. |
| M0-04 | App-data database creation, reopen, migrations, foreign keys, rollback-journal concurrency, and typed failure tests pass. | Equivalent macOS Intel validation. |
| M0-06 | Loading, healthy/no-library, startup-error, unsupported-platform, and React error-boundary states are implemented; frontend tests and Windows smoke pass. | Real macOS Intel UI smoke. |
| M0-07 | Daily structured log file and safe startup event verified under Windows app-data. | Equivalent macOS Intel verification. |
| M0-08 | Every declared gate passes locally on Windows; workflow targets Windows x64 and `macos-15-intel`. | Run the hosted workflow after publication. |
| M0-09 | Windows PDFium native render and SQLite spike pass; Drive root observation is read-only. | Authorized disposable Drive watcher test and all macOS Intel spike evidence. |

## Current execution queue — M1 Library MVP

The Windows implementation pass is complete. `Windows Done` records that the
item-level implementation and Windows checks pass in this branch; the mapped
feature remains `In Progress` until the required macOS Intel validation and merge.

| Backlog item | Feature IDs | Priority | State | Depends on | Outcome |
|---|---|---|---|---|---|
| M1-01 Configure library | LIB-001 | P0 | Windows Done | M0 | Select, validate, persist, and change one local root on either supported platform. |
| M1-02 Scanner job | LIB-002 | P0 | Windows Done | M1-01 | Recursive scan reports progress, cancellation, warnings, and failures. |
| M1-03 Candidate discovery | LIB-003, LIB-004 | P0 | Windows Done | M1-02 | PDFs and eligible image folders become deterministic candidates. |
| M1-04 Catalog reconciliation | LIB-005, LIB-006, LIB-007 | P0 | Windows Done | M1-03 | Repeated scans upsert, preserve user metadata, and mark missing items safely. |
| M1-05 Metadata extraction | LIB-008 | P0 | Windows Done | M1-04 | Core metadata is captured with per-book failure isolation. |
| M1-06 Thumbnail pipeline | LIB-009 | P0 | Windows Done | M1-03, M1-04 | Rebuildable covers are generated outside source folders. |
| M1-07 Library browser | LIB-010 | P0 | Windows Done | M1-04, M1-06 | Virtualized grid/list shows real catalog records and availability states. |
| M1-08 Rescan and repair | LIB-011 | P1 | Windows Done | M1-02–M1-07 | User can rescan and repair derived catalog/thumbnail state. |

### M1 Windows evidence

| Area | Evidence | Remaining |
|---|---|---|
| Configuration and boundary | Native folder picker, validated machine-local root, typed Tauri commands/events, and user-safe errors. | macOS Intel smoke. |
| Discovery and safety | Unit fixtures cover PDF signatures, natural image ordering, cancellation, relative paths, symlink skipping, and cloud-only status. | macOS Intel filesystem behavior. |
| Catalog | SQLite tests cover idempotent upsert, user-title preservation, missing state, unavailable state, and thumbnail scheduling. | macOS Intel database smoke. |
| Real library | Read-only two-pass scan of `H:/My Drive/07_NEW_KINDLE`: 1,019 catalog records on pass one, zero additions on pass two, and unchanged 195-entry root inventory. | Equivalent macOS Intel mixed-library smoke. |
| Thumbnails | Bounded PNG fixture is generated in app-data; PDFium access is serialized; failures remain browsable and repairable. | macOS Intel PDFium binary/render smoke. |
| Browser and repair | Tested grid/list UI uses real catalog data and exposes initialize, cancel, rescan, and repair actions. | macOS Intel UI smoke. |

M1 policy decisions are accepted in
[ADR-008](../docs/adr/ADR-008-m1-library-policies.md).

## Current execution queue — M2 External reading workflow

Detailed tasks and acceptance criteria are in [Sprint 03](sprint-03.md).

| Backlog item | Feature IDs | Priority | State | Depends on | Outcome |
|---|---|---|---|---|---|
| M2-01 Source-location use case | READ-001, READ-009 | P0 | Windows Done | M1 catalog | Resolve a book ID safely and open an authorized source directory. |
| M2-02 Platform file-manager adapter | READ-001 | P0 | Windows Done | M2-01 | Windows Explorer opens the correct directory; macOS Finder shares the same port. |
| M2-03 Catalog open interaction | READ-001, READ-009 | P0 | Windows Done | M2-01, M2-02 | Grid/list actions open folders and show honest typed errors. |
| M2-04 Realtime catalog search | LIB-014 | P0 | Windows Done | M1 browser | Unicode multi-term filtering updates while typing without network or source I/O. |

## M3 — Knowledge MVP backlog

| Backlog item | Feature IDs | Priority | Depends on | Outcome |
|---|---|---|---|---|
| M3-01 Notes-root configuration | NOTE-001 | P0 | M0 settings | Configure and validate a notes root. |
| M3-02 Markdown file workflow | NOTE-002 | P0 | M3-01 | Create and conservatively edit portable notes. |
| M3-03 Book associations | NOTE-003 | P0 | M1 catalog, M3-02 | Notes link to cataloged books; page-level locations remain deferred. |
| M3-04 Markdown projection | NOTE-004, NOTE-005 | P0 | M3-02 | Parse metadata/links into rebuildable SQLite projections. |
| M3-05 External interoperability | NOTE-006 | P1 | M3-01 | Open notes and folders in normal editors/Obsidian on both platforms. |
| M3-06 Backlinks | NOTE-007 | P1 | M3-04 | Resolve and display basic backlinks. |
| M3-07 External edit reconciliation | NOTE-008 | P1 | M3-04 | External changes update projections without destructive rewrites. |

## M4 — Search MVP backlog

| Backlog item | Feature IDs | Priority | Depends on | Outcome |
|---|---|---|---|---|
| M4-01 Search schema/projection | SEARCH-001 | P0 | M1 catalog, M3 projections | Rebuildable search documents and FTS5 schema exist. |
| M4-02 Core indexing | SEARCH-002, SEARCH-003, SEARCH-004 | P0 | M4-01 | Books, notes, and tags are searchable offline. |
| M4-03 Global search UI | SEARCH-005 | P0 | M4-02 | Query and result-type filters navigate to source items. |
| M4-04 Incremental index queue | SEARCH-006 | P1 | M4-01, scan/note events | Changed content updates without a full rebuild. |
| M4-05 Repair and diagnostics | SEARCH-007 | P1 | M4-01–M4-04 | User can rebuild and inspect failed documents. |

## M5 — Reliability and release backlog

| Backlog item | Feature IDs | Priority | Depends on | Outcome |
|---|---|---|---|---|
| M5-01 Filesystem reconciliation | REL-001 | P0 | M1 scan policies | Debounced watcher targets affected content and falls back safely on both operating systems. |
| M5-02 Job recovery | REL-002 | P0 | background job framework | Interrupted work resumes or fails explicitly. |
| M5-03 Backup and rebuild | REL-003 | P0 | stable schema | Integrity, backup, and rebuild paths protect local state. |
| M5-04 Cache maintenance | REL-004 | P1 | thumbnails/indexes | Orphans and stale rebuildable artifacts are cleaned safely. |
| M5-05 Performance suite | REL-005 | P0 | M1–M4 | Measured large-library fixtures define per-platform release budgets. |
| M5-06 Windows and macOS Intel delivery | REL-006 | P0 | all core milestones | Windows installer and macOS Intel app/DMG packaging, signing, upgrades, and clean-machine launch preserve state. |
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
- Apple Silicon and universal macOS binaries;
- direct Google Drive API integration;
- untrusted plugin sandbox or marketplace;
- in-place PDF annotation modification;
- multi-user collaboration.

## Backlog maintenance rules

- Create issues/tasks from the next incomplete backlog slice, preserving the catalog feature IDs.
- Add detailed acceptance criteria to the active sprint or issue; do not copy full module specifications into this file.
- Split a slice when it cannot be reviewed or delivered safely as one vertical outcome.
- Do not move work between milestones without updating the feature catalog and implementation plan in the same change.
- Do not mark a backlog slice complete while any mapped feature remains unimplemented or required platform checks fail.
