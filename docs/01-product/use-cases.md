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
| Rescan or repair catalog | `LIB-011` | Refresh changed content and rebuild derived catalog or thumbnail state safely. | [`03-library/`](../03-library/) |
| Open PDF | `READ-001`, `READ-002`, `READ-004`, `READ-006`–`READ-009` | Open and navigate a PDF through the reader adapter. | [`04-reader/pdf-reader.md`](../04-reader/pdf-reader.md) |
| Open image-folder book | `READ-001`, `READ-003`–`READ-009` | Read naturally ordered page images without converting or rewriting them. | [`04-reader/image-reader.md`](../04-reader/image-reader.md) |
| Resume reading | `PROG-001`–`PROG-003` | Save the latest reading location and restore it after closing or restarting the app. | [`04-reader/reading-history.md`](../04-reader/reading-history.md) |
| Manage bookmarks | `BOOKMARK-001`–`BOOKMARK-003` | Save, edit, remove, list, and return to reader-independent locations. | [`04-reader/bookmark.md`](../04-reader/bookmark.md) |
| Configure notes | `NOTE-001` | Select and validate the root that owns canonical Markdown notes. | [`05-notes/notes.md`](../05-notes/notes.md) |
| Create, edit, and associate notes | `NOTE-002`–`NOTE-005` | Maintain portable Markdown linked to books or reading locations with rebuildable projections. | [`05-notes/notes.md`](../05-notes/notes.md) |
| Use notes externally | `NOTE-006` | Open notes or their folder in Obsidian or a normal editor without conversion. | [`05-notes/obsidian.md`](../05-notes/obsidian.md) |
| Navigate note relationships | `NOTE-007`, `NOTE-008` | View backlinks and reconcile changes made by external editors. | [`05-notes/`](../05-notes/) |
| Search local knowledge | `SEARCH-001`–`SEARCH-007` | Search rebuildable projections of books, notes, bookmarks, tags, and supported extracted text. | [`06-search/`](../06-search/) |
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