# Documentation Guide

This directory contains the product and technical specification for Book
Library. The repository contains the active Windows implementation; documents
remain authoritative for scope, decisions, and delivery status.

## Source-of-truth hierarchy

When documents disagree, use this order:

1. **Accepted ADRs** — binding technical decisions.
2. **Product requirements** — binding product scope and non-functional constraints.
3. **Feature catalog** — authoritative feature status and milestone assignment.
4. **System architecture and domain model** — implementation boundaries and invariants.
5. **Module specifications** — detailed behavior owned by one subsystem.
6. **Implementation plan and backlog** — sequencing and prioritization.
7. **Sprint documents** — the currently approved delivery slice.

An `Open Questions` section records undecided options only. It is not an approved implementation choice. Any unresolved question that affects the active sprint must be settled in an ADR or removed from the sprint.

## Recommended reading order

### Product

- [Product vision](00-overview/vision.md)
- [Core principles](00-overview/principles.md)
- [Product requirements](01-product/requirements.md)
- [Feature catalog](01-product/feature-catalog.md)
- [Core use cases](01-product/use-cases.md)
- [Glossary](01-product/glossary.md)

### Architecture

- [System architecture](02-architecture/architecture.md)
- [Domain model](02-architecture/domain-model.md)
- [SQLite foundation](02-architecture/sqlite-foundation.md)
- [PDFium spike](02-architecture/pdfium-spike.md)
- [Google Drive Desktop spike](02-architecture/google-drive-spike.md)
- [Architecture Decision Records](adr/README.md)

The system architecture owns dependency direction, module ownership, initial code structure, data ownership, technology boundaries, and runtime composition. Do not create separate architecture summaries that repeat those rules.

### Module specifications

- `03-library/` — initialization, scanning, discovery, thumbnails, and filesystem reconciliation.
- `04-reader/` — PDF and image-folder readers, bookmarks, and reading history.
- `05-notes/` — Markdown ownership, parsing, and Obsidian compatibility.
- `06-search/` — local search and rebuildable indexing.
- `07-ai/` — optional OCR, dictionary, AI assistant, and Anki capabilities.

### Delivery

- [Implementation plan](../planning/implementation-plan.md)
- [Prioritized backlog](../planning/backlog.md)
- [Sprint 01](../planning/sprint-01.md)
- [Sprint 02](../planning/sprint-02.md)
- [Sprint 03](../planning/sprint-03.md)
- [Sprint 04](../planning/sprint-04.md)
- [Sprint 05](../planning/sprint-05.md)

The implementation plan is the only roadmap document. The backlog decomposes milestones into slices, and the active sprint selects the work currently approved.

## Current project status

| Area | Status | Notes |
|---|---|---|
| Product scope | Defined | Core reader and knowledge scope documented. |
| Architecture baseline | Accepted | Desktop/offline/filesystem-first boundaries are fixed. |
| Detailed module designs | Draft | Remaining questions become ADRs only when they block scheduled work. |
| Application scaffold | In progress | M0–revised M2 Windows implementation and local quality gates pass; hosted CI and macOS Intel evidence remain. |
| Current delivery target | Sprint 05 | Windows missing-source recovery and offline Search MVP. |

## Documentation responsibilities

Update only the smallest authoritative set:

- product behavior changed → requirements and feature catalog;
- user workflow changed → core use cases and the owning module specification;
- technical decision introduced or reversed → ADR;
- module ownership, dependency direction, or code structure changed → system architecture;
- domain concepts or invariants changed → domain model;
- persistence changed → migrations, recovery notes, and affected ADR/specification;
- delivery scope changed → implementation plan, backlog, or active sprint;
- feature completed → feature catalog status.

Do not create separate roadmap, journey, flow, screen, technology-stack, or dependency-summary documents unless they contain durable information that cannot live in an existing authoritative file.

## Implementation-ready definition

A feature is ready for coding only when:

- it has a stable feature ID;
- its user outcome and acceptance criteria are explicit;
- required dependencies are available in the current milestone;
- architecture ownership is clear;
- blocking alternatives are resolved by ADR;
- error, cancellation, recovery, and data-ownership behavior are specified.

Documentation must describe the repository as it exists. Proposed capabilities stay marked planned or future until implementation and validation are complete.
