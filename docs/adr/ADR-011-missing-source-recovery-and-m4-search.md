# ADR-011: Recover missing sources explicitly and use rebuildable trigram FTS5 search

- **Status:** Accepted
- **Date:** 2026-07-26

## Context

A missing catalog record still has useful history, but its former source path no
longer resolves. Disabling every action prevents the user from inspecting the
last known location or correcting a move. M4 also needs one offline search
surface across a Japanese and multilingual catalog plus portable Markdown
notes.

## Decision

For a missing book, Open folder resolves the last known path and opens the
nearest existing ancestor beneath the configured library root. A separate
Locate action accepts an explicit replacement selected by the user. The
replacement must remain inside that root and match the existing book kind. It
updates only catalog metadata; the app never moves, renames, rewrites, or
deletes the selected source.

M4 uses an application-owned SQLite FTS5 projection with the trigram tokenizer:

- books index title, relative path, and status;
- notes index canonical Markdown bodies plus projected titles and paths;
- note headings and tags are separate searchable scopes;
- filters expose books, notes, headings, and tags;
- search results retain source identifiers for navigation;
- a persisted coalescing job records index refresh work after scans, title
  edits, relinks, note creation, note saves, and note refresh;
- explicit rebuild and diagnostics remain available;
- loss of every FTS record is recoverable from catalog metadata and Markdown.

## Consequences

Short or malformed source paths cannot escape the authorized root. A relink
conflict is reported instead of replacing another catalog identity. Search
never becomes the canonical copy of books or notes. Trigram indexing increases
index size but supports substring matching for Japanese text without requiring
language-specific segmentation.

## Implementation constraints

- React sends book or note identifiers, never trusted resolved paths.
- Absolute replacement paths are accepted only at the desktop boundary and are
  validated by the application use case.
- Search query failures and per-note read failures do not break catalog or note
  workflows.
- User-facing snippets escape source HTML and allow only app-inserted match
  markers.
- macOS Intel validation remains pending until the Windows version is complete.

## Revisit when

Revisit relocation identity when content hashes support safe automatic move
detection. Revisit tokenization only with multilingual benchmarks showing a
better offline alternative.
