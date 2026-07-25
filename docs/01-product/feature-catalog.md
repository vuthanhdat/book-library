# Feature Catalog

This document provides a complete inventory of all product features.

Each feature has

- unique identifier
- business value
- implementation status
- dependencies
- target milestone

This document is the primary reference for planning and implementation.

---

# Feature Status

| Status | Meaning |
|---------|---------|
| Planned | Defined but not started |
| In Progress | Currently under development |
| Completed | Fully implemented |
| Future | Not part of current roadmap |

---

# Milestones

M1 — Foundation

M2 — Library

M3 — Reader

M4 — Knowledge

M5 — Search

M6 — AI

---

# Library

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| LIB-001 | Initialize Library | Planned | M1 |
| LIB-002 | Scan Library | Planned | M1 |
| LIB-003 | Detect New Books | Planned | M2 |
| LIB-004 | Detect Missing Books | Planned | M2 |
| LIB-005 | Generate Thumbnails | Planned | M2 |
| LIB-006 | Book Metadata | Planned | M2 |
| LIB-007 | Favorite Books | Future | M5 |
| LIB-008 | Multiple Libraries | Future | M6 |

---

# Reader

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| READ-001 | Open PDF | Planned | M2 |
| READ-002 | Open Image Folder | Planned | M2 |
| READ-003 | Next / Previous Page | Planned | M2 |
| READ-004 | Restore Reading Progress | Planned | M2 |
| READ-005 | Zoom | Planned | M2 |
| READ-006 | Fit Width | Planned | M2 |
| READ-007 | Fit Height | Planned | M2 |
| READ-008 | Fullscreen | Planned | M2 |
| READ-009 | Keyboard Shortcuts | Planned | M2 |

---

# Reading Progress

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| PROG-001 | Auto Save Progress | Planned | M2 |
| PROG-002 | Resume Reading | Planned | M2 |
| PROG-003 | Reading Statistics | Future | M5 |

---

# Bookmark

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| BOOKMARK-001 | Add Bookmark | Planned | M3 |
| BOOKMARK-002 | Delete Bookmark | Planned | M3 |
| BOOKMARK-003 | Bookmark List | Planned | M3 |

---

# Notes

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| NOTE-001 | Create Markdown Note | Planned | M3 |
| NOTE-002 | Link Note To Book | Planned | M3 |
| NOTE-003 | Open Note In Obsidian | Planned | M3 |
| NOTE-004 | Backlinks | Future | M5 |
| NOTE-005 | Graph View | Future | M6 |

---

# Search

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| SEARCH-001 | Search Books | Planned | M4 |
| SEARCH-002 | Full Text Search | Planned | M4 |
| SEARCH-003 | Search Notes | Planned | M4 |
| SEARCH-004 | Search Tags | Planned | M4 |

---

# OCR

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| OCR-001 | OCR Image Page | Future | M5 |
| OCR-002 | OCR PDF Page | Future | M5 |

---

# Dictionary

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| DICT-001 | Japanese Dictionary | Future | M5 |
| DICT-002 | Vietnamese Meaning | Future | M5 |
| DICT-003 | Kanji Lookup | Future | M5 |
| DICT-004 | Pitch Accent | Future | M6 |

---

# AI

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| AI-001 | Translation | Future | M6 |
| AI-002 | Summary | Future | M6 |
| AI-003 | Flashcard Generation | Future | M6 |
| AI-004 | Note Suggestions | Future | M6 |

---

# Settings

| ID | Feature | Status | Milestone |
|----|----------|---------|------------|
| SET-001 | Library Settings | Planned | M1 |
| SET-002 | Reader Settings | Planned | M2 |
| SET-003 | Theme | Planned | M2 |
| SET-004 | OCR Settings | Future | M5 |
| SET-005 | AI Settings | Future | M6 |

---

# Non Goals

The following features are intentionally excluded from the current roadmap.

- Cloud Sync
- Multi User
- Online Account
- DRM Support
- Store / Marketplace
- Social Features

---

# Implementation Order

Foundation

↓

Library

↓

Reader

↓

Bookmarks

↓

Notes

↓

Search

↓

OCR

↓

Dictionary

↓

AI

Each milestone should be independently usable.

The application must remain functional after every milestone.