# Product Backlog

## Priority model

- P0: required to make the current milestone usable.
- P1: important quality or workflow capability.
- P2: valuable enhancement after the core is stable.
- P3: exploratory or optional intelligence.

## Epic A — Engineering foundation

### A1. Desktop application scaffold — P0

Acceptance criteria:

- Tauri 2 app launches on Windows 11.
- React and TypeScript frontend renders the main shell.
- Tailwind and Shadcn UI are configured.
- Frontend can call a typed Rust command.

### A2. Architecture boundaries — P0

Acceptance criteria:

- Domain code has no dependency on Tauri, SQLite, filesystem, PDFium, or React.
- Tauri commands delegate to application use cases.
- Infrastructure adapters implement application/domain ports.

### A3. SQLite and migrations — P0

Acceptance criteria:

- Database is created in the selected app-data location.
- Migrations run automatically and are versioned.
- Failed migrations produce a recoverable error and diagnostic log.

### A4. Diagnostics and CI — P1

Acceptance criteria:

- Local logs are available without exposing book contents.
- CI checks Rust, TypeScript, tests, formatting, and Markdown.

## Epic B — Library management

### B1. Configure library root — P0

Acceptance criteria:

- User selects a folder through a native dialog.
- Root is validated and stored as local configuration.
- Cancelling or selecting an unavailable folder is handled safely.

### B2. Relative path model — P0

Acceptance criteria:

- Persisted book and page paths contain no drive letter or root prefix.
- `..` traversal outside the root is rejected.
- Separators are normalized consistently.

### B3. Recursive scanner — P0

Acceptance criteria:

- Scan traverses the root recursively.
- Progress, cancellation, warnings, and failures are reported.
- Hidden/system/app-cache folders can be excluded by policy.

### B4. PDF discovery — P0

Acceptance criteria:

- Each supported PDF becomes one book candidate.
- Initial title derives from filename.
- Unreadable PDFs are reported without stopping the scan.

### B5. Image-folder discovery — P0

Acceptance criteria:

- A folder containing supported page images can become one book.
- Page files use deterministic natural sorting.
- Parent category folders are not incorrectly imported as books.
- Mixed-content and nested-folder rules are explicit and tested.

### B6. Catalog persistence and reconciliation — P0

Acceptance criteria:

- Repeated scans upsert rather than duplicate records.
- Changed books update derived metadata while preserving user metadata.
- Missing books are marked missing instead of immediately deleted.

### B7. Metadata extraction — P0

Acceptance criteria:

- Title, kind, page count, size, and timestamps are captured when available.
- Extraction failures are isolated per book.

### B8. Thumbnail generation — P0

Acceptance criteria:

- PDF cover uses the first renderable page.
- Image-folder cover uses the first eligible page image.
- Thumbnails are cached outside source files and can be rebuilt.

### B9. Library browser — P0

Acceptance criteria:

- Catalog can be displayed as a virtualized grid or list.
- User can view cover, title, type, status, and folder context.
- Books can be opened from the catalog.

### B10. Manual rescan and rebuild — P1

Acceptance criteria:

- User can run incremental rescan.
- User can rebuild thumbnails and repair catalog projections.

### B11. Filesystem watcher — P1

Acceptance criteria:

- Create, rename, modify, and delete events are debounced.
- Targeted reconciliation updates affected records.
- Watcher failure falls back to manual rescan rather than corrupting state.

## Epic C — Reader

### C1. Generic reader contract — P0

Acceptance criteria:

- Reader lifecycle and location model support both PDF and image folders.
- Catalog, progress, and bookmark logic do not depend on PDFium details.

### C2. PDFium packaging spike — P0

Acceptance criteria:

- A chosen binding renders a fixture PDF on Windows development and packaged builds.
- Native binary licensing and distribution are documented in an ADR.

### C3. PDF reader MVP — P0

Acceptance criteria:

- Open, render, navigate, zoom, fit, and rotate.
- Missing, corrupt, unsupported, and password-protected states are user-readable.

### C4. Image-folder reader MVP — P0

Acceptance criteria:

- Single-page and continuous reading modes work.
- Pages lazy-load and memory use remains bounded.
- Natural catalog order matches reader order.

### C5. Reading state — P0

Acceptance criteria:

- Current location is saved with debouncing.
- Reopening a book resumes at the saved location.
- Recent books list reflects activity.

### C6. Bookmarks — P1

Acceptance criteria:

- Bookmark current location with optional title and note.
- Navigate from bookmark back to the saved location.
- Bookmark persists when a source book is temporarily missing.

## Epic D — Notes and knowledge

### D1. Notes-root configuration — P0

### D2. Create and edit Markdown notes — P0

### D3. Book and location associations — P0

### D4. Markdown parsing projection — P0

### D5. Obsidian interoperability — P1

### D6. Backlinks — P1

### D7. External note watcher — P1

Common acceptance criteria:

- Markdown text remains canonical on disk.
- SQLite stores only projections and relationships.
- Files remain usable in Obsidian and plain editors.
- Reindexing does not rewrite user formatting unexpectedly.

## Epic E — Search

### E1. FTS5 schema — P0

### E2. Search-document projection — P0

### E3. Book and note indexing — P0

### E4. Global search UI — P0

### E5. Incremental index queue — P1

### E6. Rebuild and repair — P1

Acceptance criteria:

- Search is offline.
- Canonical sources can rebuild all FTS data.
- Filters distinguish books, notes, bookmarks, tags, and later OCR pages.

## Epic F — Reliability and delivery

### F1. Persistent background jobs — P1

### F2. Cancellation and recovery — P1

### F3. Backup and rebuild tools — P1

### F4. Large-library performance suite — P1

### F5. Windows installer and upgrade — P0 for release

### F6. Accessibility and keyboard navigation — P1

## Epic G — Optional intelligence

### G1. OCR module — P3

### G2. Japanese dictionary module — P3

### G3. AI provider abstraction — P3

### G4. Explain, translate, and summarize actions — P3

### G5. Anki export — P3

### G6. Plugin manifest and permissions — P3

Acceptance criteria:

- Core workflows work with every optional module disabled.
- AI output is draft content until explicitly accepted by the user.
- Network permissions and secrets are isolated from the core.

## Deferred from first release

- Multi-user collaboration.
- Hosted web application.
- Google Drive API integration.
- Mobile clients.
- Editing source PDF annotations in place.
- EPUB, CBZ, CBR, MOBI, and AZW3 support.
- Plugin marketplace and untrusted plugin sandbox.
