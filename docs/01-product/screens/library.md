# Library Screen

## Purpose

The Library Screen is the primary entry point of the application.

Its purpose is to help users organize, browse, search and open books stored inside their personal library.

This screen is responsible only for library management.

It does not display book content.

---

# User Goals

Users should be able to

- browse all books
- discover newly added books
- detect missing books
- search books
- filter books
- sort books
- open a book
- inspect metadata
- rescan the library

---

# Layout

```

+-----------------------------------------------------------+
| Toolbar |
+-----------------------------------------------------------+

Search...

[Filter]

[Sort]

[Rescan]

-------------------------------------------------------------

Grid/List of Books

-------------------------------------------------------------

Status Bar

Books: 521

Indexed: 521

Missing: 3

Scanning...

```

---

# Main Sections

## Toolbar

Contains

- Search Box
- Filter Button
- Sort Button
- Rescan Button
- View Mode Switch

---

## Book List

Displays

- Cover
- Title
- Author
- Progress
- Last Read
- Tags
- File Type

Supports

- Grid View
- List View

---

## Status Bar

Displays

- Total Books
- Scan Status
- Index Status

---

# Main Actions

## Initialize Library

Shown only during first launch.

User selects a root folder.

System scans the library.

System creates metadata database.

---

## Open Book

Double-click

↓

Open Reader Screen

---

## Search

User enters keywords.

Results update immediately.

Search should remain responsive even for large libraries.

---

## Filter

Examples

Unread

Reading

Finished

PDF

Image Folder

Favorites

Tags

Language

---

## Sort

Examples

Title

Author

Recently Added

Recently Opened

Progress

---

## Rescan Library

Detect

- newly added books
- removed books
- renamed books

Existing reading progress must be preserved whenever possible.

---

# Navigation

```

Library

↓

Reader

↓

Back

↓

Library

```

---

# Required Data

Each book displays

- Cover Thumbnail
- Title
- Relative Path
- File Type
- Last Opened
- Reading Progress
- Tags
- Favorite Flag

---

# Related Use Cases

InitializeLibrary

ScanLibrary

SearchBooks

OpenBook

UpdateReadingProgress

GenerateThumbnail

---

# Related Database Tables

books

reading_progress

thumbnails

tags

book_tags

---

# Related Events

LibraryInitialized

BookAdded

BookRemoved

BookUpdated

ScanStarted

ScanCompleted

ThumbnailGenerated

---

# Empty States

## No Library

Show

"Choose your library folder."

Button

Initialize Library

---

## Empty Library

"No books were found."

Button

Rescan

---

## Searching

"No matching books."

---

# Error States

Library not found

↓

Offer to relocate library

---

Book missing

↓

Mark as Missing

↓

Allow Rescan

---

Thumbnail generation failed

↓

Show placeholder

↓

Retry later

---

# Performance Targets

Open screen

< 300 ms

Search response

< 100 ms

Open reader

< 500 ms

Thumbnail loading

Lazy loaded

---

# Future Extensions

Multiple libraries

Collections

Smart Collections

Pinned Books

Recent Books

Cloud Sync (optional)

Plugin-provided metadata