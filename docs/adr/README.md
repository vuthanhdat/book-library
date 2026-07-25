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
| [ADR-003](ADR-003-relative-paths.md) | Accepted | Persist source-content references as normalized relative paths. |
| [ADR-004](ADR-004-markdown-notes.md) | Accepted | Keep user-authored note text in Markdown; SQLite contains projections only. |
| [ADR-005](ADR-005-local-application-data.md) | Accepted | Store the database and rebuildable application artifacts in OS application data, outside the library root. |
| [ADR-006](ADR-006-rust-modular-monolith.md) | Accepted | Keep domain and application behavior in Rust using a modular-monolith structure for the first implementation. |

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