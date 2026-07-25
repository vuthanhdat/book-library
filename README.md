# Book Library

Book Library is a desktop-first, offline-first application for managing and reading a personal collection of PDF books and image-folder books on Windows 11 x64 and macOS Intel x64.

The application treats the user's filesystem as the source of truth. It does not upload books, copy them into a database, or require a cloud account. A folder synchronized by Google Drive Desktop is treated as an ordinary local folder.

> **Current status:** specification and engineering-foundation phase. The repository does not contain the Tauri/React/Rust application scaffold yet. Implementation starts with [Sprint 01](planning/sprint-01.md).

## Product scope

The first usable release will support:

- selecting one local library root;
- scanning PDF files and folders of ordered page images;
- generating rebuildable thumbnails and metadata;
- browsing and opening books;
- restoring reading progress and bookmarks;
- storing notes as portable Markdown files;
- searching local metadata and notes with SQLite FTS5.

OCR, Japanese dictionary lookup, AI assistance, Anki export, additional book formats, and plugins are optional later capabilities. They are not dependencies of the core reader.

## Non-negotiable principles

- **Two required desktop platforms:** Windows 11 x64 and macOS Intel x64 are supported from the engineering foundation onward through one shared codebase.
- **Offline first:** daily reading and knowledge workflows work without Internet access.
- **Filesystem first:** source books remain in user-owned folders.
- **Relative paths only:** persisted book and note references never contain machine-specific absolute paths.
- **Markdown first:** note text remains readable outside the application.
- **AI optional:** no core workflow depends on an external model or service.
- **Non-destructive:** scans and readers do not rename, move, delete, or rewrite source books.

## Start reading here

1. [Documentation index](docs/README.md) — document map, authority rules, and reading order.
2. [Product requirements](docs/01-product/requirements.md) — product scope and constraints.
3. [Feature catalog](docs/01-product/feature-catalog.md) — feature IDs, status, and milestone ownership.
4. [System architecture](docs/02-architecture/architecture.md) — layers, modules, runtime boundaries, and data ownership.
5. [Architecture decisions](docs/adr/README.md) — decisions that override unresolved options in design documents.
6. [Implementation plan](planning/implementation-plan.md) — milestone delivery sequence.
7. [Sprint 01](planning/sprint-01.md) — first executable engineering scope.

## Planned implementation stack

- Tauri 2 desktop shell
- React and TypeScript frontend
- Tailwind CSS and shadcn/ui
- Rust application, domain, and infrastructure modules
- SQLite with FTS5
- PDFium behind a reader adapter
- Markdown notes compatible with normal editors and Obsidian

The exact dependency choices are confirmed by ADRs before they are introduced into code.

## Repository workflow

Before implementing a feature:

1. locate its feature ID in the feature catalog;
2. read the relevant product and module specification;
3. check applicable ADRs;
4. confirm that the work is included in the current milestone or sprint;
5. update tests and documentation with the implementation.

See [AGENTS.md](AGENTS.md) for mandatory architecture rules and [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow.