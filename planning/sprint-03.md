# Sprint 03 — Windows catalog launch and live search

## Status

- **State:** Windows Done — macOS Intel validation pending
- **Milestone:** M2 — External reading workflow
- **Platform order:** Windows 11 x64 implementation first; macOS Intel validation later
- **Feature IDs:** READ-001, READ-009, LIB-014

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

## Windows acceptance

- React passes only a book ID to the desktop command;
- relative catalog paths resolve beneath the configured root;
- PDF books open their parent directory;
- image books open their own directory;
- missing or invalid books produce stable safe errors;
- typing filters title, relative path, kind, and status without network access;
- Japanese titles and whitespace-separated queries match correctly;
- opening and searching never changes source books.

## Out of scope

- embedded PDF/image rendering;
- reading progress, recent-reading history, and bookmarks;
- opening files through a default reader application;
- FTS5, note search, content extraction, fuzzy ranking, or web search;
- macOS Intel validation during the Windows implementation pass.

## Windows result

- All four M2 work packages are `Windows Done`.
- The desktop command accepts only a book UUID and resolves root/path through the
  catalog repository.
- Application tests prove PDF-parent, image-folder, missing, and unknown-book
  behavior without launching a real file manager.
- Live filtering passes Unicode Japanese and multi-term AND query tests.
- Rust, Clippy, format, frontend tests, typecheck, web build, Tauri release build,
  MSI/NSIS packaging, and release launch smoke all pass.
