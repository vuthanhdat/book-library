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
10. Repair retries only missing/error covers with a 30-second cloud hydration
    window and reports cover successes and failures explicitly.

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

## Verification evidence

- Rust tests cover containment, source-kind validation, missing-parent opening,
  FTS migration, rebuild, Japanese matching, filters, diagnostics, and
  preservation of reading status/tags through catalog reconciliation.
- Frontend tests cover global results, filters, safe snippets, and existing
  Unicode catalog behavior plus Book Detail controls and hashtag normalization.
- Windows production build, hidden launch smoke, MSI packaging, and NSIS setup
  packaging passed on 2026-07-26.
- macOS Intel remains pending by explicit product sequencing.
