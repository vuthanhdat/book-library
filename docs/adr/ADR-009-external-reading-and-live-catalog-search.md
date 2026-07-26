# ADR-009: Open source locations externally and search the catalog live

- **Status:** Accepted
- **Date:** 2026-07-26

## Context

The original M2 plan embedded PDF and image readers, reading progress, and
bookmarks. Dedicated external applications already provide stronger reading
experiences. Building a second reader would increase native rendering, caching,
navigation, and state complexity while distracting from the catalog workflow.

The catalog also needs a fast way to find one book among a large collection.
This is useful before the broader M4 knowledge-search and FTS5 work exists.

## Decision

Book Library is a catalog and launcher, not an embedded reader.

Opening a catalog book launches its source location in the operating system file
manager:

- a PDF opens its containing directory;
- an image-folder book opens the image folder itself;
- the adapter never modifies, moves, renames, or deletes the source;
- missing, invalid, or unresolved records return a typed user-safe error;
- path resolution starts from the configured machine-local root and a persisted
  validated relative path;
- Windows Explorer and macOS Finder behavior is isolated in infrastructure.

M2 also adds offline live catalog search. Results update while the user types and
match Unicode-preserving derived data already loaded from the catalog:

- title;
- relative path;
- book kind;
- availability status.

All whitespace-separated terms must match. M2 search does not read book content,
create an FTS index, or alter the M4 search architecture. M4 continues to own
FTS5 and unified search across notes, tags, and supported extracted
text.

The embedded reader, reading progress, recent-reading history, and bookmarks are
deferred. They may return only after an explicit product decision.

## Considered options

### Build the original embedded M2 reader

Rejected because it duplicates mature external readers and expands the product
away from its strongest catalog workflow.

### Open source files through their default application

Deferred. Opening the containing folder is more predictable, keeps the user in
control of application choice, and works for image-folder books.

### Implement FTS5 for M2 catalog search

Rejected for this slice. The current catalog size is small enough for responsive
in-memory filtering, while M4 needs a broader persistent search projection.

## Consequences

- PDFium remains a thumbnail adapter rather than a reader dependency.
- M2 contains no page-rendering UI or reading-state migrations.
- book-level Markdown associations remain possible; page/location associations
  remain deferred with the embedded reader.
- live catalog search is immediately useful but does not search book contents.
- macOS Intel behavior still requires validation before cross-platform
  completion.

## Implementation constraints

- All external-open operations enter through an application use case.
- React never constructs or opens absolute filesystem paths.
- The desktop boundary accepts a book ID, not a path from the frontend.
- The repository resolves catalog identity and the application validates status.
- Platform commands belong in infrastructure and receive an already authorized
  directory.
- Tests cover PDF parent selection, image-folder selection, missing/error states,
  invalid IDs, Unicode search, and multi-term matching.

## Revisit when

Revisit if users require integrated annotations, reliable cross-application
progress synchronization, or a reader experience that external applications
cannot provide.
