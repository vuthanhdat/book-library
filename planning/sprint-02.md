# Sprint 02 — Windows Library MVP

## Status

- **State:** Windows Done — macOS Intel validation pending
- **Milestone:** M1 — Library MVP
- **Platform order:** Windows 11 x64 implementation first; macOS Intel validation later
- **Feature IDs:** LIB-001 through LIB-011

## Goal

Configure one real library root, scan PDFs and image folders without modifying
them, reconcile an idempotent SQLite catalog, generate rebuildable thumbnails,
and browse the resulting catalog on Windows.

The shared M1 implementation and Windows acceptance pass are complete. M0 and
M1 macOS evidence remains pending, so the cross-platform milestone is not closed.

## Work packages

1. Configure and validate one machine-local root (`M1-01`).
2. Run a cancellable recursive scanner with typed progress/issues (`M1-02`).
3. Discover PDFs and eligible image folders under ADR-008 (`M1-03`).
4. Reconcile catalog records idempotently and mark missing (`M1-04`).
5. Extract core metadata with per-item failure isolation (`M1-05`).
6. Generate and repair thumbnails only in app-data (`M1-06`).
7. Browse catalog as grid/list with honest states (`M1-07`).
8. Manually rescan and repair derived state (`M1-08`).

## Windows acceptance

- selecting `H:/My Drive/07_NEW_KINDLE` saves only machine-local root
  configuration;
- scans persist only normalized relative book/page paths;
- repeated scans do not create duplicates;
- unavailable cloud-only sources become `unavailable`; sources absent from a
  completed scan become `missing`; neither is deleted automatically;
- one failed candidate does not block successful candidates;
- cancellation leaves prior committed batches valid and retryable;
- thumbnails and operational files remain outside the source root;
- grid/list browsing uses real catalog data and shows scan progress/issues;
- a before/after source inventory confirms scanning is non-destructive.

## Out of scope

- reading book contents in the application;
- filesystem watcher production behavior;
- fuzzy rename detection;
- editing source books;
- macOS Intel validation during the Windows implementation pass.

## Windows result

- All eight work packages are `Windows Done`.
- Rust: 24 tests pass and one real-library smoke test is opt-in.
- Frontend: three interaction tests, type checking, and production build pass.
- The read-only real-library smoke traversed more than 27,900 entries, cataloged
  1,019 unique books on its first pass, added zero books on its second pass, and
  confirmed the 195-entry root inventory was unchanged.
- Google Drive availability varied between passes; unseen records were retained
  as missing rather than deleted.
- Required macOS Intel validation remains intentionally deferred.
