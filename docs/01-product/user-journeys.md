# User Journeys

This document describes how users interact with Book Library from the first launch through daily usage.

These journeys define the expected product behavior and guide the implementation of screens, database design, APIs, and background jobs.

---

# Journey 1 - First Launch

Goal

Initialize the application for first use.

Flow

Application Starts

↓

No Library Configured

↓

Welcome Screen

↓

Choose Library Folder

↓

Initialize Library

↓

Background Scan

↓

Generate Thumbnails

↓

Build Search Index

↓

Open Library Screen

Expected Result

- Library successfully initialized
- Books imported
- Search index created
- Ready for reading

---

# Journey 2 - Daily Reading

Goal

Open a book and continue reading.

Flow

Launch App

↓

Library Screen

↓

Recently Read Books

↓

Open Book

↓

Reader Screen

↓

Read

↓

Auto Save Reading Progress

↓

Close Book

↓

Return to Library

Expected Result

Reading position is automatically restored next time.

---

# Journey 3 - Discover New Books

Goal

Import newly added books without losing existing data.

Flow

User Copies Books Into Library Folder

↓

Press Rescan

↓

Scanner Detects Changes

↓

Import New Books

↓

Generate Missing Thumbnails

↓

Update Search Index

↓

Refresh Library

Expected Result

Existing books remain unchanged.

Only new books are imported.

---

# Journey 4 - Deleted Books

Goal

Handle removed books safely.

Flow

User Deletes Book Outside Application

↓

Rescan

↓

Scanner Detects Missing File

↓

Book Marked As Missing

↓

Reading Progress Preserved

↓

User May

- Ignore
- Remove Metadata
- Relocate File

Expected Result

No data loss.

---

# Journey 5 - Reading Notes

Goal

Create notes while reading.

Flow

Open Book

↓

Select Text

↓

Create Note

↓

Markdown File Created

↓

Backlinks Generated

↓

Continue Reading

Expected Result

Markdown note remains editable outside the application.

---

# Journey 6 - Search

Goal

Find information quickly.

Flow

Search

↓

Local Search Index

↓

Book Results

↓

Open Reader

or

↓

Open Note

Expected Result

Results should appear instantly.

---

# Journey 7 - Reading Progress

Goal

Never lose progress.

Flow

Reader Open

↓

Page Changed

↓

Update Progress

↓

SQLite Updated

↓

Application Closed

↓

Application Opened Again

↓

Resume Reading

Expected Result

Resume exactly where the user stopped.

---

# Journey 8 - Thumbnail Generation

Goal

Generate thumbnails only when needed.

Flow

New Book Found

↓

Thumbnail Exists?

↓

No

↓

Background Thumbnail Job

↓

Cache Thumbnail

↓

Refresh UI

Expected Result

UI remains responsive.

---

# Journey 9 - Library Relocation

Goal

Move library without losing metadata.

Flow

Drive Letter Changed

↓

Library Missing

↓

Prompt User

↓

Select New Root

↓

Rebuild Absolute Paths

↓

Continue

Expected Result

Metadata remains valid because only relative paths are stored.

---

# Journey 10 - Settings

Goal

Customize the application.

Flow

Open Settings

↓

Modify Configuration

↓

Persist Settings

↓

Apply Changes

Expected Result

Restart should not be required whenever possible.

---

# Background Processes

The following work should always execute in the background.

- Library Scan
- Thumbnail Generation
- Search Index Update
- OCR
- Cache Cleanup

These jobs must never block the user interface.

---

# Design Principles

All journeys follow these principles.

- User files are never modified automatically.
- Reading progress is never lost.
- Metadata should survive file relocation whenever possible.
- The application should remain usable while background jobs are running.
- AI features must never interrupt the reading experience.