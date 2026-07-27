# Sprint 05 — Windows missing-source recovery and Search MVP

## Status

- **State:** Windows implementation complete; cross-platform validation pending
- **Milestone:** M4 — Search MVP
- **Platform order:** Windows 11 x64 first; macOS Intel validation later
- **Feature IDs:** LIB-016 through LIB-018, SEARCH-001 through SEARCH-007

## Delivered scope

1. Open the nearest existing authorized folder for missing books.
2. Explicitly relink a missing PDF or image folder without modifying sources.
3. Rebuildable trigram FTS5 projection for books and Markdown notes.
4. Separate note, heading, and tag search scopes.
5. Realtime global search with result filters and source navigation.
6. Persisted coalescing refresh job after catalog and note mutations.
7. Explicit rebuild, run history, issue count, and diagnostics.
8. Book Detail with persistent reading status, searchable book tags, linked
   Markdown notes, and a 30-second explicit cover retry.
9. Persistent light/dark theme selection and last-known-good cover replacement
   during repair or explicit retry.
10. Repair retries only books without a usable cover, including
    cloud-unavailable sources, with a 30-second cloud hydration window and
    reports cover successes and failures explicitly.
11. Repair restores decodable app-data covers whose catalog link was lost
    before retrying source rendering, and reports restored covers separately.
12. Repair reads missing-cover targets from the catalog without scanning the
    library filesystem; normal rescans retain their short identity probe.
13. Cover cache filenames stay bounded for unavailable books with long source
    paths, and Force cover reports timeout separately from render/save failure.
14. A panicked PDF worker no longer poisons the shared render lock and disables
    subsequent Force or Repair attempts.
15. Book Detail displays a live Force cover log for source hydration, first-page
    rendering, app-data saving, completion, and failure.
16. Library browsing orders books by normalized relative path, then by title,
    keeping books from the same source folder together.

## Windows acceptance

- replacement sources outside the library or of the wrong kind are rejected;
- no source book or Markdown file is moved, renamed, deleted, or rewritten by
  recovery or indexing;
- Japanese title, note-body, heading, and tag fixtures are searchable;
- deleting the FTS projection cannot remove canonical content;
- one failed note read does not prevent other documents from indexing;
- changed titles, paths, and notes enqueue a refresh without a full rescan;
- global results navigate to their catalog or note source.
- unavailable cloud books can be retried explicitly without clearing a previous
  cover, and failed bulk repairs preserve all last-known-good cover references.
- orphaned but valid app-data covers are relinked without reading or changing
  source books.
- an available or cloud-unavailable catalog book without a cover can enter
  Repair's queue without a discovery rescan.
- long Unicode source paths cannot become nested or overlong cover-cache paths.
- a failed PDF worker is isolated; a later cover attempt can acquire a recovered
  render lock with fresh document state on the process-lifetime PDFium binding.
- two or more PDF covers can be rendered sequentially without restarting the
  application or attempting to initialize PDFium bindings again.
- a user can see the last completed Force cover stage instead of receiving only
  an opaque final error.

## Verification evidence

- Rust tests cover containment, source-kind validation, missing-parent opening,
  FTS migration, rebuild, Japanese matching, filters, diagnostics, and
  preservation of reading status/tags through catalog reconciliation.
- Frontend tests cover global results, filters, safe snippets, and existing
  Unicode catalog behavior plus Book Detail controls and hashtag normalization.
- Windows production build, hidden launch smoke, MSI packaging, and NSIS setup
  packaging passed on 2026-07-26.
- macOS Intel remains pending by explicit product sequencing.
