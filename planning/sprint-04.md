# Sprint 04 — Windows Markdown knowledge workflow

## Status

- **State:** Completed
- **Milestone:** M3 — Knowledge MVP
- **Platforms validated:** Windows 11 x64 and macOS Intel x64
- **Feature IDs:** NOTE-001 through NOTE-008

## Goal

Create, edit, associate, refresh, and open portable Markdown notes while keeping
the filesystem canonical and SQLite projections rebuildable.

## Work packages

1. Configure and validate one notes root.
2. Create and atomically save UTF-8 Markdown files.
3. Associate app-created notes with books through portable frontmatter.
4. Project titles, headings, tags, links, and book relationships into SQLite.
5. List/read/edit notes in a minimal Markdown workspace.
6. Open a note or notes root externally.
7. Resolve and display basic backlinks.
8. Refresh externally edited notes without rewriting or deleting them.

## Windows acceptance

- note bodies exist as `.md` files beneath the selected notes root;
- SQLite never becomes the only copy of user text;
- all persisted note paths are relative and Unicode preserving;
- explicit Save uses an atomic replacement in the same directory;
- refresh does not rewrite, rename, move, or delete Markdown;
- book links use `book_relative_path`, not an app UUID;
- wiki and relative Markdown links produce backlinks;
- an unreadable or malformed note does not block other notes;
- notes and the notes root open through normal external applications;
- M4 can consume the resulting projection without changing canonical ownership.

## Verification evidence

- Rust unit and integration suite covers atomic Markdown saves, parsing, SQLite
  reconciliation, and backlink resolution.
- Frontend tests cover the explicit Save/Refresh workspace and external-open
  affordances.
- Windows x64 production build, hidden launch smoke check, MSI packaging, and
  NSIS setup packaging passed on 2026-07-26.
- The maintainer confirmed the required macOS Intel notes-root, Markdown
  create/edit/refresh, external-open, and backlink validation; NOTE-001 through
  NOTE-008 are `Completed`.

## Out of scope

- rich-text or rendered Markdown editing;
- automatic filesystem watchers;
- page-level reading locations;
- graph visualization;
- FTS5/global search implementation;
- macOS Intel validation during the Windows implementation pass.
