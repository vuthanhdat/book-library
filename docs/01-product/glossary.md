# Glossary

This document defines the shared vocabulary used throughout the Book Library project.

Every document, database schema, API, and source code should use these terms consistently.

---

# Library

A Library is the root directory selected by the user that contains books.

A Library is **not** the SQLite database.

A Library is **not** a cloud service.

A Library is simply a filesystem location.

Example

```
D:\Books
```

---

# Book

A Book is a readable item inside the Library.

A Book may be

- PDF
- Folder containing page images

Future formats

- EPUB
- CBZ

A Book is identified by its relative path.

---

# Book Format

Describes how a book is physically stored.

Examples

- PDF

- Image Folder

Future

- EPUB

- CBZ

---

# Relative Path

The path of a book relative to the Library Root.

Example

Library Root

```
D:\Books
```

Book

```
D:\Books\Japanese\N2\Book.pdf
```

Stored value

```
Japanese/N2/Book.pdf
```

Only relative paths are persisted.

---

# Absolute Path

The actual filesystem path.

Absolute paths are reconstructed at runtime.

They must never be stored in SQLite.

---

# Metadata

Information describing a book.

Examples

- Title
- Author
- Language
- Tags
- Cover Thumbnail
- Reading Progress

Metadata is stored in SQLite.

Metadata never replaces the original file.

---

# Thumbnail

A generated preview image representing a book.

Thumbnails are cached.

They can always be regenerated.

They are disposable.

---

# Reading Progress

The last known reading location of a user.

Examples

PDF

- Page Number

Image Folder

- Image Index

Reading Progress should always be restored automatically.

---

# Bookmark

A user-created marker pointing to a specific location inside a book.

Bookmarks are intentional.

Reading Progress is automatic.

These are different concepts.

---

# Highlight

A selected piece of text or page region.

Highlights may later become Notes.

Highlights are optional.

---

# Note

A Markdown document created by the user.

Notes belong to the user.

Markdown files are the canonical source.

SQLite stores only metadata.

---

# Reading Session

A continuous period during which a user is reading a book.

A Reading Session starts when a book is opened.

It ends when the book is closed.

Reading Sessions may be used for future statistics.

---

# Scanner

The component responsible for discovering books inside the Library.

Responsibilities

- Detect new books
- Detect removed books
- Detect renamed books
- Update metadata

Scanner never modifies user files.

---

# Search Index

A local index used to provide fast search.

The Search Index is derived data.

It can always be rebuilt.

---

# Cache

Temporary data created to improve performance.

Examples

- Thumbnails
- OCR Results
- Rendered Pages

Cache should never contain user-owned data.

Cache can always be deleted.

---

# Job

A background task executed asynchronously.

Examples

- Scan Library
- Generate Thumbnail
- OCR
- Search Index Update

Jobs should report progress.

Jobs should support cancellation.

---

# Event

A notification indicating that something has happened.

Examples

BookAdded

BookRemoved

ScanCompleted

ReadingProgressChanged

Events are used to reduce coupling between modules.

---

# Projection

Data derived from another source.

Examples

SQLite metadata

Search Index

Thumbnail Cache

Projections can always be rebuilt.

The original source remains authoritative.

---

# Source of Truth

The authoritative version of data.

Books

→ Filesystem

Notes

→ Markdown

Metadata

→ SQLite

Search Index

→ Derived

Cache

→ Derived

---

# Reader

The component responsible for displaying book content.

Reader responsibilities

- Open books
- Render pages
- Restore progress
- Navigate pages

Reader is not responsible for Library management.

---

# Library Scan

The process of synchronizing SQLite metadata with the filesystem.

Scanning should detect

- Added books
- Removed books
- Renamed books (when possible)

Scanning should never modify the user's Library.

---

# AI Feature

An optional capability powered by an AI model.

Examples

- OCR
- Translation
- Summary
- Dictionary
- Note Suggestions

The application must remain fully usable without AI.

---

# Offline Mode

The normal operating mode of the application.

Offline Mode must support

- Reading
- Searching
- Notes
- Bookmarks
- Progress Tracking

Internet access should never be required for core functionality.

---

# Future Terms

Additional glossary entries should be added whenever a new domain concept is introduced.

Avoid introducing multiple names for the same concept.