# Book Library

Book Library is a desktop-first, offline-first application for managing and reading a personal collection of PDF books and image-folder books on Windows 11 x64 and macOS Intel x64.

The application treats the user's filesystem as the source of truth. It does not upload books, copy them into a database, or require a cloud account. A folder synchronized by Google Drive Desktop is treated as an ordinary local folder.

> **Current status:** The revised Windows M2 pass is complete. The app can
> configure and scan a real library, search the catalog live, and open book
> locations in Windows Explorer. macOS Intel validation remains deferred.

## Product scope

The first usable release will support:

- selecting one local library root;
- scanning PDF files and folders of ordered page images;
- generating rebuildable thumbnails and metadata;
- browsing books, searching the catalog live, and opening source folders in the
  operating system file manager;
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
7. [Sprint 02](planning/sprint-02.md) — active Library MVP delivery record.

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

## Engineering-foundation commands

Install Node.js 24 and the current stable Rust toolchain, then run:

```text
npm ci
npm run typecheck
npm run test
npm run build
npm run check:links
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

The CI workflow runs the same quality gates on Windows x64 and the GitHub-hosted
`macos-15-intel` runner. A real Intel Mac smoke test remains required by Sprint 01.

## Local operational data

The database and structured JSON-line logs are stored under the Tauri-resolved
application-data directory, never under the configured library root.

- Windows development: `%APPDATA%/dev.booklibrary.desktop/`
- macOS development: `~/Library/Application Support/dev.booklibrary.desktop/`
- Logs: the `logs/` child directory, with daily `book-library.jsonl` files

Logs include safe operation IDs, event names, error codes, and platform metadata.
They must not contain note bodies, extracted book text, secrets, page images, or
unnecessary absolute source paths.
