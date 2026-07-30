# Sprint 03 — Windows catalog launch and live search

## Status

- **State:** Completed
- **Milestone:** M2 — External reading workflow
- **Platforms validated:** Windows 11 x64 and macOS Intel x64
- **Feature IDs:** READ-001, READ-009, LIB-014, LIB-015

## Goal

Find a cataloged book instantly and open its authorized source location in the
operating system file manager without embedding a reader or modifying source
content.

## Work packages

1. Resolve an available book ID to an authorized source directory.
2. Open PDF parent directories and image-book directories through a platform
   adapter.
3. Add explicit open-folder actions and honest unavailable/missing errors.
4. Add live, Unicode-aware, multi-term catalog filtering.
5. Validate the Windows release and preserve macOS-compatible boundaries.
6. Edit one app-local display title without rescanning or changing source files.

## Windows acceptance

- React passes only a book ID to the desktop command;
- relative catalog paths resolve beneath the configured root;
- PDF books open their parent directory;
- image books open their own directory;
- missing or invalid books produce stable safe errors;
- typing filters title, relative path, kind, and status without network access;
- Japanese titles and whitespace-separated queries match correctly;
- opening and searching never changes source books.
- title edits validate Unicode text, update search immediately, and survive
  later rescans.

## Out of scope

- embedded PDF/image rendering;
- reading progress, recent-reading history, and bookmarks;
- opening files through a default reader application;
- FTS5, note search, content extraction, fuzzy ranking, or web search;
- macOS Intel validation during the Windows implementation pass.

## Cross-platform result

- All M2 work packages are `Done`.
- The desktop command accepts only a book UUID and resolves root/path through the
  catalog repository.
- Application tests prove PDF-parent, image-folder, missing, and unknown-book
  behavior without launching a real file manager.
- Live filtering passes Unicode Japanese and multi-term AND query tests.
- Per-book Edit updates only the SQLite display title, marks its provenance as
  `user`, refreshes search immediately, and survives reconciliation.
- Rust, Clippy, format, frontend tests, typecheck, web build, Tauri release build,
  MSI/NSIS packaging, and release launch smoke all pass.
- The maintainer confirmed the required macOS Intel Finder, catalog-search, title
  editing, and user-safe error validation.
