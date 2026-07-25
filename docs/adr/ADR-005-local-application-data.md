# ADR-005: Store operational data in OS application data

- **Status:** Accepted
- **Date:** 2026-07-25

## Context

The selected library root may be a normal local folder or a folder synchronized by Google Drive Desktop. Placing SQLite, thumbnails, logs, job state, and search indexes inside that root would cause several problems:

- SQLite files and WAL files are unsafe to synchronize as ordinary files while the application is running;
- generated caches would create unnecessary sync traffic;
- different machines may require different absolute root paths and local runtime state;
- recovery becomes harder when user-owned books and application-owned artifacts are mixed;
- the application must never make hidden infrastructure folders look like part of the user's book collection.

At the same time, the application needs to remember the absolute location of the configured library root on the current machine. That value is machine-local configuration, not a portable identifier for a book.

## Decision

Store the SQLite database and application-owned runtime artifacts in the operating-system application-data directories provided by Tauri.

The initial layout is conceptually:

```text
<app-data>/
  book-library.sqlite3
  backups/
  thumbnails/
  cache/
  logs/
```

The exact platform path is resolved through Tauri APIs and must not be hard-coded.

The current absolute library root may be stored in machine-local application settings. All durable references *inside* the library remain normalized relative paths:

- `books.relative_path` is relative to the configured library root;
- image page paths are relative to the configured library root;
- note paths are relative to the configured notes root;
- thumbnail database references are relative to the application cache namespace.

An absolute root setting is therefore an explicit configuration exception to the "relative paths only" content rule. Absolute paths must never be used as book identity or copied into domain entities as persisted content references.

## Considered options

### Store everything inside the library root

Rejected for the first implementation because SQLite and generated caches may be synchronized by Google Drive Desktop, causing conflicts, unnecessary uploads, and platform-specific artifacts in the user's content folder.

### Store the database in app data and thumbnails in the library root

Rejected because generated covers are rebuildable and should not modify or clutter source folders.

### Store operational data in app data

Accepted because it isolates mutable runtime state, follows desktop platform conventions, and keeps the library folder user-owned and portable.

## Consequences

Positive consequences:

- source folders remain clean and non-destructive;
- SQLite WAL and migration activity are not synchronized as book content;
- thumbnails, indexes, and logs can be deleted or rebuilt independently;
- each machine can point to a different absolute location for the same relative library layout.

Trade-offs:

- reading state and app-local metadata do not automatically synchronize between machines;
- deleting OS application data removes local metadata unless it has been backed up or exported;
- relocating the library requires updating the machine-local root setting.

These trade-offs are acceptable for the first release. Cross-machine metadata synchronization is a separate future feature and must not be approximated by syncing a live SQLite database.

## Implementation constraints

- Resolve app-data and cache directories using Tauri path APIs.
- Enable SQLite foreign keys on every connection.
- Decide WAL configuration during Sprint 01 using Windows integration tests.
- Never write thumbnails, indexes, logs, or database files into source book folders.
- Provide a future backup/export command before claiming that application metadata is portable.
- Treat a missing configured root as a recoverable state; do not delete catalog or reading records automatically.

## Revisit when

Revisit this decision only when a designed metadata synchronization or portable-profile feature exists with explicit conflict handling, backups, and migration semantics.