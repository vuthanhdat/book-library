# SQLite Foundation

## Status

- **Sprint:** Sprint 01
- **Implementation:** in progress
- **Validated:** Windows 11 x64 development build
- **Pending:** real macOS Intel x64 initialization and concurrency validation

This document records implementation and recovery details for the SQLite foundation.
ADR-001 owns the database choice and ADR-005 owns its location.

## Current implementation

- Library: `rusqlite` with bundled SQLite, avoiding a machine-installed SQLite dependency.
- Connection ownership: one process-local connection protected by a mutex for the initial
  status flow. A pool is deferred until concurrent jobs require measured parallel access.
- Database file: `book-library.sqlite3` under the Tauri-resolved application-data directory.
- Connection setup: foreign keys enabled and a five-second busy timeout.
- Journal mode: SQLite default rollback journal. WAL is not enabled until Windows and
  macOS Intel concurrency behavior has been validated.
- Transactions: every migration batch runs in a transaction.
- Migration organization: ordered, forward-only Rust migration entries recorded in
  `schema_migrations`.

The first migration creates:

- `schema_migrations` for applied version and name history;
- `application_settings` for local settings;
- `configured_libraries` for the machine-local configured root.

No source book, note body, cache, thumbnail, or index is stored by this migration.

The second migration adds the M1 catalog projection:

- `scan_jobs` and `scan_issues`;
- `books` with path uniqueness, derived/user title provenance, availability,
  fingerprints, metadata, and thumbnail state;
- `image_pages` with deterministic page order;
- `thumbnails` with app-cache-relative references.

Source books and image bytes remain outside SQLite. The only absolute source
path is the machine-local configured root; persisted book and page identities
are normalized relative paths.

## Recovery behavior

Database initialization returns typed errors for application-data creation, opening,
connection configuration, and migration failures. A failed migration transaction is
rolled back. The application must not delete or recreate the database automatically.

During Sprint 01 development:

1. preserve the failed database for diagnosis;
2. inspect the stable startup error and local logs after logging is implemented;
3. restore from a known backup when backup support exists;
4. never treat database recovery as authorization to modify source books or Markdown notes.

## Validation evidence

Automated temporary-directory tests verify:

- first initialization creates the database outside a separate library fixture;
- reopening applies each migration exactly once;
- foreign keys are enabled;
- the initial status is healthy with no configured library;
- incompatible migration state returns a typed migration error.
- repeated catalog reconciliation creates no duplicate path record, preserves a
  user title, and marks unseen records missing;
- cloud-only candidates remain cataloged as unavailable without thumbnail work;
- thumbnail cache references resolve beneath app data.

The Windows development executable was launched and created:

```text
%APPDATA%/dev.booklibrary.desktop/book-library.sqlite3
```

Four synchronized writer threads were also exercised against the process-owned
connection. All writes completed, the expected row count was preserved, and the
journal remained `DELETE`. This supports the initial single-connection model on
Windows. WAL remains deferred: SQLite documents its higher read/write concurrency,
but it also introduces shared-memory and checkpoint behavior that is unnecessary
for the current status flow. See [SQLite WAL](https://www.sqlite.org/wal.html).

The absolute expanded user path is intentionally not recorded. macOS Intel app-data,
reopen, and concurrency evidence remain required before M0-04 is `Done`.
