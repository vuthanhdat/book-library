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

Normal scans keep the short 250-millisecond PDF identity probe. ADR-013 removes
filesystem scanning from Repair; Repair now selects missing covers directly
from the existing catalog.

Cover repair and explicit retry use last-known-good replacement semantics. The
existing catalog reference and cover file remain untouched while a new cover is
rendered to a new file. The catalog switches references only after success.
Bulk Repair target selection is refined by ADR-013. It gives each selected
target the same thirty-second hydration window and does not regenerate a usable
current cover.

Generated cover filenames contain only the book identifier and a new generation
identifier. Source fingerprints remain metadata and are never embedded in cache
paths because an unavailable fingerprint may contain a long relative source
path. Explicit retry distinguishes a real 30-second timeout from an immediate
render or cache-write failure in its user-safe error response.

PDFium rendering remains serialized, but a panic in one isolated thumbnail
worker does not permanently disable the process-wide render lock. A later
attempt recovers the lock and creates fresh document state. The PDFium binding
itself is initialized once and retained for the application process because the
library rejects a second binding initialization.

Explicit Force cover emits typed stages for opening/hydrating the source,
rendering the first page, saving the app-data cover, and completion. Book Detail
shows the accumulated stages and appends the stable user-safe failure message
when an attempt stops.

Before retrying source rendering, Repair checks app-data thumbnail history for
a decodable cover whose catalog link was lost. It restores that link and marks
the cover ready without reading the source again. Invalid or missing cached
files are skipped and remain eligible for normal repair.

## Consequences

Large rescans remain bounded while the user can deliberately wait longer for a
cloud-backed book. Reading status is book-level workflow metadata, not reading
progress or an embedded-reader location.

## Implementation constraints

- no source book is rewritten;
- forcing a cover is unavailable for missing, unsupported, or error sources;
- a successful forced render promotes a cloud-unavailable source to available;
- a failed repair or forced render retains the previous cover;
- cache filenames remain flat and bounded even when source paths or titles are
  long;
- one failed worker cannot make every later Force or Repair attempt fail
  immediately for the remainder of the app process;
- the user can distinguish a source-open failure from rendering, cache writing,
  completion, or timeout without exposing an absolute source path;
- Repair reports recovered, generated, and failed cover counts separately from
  scan issues;
- Repair does not run a PDF identity probe or filesystem discovery scan;
- tags are bounded, normalized, and user editable;
- Markdown notes remain portable and rebuildable;
- macOS Intel validation follows the completed Windows version.

## Revisit when

Revisit when the app has a persistent hydration job API, configurable cloud
timeouts, or a cross-application reading-progress integration.
