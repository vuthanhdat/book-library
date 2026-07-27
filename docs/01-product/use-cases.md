# Core Use Cases

## Status

This document is the single product-level map of user actions. It identifies the outcome and ownership of each workflow without duplicating detailed behavior from module specifications.

Feature IDs, status, and milestone assignment are authoritative in the [feature catalog](feature-catalog.md). Technical decisions are authoritative in [ADRs](../adr/README.md).

## Use-case map

| Use case | Feature IDs | User outcome | Canonical specification |
|---|---|---|---|
| Configure library | `LIB-001` | Select one local root and initialize application-owned metadata safely. | [`03-library/initialization.md`](../03-library/initialization.md) |
| Scan and reconcile library | `LIB-002`–`LIB-009` | Discover PDFs and image-folder books, reconcile changes, extract metadata, and generate rebuildable thumbnails without modifying source files. | [`03-library/`](../03-library/) |
| Browse catalog | `LIB-010` | View discovered books, status, cover, type, and folder context. | Feature catalog and active sprint/backlog |
| Rescan catalog | `LIB-011` | Refresh changed content and rebuild derived catalog state safely. | [`03-library/`](../03-library/) |
| Repair missing covers | `LIB-011` | Retry catalog books without a usable cover, one at a time, without rescanning source folders. | [`03-library/thumbnail.md`](../03-library/thumbnail.md) |
| Open book source location | `READ-001`, `READ-009` | Open a PDF's containing directory or an image book's directory in the OS file manager. | [ADR-009](../adr/ADR-009-external-reading-and-live-catalog-search.md) |
| Filter catalog live | `LIB-014` | Narrow visible books immediately by title, relative path, kind, or status. | [ADR-009](../adr/ADR-009-external-reading-and-live-catalog-search.md) |
| Edit book display title | `LIB-015` | Correct one catalog title immediately without rescanning or modifying its source path. | [`03-library/discovery.md`](../03-library/discovery.md) |
| Recover a missing source | `LIB-016`, `LIB-017` | Open the nearest safe parent or explicitly relink a matching source inside the configured library without modifying files. | [`ADR-011`](../adr/ADR-011-missing-source-recovery-and-m4-search.md) |
| Manage book details | `LIB-018` | Set book-level reading status and tags, navigate linked Markdown notes, and explicitly retry a cloud-backed cover. | [`ADR-012`](../adr/ADR-012-book-detail-and-explicit-cover-retry.md) |
| Configure notes | `NOTE-001` | Select and validate the root that owns canonical Markdown notes. | [`05-notes/notes.md`](../05-notes/notes.md) |
| Create, edit, and associate notes | `NOTE-002`–`NOTE-005` | Maintain portable Markdown linked to books or reading locations with rebuildable projections. | [`05-notes/notes.md`](../05-notes/notes.md) |
| Use notes externally | `NOTE-006` | Open notes or their folder in Obsidian or a normal editor without conversion. | [`05-notes/obsidian.md`](../05-notes/obsidian.md) |
| Navigate note relationships | `NOTE-007`, `NOTE-008` | View backlinks and reconcile changes made by external editors. | [`05-notes/`](../05-notes/) |
| Search local knowledge | `SEARCH-001`–`SEARCH-007` | Search rebuildable projections of books, notes, tags, and supported extracted text. | [`06-search/`](../06-search/) |
| Run optional intelligence | `OCR-*`, `DICT-*`, `AI-*`, `ANKI-*`, `PLUGIN-*` | Use explicitly enabled optional capabilities without making them core dependencies. | [`07-ai/`](../07-ai/) |

## Shared workflow rules

Every use case must follow these rules:

- daily core behavior works offline;
- source PDFs and image folders are never renamed, moved, deleted, or rewritten;
- user-authored note text remains in Markdown;
- durable source references use normalized relative paths;
- SQLite, caches, logs, and job state remain in OS application data;
- long-running work reports progress, supports cancellation, and leaves recoverable state;
- one failed book, note, or derived job does not abort unrelated work;
- optional providers are disabled by default and do not own canonical data.

## Common lifecycle states

Use consistent states rather than inventing screen-specific alternatives:

- library: `unconfigured`, `scanning`, `ready`, `degraded`, `error`;
- book: `available`, `missing`, `unsupported`, `error`, `ignored`;
- background job: `queued`, `running`, `completed`, `failed`, `cancelled`;
- optional module: `disabled`, `needs_configuration`, `ready`, `error`.

Exact persisted shapes are defined during the owning milestone and must remain consistent with the domain model and migrations.

## Boundary rule

This file does not define layouts, database tables, event names, performance numbers, or implementation classes. Those details belong in active UI work, migrations, application contracts, measured performance requirements, and module specifications. Avoid creating separate journey or flow documents that repeat this map.
