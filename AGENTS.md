# AGENTS.md

# Book Library

This document defines the engineering rules for every AI coding agent working on this repository.

The goal is to ensure all implementations remain consistent with the project architecture, regardless of which AI model or human contributor writes the code.

---

# 1. Mission

Book Library is a desktop-first, offline-first personal knowledge platform.

It is **NOT**

- a cloud service
- a web application
- an online ebook platform
- a database-centric system

It **IS**

- a local application
- a filesystem-centric application
- a reader
- a knowledge management tool

---

# 2. Core Principles

These principles are mandatory.

## Filesystem First

The filesystem is the source of truth.

Books are never copied into SQLite.

SQLite only stores metadata and indexes.

---

## Offline First

Everything required for daily usage must work without Internet.

Internet is optional.

AI is optional.

---

## Relative Path Only

Persist only relative paths.

Never store absolute paths inside SQLite.

Absolute paths are reconstructed at runtime.

---

## Markdown First

Notes are Markdown files.

SQLite stores only projections.

Markdown remains the canonical source.

---

## AI Optional

The application must remain fully usable without AI.

No workflow may depend on an external AI service.

---

# 3. Architecture

Always follow this dependency direction.

Presentation

↓

Tauri Commands

↓

Application Use Cases

↓

Domain

↓

Infrastructure

Dependencies only point downward.

Never reverse dependencies.

---

# 4. Layer Responsibilities

## Presentation

Responsible for

- UI
- user interaction
- view models
- routing

Never

- query SQLite
- access filesystem
- contain business rules

---

## Application

Responsible for

- use cases
- orchestration
- transactions

Never

- contain UI logic
- contain SQL

---

## Domain

Responsible for

- entities
- value objects
- business rules

Never

- reference Tauri
- reference SQLite
- reference React

---

## Infrastructure

Responsible for

- SQLite
- Filesystem
- PDFium
- OCR
- AI
- Markdown

Never

- contain business rules

---

# 5. Module Boundaries

Current modules

- Library
- Reader
- Notes
- Search
- AI
- Settings

Modules communicate through interfaces.

Never import another module's internal implementation.

---

# 6. Database Rules

SQLite stores

- metadata
- settings
- jobs
- indexes
- reading state

SQLite never stores

- PDFs
- images
- Markdown note contents

---

# 7. File Rules

Never modify user files automatically.

Allowed

- create Markdown notes
- create thumbnails
- create cache

Forbidden

- rename books
- move books
- rewrite PDFs
- rewrite image folders

unless explicitly requested by the user.

---

# 8. Background Jobs

Long-running work must execute as background jobs.

Examples

- library scan
- thumbnail generation
- OCR
- search indexing

Jobs must

- report progress
- support cancellation
- survive application restart

---

# 9. Event Driven

Modules communicate using events.

Typical events

- LibraryInitialized
- ScanStarted
- ScanCompleted
- BookAdded
- BookUpdated
- BookRemoved
- ReadingProgressChanged
- NoteUpdated

Avoid direct module coupling whenever possible.

---

# 10. Error Handling

Errors must be recoverable.

Prefer

Recover → Retry → Continue

instead of

Crash → Exit

User data is more important than strict correctness.

---

# 11. Coding Rules

Prefer

Small functions.

Composable modules.

Explicit types.

Immutable data where practical.

Avoid

God objects.

Massive services.

Hidden global state.

Duplicate logic.

---

# 12. Naming

Use consistent names.

Examples

InitializeLibraryUseCase

BookRepository

ThumbnailGenerator

ReadingStateRepository

SearchIndexer

RelativePath

BookKind

ReadingLocation

---

# 13. Documentation

Every new feature must update

- Architecture (if needed)
- Database (if changed)
- API contract
- Planning
- Feature specification

Documentation is part of the implementation.

---

# 14. Before Writing Code

Every AI agent should verify

- Does this violate an ADR?
- Does this duplicate existing functionality?
- Does this belong in the correct layer?
- Is there already a Use Case for this?
- Can this be implemented without coupling modules?

If any answer is "No", stop and redesign first.

---

# 15. Success Criteria

A feature is considered complete only if

- architecture remains clean
- tests pass
- documentation is updated
- no duplicated logic is introduced
- no unnecessary coupling is added
- future AI agents can understand the implementation