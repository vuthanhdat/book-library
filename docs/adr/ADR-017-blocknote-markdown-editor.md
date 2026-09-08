# ADR-017: Use BlockNote as the in-app Markdown note editor

- **Status:** Accepted
- **Date:** 2026-09-08

## Context

The notes workflow needs a capable in-app editor, but ADR-004 and ADR-010
require Markdown files to remain the canonical, portable store. BlockNote is a
browser-first React block editor and its built-in Markdown conversion supports
the common CommonMark/GFM subset, but the conversion is intentionally lossy.

The application also uses YAML frontmatter for book associations and allows
notes to be edited by Obsidian or another external editor.

## Decision

Use BlockNote with its Mantine React view as the official in-app note editor.
Keep the existing application `draft` state and `save_note` use case as the
only save path. Convert the note body from Markdown to BlockNote blocks when a
note opens, and convert blocks back to Markdown when the user edits and
explicitly saves.

YAML frontmatter remains opaque to BlockNote: the editor receives only the
Markdown body, and the original frontmatter block is reattached before the
save. This preserves `book_relative_path` and unknown frontmatter keys without
moving metadata into the editor or SQLite.

## Considered options

### Keep the textarea editor

Rejected because it provides portability but not the requested block editing,
formatting toolbar, slash commands, or drag-and-drop block workflow.

### Store BlockNote JSON in SQLite or note files

Rejected because it would make a proprietary representation canonical and would
break the Markdown/Obsidian portability invariant.

### Store BlockNote JSON alongside Markdown as hidden state

Rejected because ADR-004 forbids hidden binary/application state in note files
and the projection must remain rebuildable from Markdown.

## Consequences

- Notes remain ordinary `.md` files and external editing remains supported.
- Rich editing is available offline in the React presentation layer.
- Explicit saves may normalize body formatting to BlockNote's supported
  CommonMark/GFM subset; the UI and documentation must make this limitation
  clear.
- Frontmatter is preserved separately from the editor round-trip.
- No Rust domain, application port, SQLite schema, or source-book workflow
  changes are needed.

## Implementation constraints

- Do not auto-save while typing; use the existing explicit Save action.
- Do not pass absolute paths, source-book contents, or note bodies to an
  external provider.
- Keep the editor behind a React component so the desktop boundary and Rust
  layers remain unaware of BlockNote.
- Retain a non-browser rendering fallback for static tests and diagnostics.
- Use the same editor theme as the application light/dark setting.

## Revisiting conditions

Revisit this decision if BlockNote's Markdown conversion can no longer support
the note syntax required by the product, if lossless Markdown editing becomes
a release requirement, or if a future accepted ADR chooses a different
portable editor boundary.
