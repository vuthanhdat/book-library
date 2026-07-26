# ADR-012: Keep reading status local and retry one cover explicitly

- **Status:** Accepted
- **Date:** 2026-07-26

## Context

The catalog needs a focused place for user-managed reading state, book tags, and
linked notes. The normal one-second thumbnail timeout protects a large scan
from stalling, but it is too short when Google Drive must hydrate one selected
source.

## Decision

Book Detail stores `unread`, `reading`, or `read` plus user book tags in SQLite.
These fields survive rescans and become catalog search input. Notes shown on the
page remain canonical Markdown files linked through the existing note model;
the detail page does not introduce a database-only note body.

Normal scans keep the one-second per-cover timeout. An explicit Force cover
action retries only the selected available or cloud-unavailable book, opens its
source as part of rendering, waits up to thirty seconds for hydration/rendering,
and writes the result only to application data.

Cover repair and explicit retry use last-known-good replacement semantics. The
existing catalog reference and cover file remain untouched while a new cover is
rendered to a new file. The catalog switches references only after success.
Bulk Repair retries only covers that are missing or already in an error state
and gives each target the same thirty-second hydration window. It does not
regenerate healthy covers.

## Consequences

Large rescans remain bounded while the user can deliberately wait longer for a
cloud-backed book. Reading status is book-level workflow metadata, not reading
progress or an embedded-reader location.

## Implementation constraints

- no source book is rewritten;
- forcing a cover is unavailable for missing, unsupported, or error sources;
- a successful forced render promotes a cloud-unavailable source to available;
- a failed repair or forced render retains the previous cover;
- Repair reports generated and failed cover counts separately from scan issues;
- tags are bounded, normalized, and user editable;
- Markdown notes remain portable and rebuildable;
- macOS Intel validation follows the completed Windows version.

## Revisit when

Revisit when the app has a persistent hydration job API, configurable cloud
timeouts, or a cross-application reading-progress integration.
