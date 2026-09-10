# Domain Model

This document describes the core business domain of Book Library.

It is independent of implementation details such as SQLite, Tauri, React or Rust.

The Domain Model defines:

- Core business objects
- Ownership
- Relationships
- Source of Truth
- Aggregate boundaries

---

# Domain Overview

```

Library

├── Books

│ ├── Reading Progress

│ ├── Bookmarks

│ ├── Tags

│ └── Notes

│

├── Scanner

├── Search Index

├── Cache

└── Settings

```

---

# Aggregate Roots

The following objects are Aggregate Roots.

| Aggregate | Description |
|------------|-------------|
| Library | Root of the user's collection |
| Book | Represents one readable item |
| Note | Represents one Markdown note |
| Settings | User configuration |

Everything else belongs to one of these aggregates.

---

# Library

Represents the user's book collection.

Responsibilities

- Knows the Library Root
- Owns Books
- Coordinates Scanner
- Coordinates Search Index

Owns

- Books
- Scanner
- Search Index

Source of Truth

Filesystem

---

# Book

Represents one readable publication.

A Book may be

- PDF
- Image Folder

A Book owns

- Reading Progress
- Bookmarks
- Tags
- Thumbnail Metadata

A Book is associated with its Notes through the note projection, but it does
not own the Markdown files or their contents.

Source of Truth

Filesystem

Projection

SQLite Metadata

---

# Reading Progress

Represents where the user stopped reading.

Owned by

Book

Source of Truth

SQLite

Exactly one Reading Progress exists per Book.

---

# Bookmark

Represents a user-created location.

Owned by

Book

Multiple Bookmarks may exist.

Source of Truth

SQLite

---

# Tag

Represents a user classification.

Tags are shared.

Books reference Tags.

Many-to-many relationship.

---

# Note

Represents a Markdown document.

Source of Truth

Markdown File

Projection

SQLite Metadata

A managed Note references exactly one Book. The Markdown file remains
user-owned and portable; the book association is represented in frontmatter
and the rebuildable SQLite projection.

---

# Thumbnail

Represents a generated preview image.

Owned by

Book

Source of Truth

Cache

Can always be regenerated.

---

# Scanner

Synchronizes

Filesystem

↓

SQLite

Scanner never modifies user files.

Scanner is stateless.

---

# Search Index

Optimized representation for searching.

Source of Truth

Derived

Can always be rebuilt.

---

# Cache

Contains temporary data.

Examples

- Thumbnails
- OCR
- Render Cache

Disposable.

---

# Settings

Represents user preferences.

Examples

- Theme
- Library Root
- Reader Preferences
- OCR
- AI

Source of Truth

SQLite

---

# Ownership Rules

Library owns

- Books

Book owns

- Reading Progress
- Bookmarks
- Thumbnail Metadata

Tag owns nothing.

Scanner owns nothing.

Cache owns nothing.

Search Index owns nothing.

---

# Source of Truth Matrix

| Object | Source of Truth |
|----------|----------------|
| Library | Filesystem |
| Book | Filesystem |
| Reading Progress | SQLite |
| Bookmark | SQLite |
| Tag | SQLite |
| Note | Markdown |
| Search Index | Derived |
| Thumbnail | Cache |
| Settings | SQLite |

---

# Projections

The following objects are projections.

- SQLite Metadata
- Search Index
- Thumbnail Cache

Projections may always be rebuilt.

---

# Object Relationships

```

Library

│

├── owns ─────► Book

│ │

│ ├── Reading Progress

│ ├── Bookmark

│ ├── Thumbnail

│ └── Tags

│

└──────────────► Scanner

Book ◄──────────── Note

Book ◄──────────── Tag

Book ◄──────── Search Index

```

---

# Domain Invariants

The following rules must always be true.

## Library

A Library always has exactly one Root Folder.

---

## Book

A Book always has one Relative Path.

---

## Reading Progress

At most one Reading Progress exists per Book.

---

## Bookmark

Bookmarks always belong to one Book.

---

## Notes

Every managed Note references exactly one Book.

Books do not own Markdown note files, but their linked notes are grouped and
shown in the book workflow.

---

## Cache

Cache can always be deleted.

---

## Search Index

Search Index can always be rebuilt.

---

# Design Principles

The Domain must remain independent from

- React
- Tauri
- SQLite
- PDFium
- OCR
- AI

Infrastructure changes must never require changes to the Domain Model.
