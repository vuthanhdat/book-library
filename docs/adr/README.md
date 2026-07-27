# Architecture Decision Records

ADRs capture decisions that materially constrain implementation. Accepted ADRs override alternatives and open questions in older design documents.

## Status values

- **Proposed:** under discussion; not binding.
- **Accepted:** binding for implementation.
- **Superseded:** replaced by a newer ADR.
- **Deprecated:** retained for history but should not be used.

## Decision index

| ADR | Status | Decision |
|---|---|---|
| [ADR-001](ADR-001-use-sqlite.md) | Accepted | Use SQLite for local metadata, operational state, jobs, and FTS indexes. |
| [ADR-002](ADR-002-google-drive-desktop.md) | Accepted | Treat Google Drive Desktop as an external local-filesystem sync layer; do not integrate the Drive API. |
| [ADR-003](ADR-003-relative-path.md) | Accepted | Persist source-content references as normalized relative paths. |
| [ADR-004](ADR-004-markdown-notes.md) | Accepted | Keep user-authored note text in Markdown; SQLite contains projections only. |
| [ADR-005](ADR-005-local-application-data.md) | Accepted | Store the database and rebuildable application artifacts in OS application data, outside the library root. |
| [ADR-006](ADR-006-rust-modular-monolith.md) | Accepted | Keep domain and application behavior in Rust using a modular-monolith structure for the first implementation. |
| [ADR-007](ADR-007-supported-desktop-platforms.md) | Accepted | Require one shared codebase supporting Windows 11 x64 and macOS Intel x64 from M0. |
| [ADR-008](ADR-008-m1-library-policies.md) | Accepted | Define M1 path identity, case, symlink, image-folder, and thumbnail policies. |
| [ADR-009](ADR-009-external-reading-and-live-catalog-search.md) | Accepted | Open source locations in the OS file manager and search the catalog live instead of embedding a reader. |
| [ADR-010](ADR-010-m3-markdown-notes-policy.md) | Accepted | Keep M3 notes portable with a configurable root, conservative editor, relative book links, rebuildable projections, and explicit refresh. |
| [ADR-011](ADR-011-missing-source-recovery-and-m4-search.md) | Accepted | Recover missing sources only through authorized explicit relinking and use rebuildable trigram FTS5 for M4. |
| [ADR-012](ADR-012-book-detail-and-explicit-cover-retry.md) | Accepted | Add app-local book workflow metadata and retry one cloud-backed cover explicitly without slowing normal scans. |
| [ADR-013](ADR-013-bulk-cover-repair-targets.md) | Accepted | Treat Repair as a batch Force retry for books that have no usable cover. |

## ADR format

Every new ADR should include:

1. status and date;
2. context;
3. decision;
4. considered options;
5. consequences;
6. implementation constraints;
7. conditions that would justify revisiting the decision.

Create a new ADR instead of silently rewriting the outcome of an accepted ADR. Mark the old ADR as superseded and link both records.
