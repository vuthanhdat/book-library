# ADR-010: Define M3 Markdown note ownership and association policy

- **Status:** Accepted
- **Date:** 2026-07-26

## Context

ADR-004 makes Markdown the canonical note body but leaves M3 choices open:
notes-root location, internal versus external editing, book-link portability,
frontmatter behavior, and external-change reconciliation.

## Decision

The notes root is one machine-local user-selected directory. It may be inside or
outside the library root. Note identities persisted in SQLite are normalized
paths relative to that configured notes root.

M3 provides a conservative plain-Markdown editor and external opening:

- app-created notes are UTF-8 `.md` files;
- explicit Save atomically replaces only the selected note file;
- projection refresh never rewrites a Markdown file;
- unknown Markdown and frontmatter text is preserved by the editor;
- external editors and Obsidian remain peer owners of the same files.

App-created book notes use readable YAML frontmatter:

```yaml
---
book_relative_path: "Shelf/Book.pdf"
---
```

`book_relative_path` is portable catalog identity. App-specific book UUIDs are
not written into Markdown. Notes without frontmatter remain valid general notes.

Projection refresh recursively discovers `.md` files without following
symlinks. It parses:

- first level-one heading as title, then filename as fallback;
- headings;
- hashtags;
- `[[wiki links]]`;
- relative Markdown links to `.md` files;
- `book_relative_path` frontmatter.

Refresh upserts changed notes and marks unseen notes missing; it never deletes
Markdown. Basic backlinks resolve against relative path, file stem, or projected
title. M3 uses explicit refresh after external edits. Automatic watcher
reconciliation remains M5.

## Consequences

- SQLite note records, links, headings, tags, and book relationships are
  rebuildable projections.
- changing a book path can break a human-readable association until the
  frontmatter is updated explicitly.
- users can edit notes in Book Library, Obsidian, or another editor.
- M4 can build FTS5 from the canonical Markdown and these projections.

## Implementation constraints

- Never store the only copy of a note body in SQLite.
- Never write hidden application files into the notes root.
- Validate every resolved note path beneath the authorized notes root.
- Use atomic create/replace for explicit app saves.
- Do not delete, rename, or move notes during refresh or repair.
- Do not log note bodies.

## Revisit when

Revisit for automatic watcher behavior, portable book-relocation links, richer
frontmatter editing, or multi-root notes.
