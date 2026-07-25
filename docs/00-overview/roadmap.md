# Product Roadmap

## Purpose

Sequence Book Library by user outcomes so the project becomes a dependable local reader before expanding into optional intelligence.

The [feature catalog](../01-product/feature-catalog.md) is authoritative for feature status and milestone ownership. The [implementation plan](../../planning/implementation-plan.md) defines engineering gates and detailed delivery scope.

## Roadmap principles

- Build a usable vertical outcome at the end of every product milestone.
- Protect filesystem, offline, relative-path, Markdown, and non-destructive invariants throughout delivery.
- Establish reader reliability before advanced knowledge or AI features.
- Do not require cloud credentials, external services, or AI keys for core workflows.
- Delay plugin and multi-device complexity until stable module boundaries and recovery paths exist.

## Milestone sequence

```mermaid
flowchart LR
    M0["M0 Engineering foundation"] --> M1["M1 Library MVP"]
    M1 --> M2["M2 Reading MVP"]
    M2 --> M3["M3 Knowledge MVP"]
    M3 --> M4["M4 Search MVP"]
    M4 --> M5["M5 Reliability and release"]
    M5 --> M6["M6 Optional intelligence"]
```

| Milestone | User/developer outcome | Depends on |
|---|---|---|
| M0 — Engineering foundation | The repository builds and launches a typed, tested desktop shell with SQLite migrations and enforceable boundaries. | Accepted architecture baseline |
| M1 — Library MVP | The user selects one folder and browses an idempotently discovered catalog of PDFs and image-folder books. | M0 |
| M2 — Reading MVP | The user opens either supported book kind, reads smoothly, resumes progress, and uses bookmarks. | M1, PDFium spike |
| M3 — Knowledge MVP | The user creates portable Markdown notes linked to books and reading locations and uses them with external editors. | M2 |
| M4 — Search MVP | The user searches books, notes, bookmarks, tags, and supported extracted text fully offline. | M1, M3 |
| M5 — Reliability and release | The installed Windows app watches, recovers, backs up, repairs, and upgrades safely on a real library. | M1–M4 |
| M6 — Optional intelligence | OCR, Japanese dictionary, AI assistance, Anki, and module experiments add value without becoming core dependencies. | M5 |

## Dependency anchors

- `Library` and `Book` exist before reader sessions and reading state.
- `ReadingLocation` exists before bookmarks and location-linked notes.
- canonical Markdown notes exist before backlinks and note search.
- rebuildable `SearchDocument` projections exist before FTS5 query features.
- optional providers use stable application ports and never become prerequisites for M1–M5.
- installer/recovery work is not complete until user-owned files and non-rebuildable local state are protected.

## Release direction

- `0.1.0` targets M0–M2: foundation, library management, PDF/image reading, progress, and bookmarks.
- `0.2.0` targets M3: Markdown notes and Obsidian interoperability.
- `0.3.0` targets M4–M5: local search, reliability, recovery, performance, packaging, and the first dependable daily-use release.
- M6 begins only after the core release gates pass.

Release numbers are planning targets, not promises. A release does not ship merely because its documents are complete.

## Deferred direction

These themes remain outside the committed roadmap until explicitly promoted:

- multiple libraries and profiles;
- hosted web or mobile clients;
- account-based or Google Drive API sync;
- untrusted plugin sandbox/marketplace;
- semantic/vector search;
- additional ebook/archive formats;
- multi-user collaboration;
- in-place PDF annotation modification.

Promoting a deferred theme requires updates to product requirements, the feature catalog, this roadmap, and any affected ADRs.