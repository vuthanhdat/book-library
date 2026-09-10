# ADR-018: Require every note to belong to one book

- **Status:** Accepted
- **Date:** 2026-09-10

## Context

The notes workspace previously allowed standalone Markdown notes and placed all
new files directly in one notes root. That made book notes easy to lose among
unrelated files and allowed a note to be created without a book association.
The product workflow now treats notes as reading context for a cataloged book.

ADR-004 and ADR-010 still require Markdown to remain the canonical note content
and prohibit destructive filesystem maintenance.

## Decision

Every Book Library-managed note must be associated with exactly one cataloged
book. The association is stored in portable YAML frontmatter:

```yaml
---
book_relative_path: "Shelf/Book.pdf"
---
```

New notes are created below the configured notes root in a readable
book-scoped directory derived from the book relative path, for example:

```text
<notes-root>/Shelf/Book.pdf/Reading note-<short-id>.md
```

The application rejects note creation without a book. Refresh skips notes that
do not have a valid book association or refer to a book absent from the local
catalog and reports them as refresh issues. Existing unlinked Markdown files
are never moved or deleted automatically; they remain user-owned files and can
be reintroduced after the user adds a valid book association.

SQLite keeps the rebuildable one-note-to-one-book projection in
`book_note_links`. Book records do not own the Markdown bytes, and deleting a
note does not delete its book.

## Consequences

- The Notes workspace and Book Detail workflow always show a book context.
- Notes for one book are grouped together in the filesystem while remaining
  ordinary Markdown files usable by Obsidian.
- A book relocation can invalidate the frontmatter path until the note is
  explicitly updated; the app does not rewrite notes during refresh.
- Existing standalone notes are preserved but are no longer treated as managed
  notes until linked explicitly.

## Implementation constraints

- Keep note paths relative to the configured notes root.
- Validate book-scoped directories and never follow a symlink outside the notes
  root.
- Preserve existing frontmatter when an editor save supplies only the Markdown
  body.
- Do not automatically rename, move, or delete existing Markdown files.
- Keep note text in Markdown; SQLite remains a projection and index.

## Revisit when

Revisit this decision if the product adds a first-class topic/daily-note
workflow that needs notes without a book, or if portable multi-book notes
become a required workflow.
