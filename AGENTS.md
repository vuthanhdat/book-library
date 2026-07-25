# Engineering Rules for AI Agents

These rules are mandatory for every coding agent and contributor working in this repository.

## Repository status

The repository is currently in the engineering-foundation phase. Do not describe a feature as implemented unless the corresponding source, tests, and configuration exist in the branch being reviewed.

Before changing code, read:

1. [documentation authority and map](docs/README.md);
2. the relevant product or module specification;
3. [accepted ADRs](docs/adr/README.md);
4. the active milestone and sprint documents.

When documents disagree, follow the authority order in `docs/README.md`.

## Product invariants

- Desktop first: Windows 11 is the first supported platform.
- Offline first: core library, reader, notes, and search workflows require no Internet access.
- Filesystem first: source books remain in user-owned folders.
- Markdown first: user-authored note text remains in `.md` files.
- Relative paths only: content references are persisted relative to a configured root.
- AI optional: disabling every AI/provider module must not break core workflows.
- Non-destructive: never rename, move, delete, or rewrite source books automatically.

## Architecture

The initial application is a Rust modular monolith with React presentation.

```text
React presentation
    -> Tauri desktop boundary
        -> application use cases and ports
            -> domain rules

infrastructure adapters
    -> implement application/domain ports
```

Compile-time dependencies point inward:

- `domain` imports no Tauri, SQLite, filesystem, PDFium, React, or provider code;
- `application` depends on domain and owns use cases, transactions, jobs, and ports;
- `infrastructure` implements ports and may depend on application/domain contracts;
- `desktop` composes adapters and exposes thin Tauri commands/events;
- React owns screens and view state, not authoritative business rules.

Call flow is not dependency direction. A use case may call an infrastructure port, but the adapter depends on the port contract—not the reverse.

Expected initial Rust layout:

```text
src-tauri/src/
  domain/
  application/
  infrastructure/
  desktop/
```

Use narrow visibility by default. Do not create a generic `utils` module for unrelated helpers. Do not import another feature's private repository or adapter.

## Layer responsibilities

### Presentation

May contain components, routing, view models, transient UI state, formatting, and client-side interaction validation.

Must not access SQLite, inspect source folders directly, or implement catalog/reader/note business rules.

### Desktop boundary

May deserialize payloads, perform boundary validation, map errors, call use cases, and emit typed events.

Must not contain SQL, filesystem traversal, or business decisions.

### Application

Owns use cases, orchestration, transaction boundaries, cancellation, job coordination, and infrastructure ports.

Must not contain React or Tauri UI concerns, raw SQL, or provider-specific SDK behavior.

### Domain

Owns entities, value objects, invariants, policies, and domain errors.

Must remain deterministic and infrastructure-independent.

### Infrastructure

Owns SQLite repositories, migrations, filesystem scanning/watching, PDFium, Markdown I/O, FTS5, thumbnails, OCR, and provider adapters.

Must not invent business policy that belongs in domain or application.

## Data and filesystem safety

SQLite may store metadata, settings, relationships, reading state, jobs, and rebuildable indexes. It must not be the canonical store for PDFs, page images, or Markdown note bodies.

The database, cache, thumbnails, and logs live in OS application data. Source folders must not receive hidden application infrastructure. See `ADR-005`.

Persisted content paths must:

- use `/` as the normalized separator;
- contain no drive letter or UNC root;
- contain no leading root separator;
- reject `..` traversal outside the configured root;
- preserve valid Unicode names.

An absolute library or notes root is machine-local configuration. It is not a book/note identity and must not leak into persisted content references.

## Use cases, events, and jobs

Every user operation enters through an application use case. Prefer explicit names such as:

- `InitializeLibrary`
- `RescanLibrary`
- `OpenBook`
- `SaveReadingProgress`
- `CreateBookNote`
- `SearchLibrary`

Long-running operations such as scans, thumbnails, indexing, and OCR must report progress and isolate per-item failures. Add cancellation and restart recovery according to the active milestone's acceptance criteria.

Events communicate facts and progress; they do not replace use-case calls or transactions. Use stable, past-tense names and typed payloads.

## Error handling and logging

- Return typed domain/application errors.
- Map errors to stable user-safe codes at the desktop boundary.
- Prefer recover, retry, skip-one-item, or continue over crashing the app.
- Never silently swallow errors.
- Never log note bodies, extracted book text, API keys, or secrets.
- Avoid logging full absolute user paths unless diagnostics explicitly require and redact them.

## Testing requirements

Add the smallest effective test at the owning layer:

- domain unit tests for invariants and policies;
- application tests with fake ports for orchestration;
- infrastructure integration tests with temporary folders/databases;
- contract tests for Tauri payloads and error mapping;
- frontend tests for meaningful interaction behavior.

Critical rules such as path normalization, natural image ordering, idempotent discovery, non-destructive scanning, migrations, and progress restoration require tests.

## Documentation rules

Documentation is part of the change, but do not duplicate rules across files.

- new/reversed technical choice → ADR;
- product behavior/scope change → requirements and feature catalog;
- module ownership/dependency change → architecture documents;
- persistence change → migration/schema and recovery notes;
- delivery-scope change → plan, backlog, or active sprint;
- completed feature → update feature status only after tests pass.

Remove stale claims and unresolved alternatives when an ADR settles them. Never mark planned work as completed based only on a design document.

## Before committing

Verify:

- the change belongs to the active scope;
- no accepted ADR is violated;
- no duplicate use case, repository, path logic, or UI implementation was introduced;
- user-owned files remain safe;
- tests cover critical behavior;
- documentation reflects the actual branch;
- generated/debug files and secrets are not included.

A feature is complete only when its acceptance criteria pass, relevant tests pass, documentation matches implementation, and recovery/error behavior is handled.