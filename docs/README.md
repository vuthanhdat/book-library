# Documentation Guide

This directory contains the product and technical specification for Book Library. The repository is currently documentation-first: application code has not been scaffolded yet.

## Source-of-truth hierarchy

When two documents disagree, use this order:

1. **Accepted ADRs** — binding technical decisions.
2. **Product requirements** — binding product scope and non-functional constraints.
3. **Feature catalog** — authoritative feature status and milestone assignment.
4. **System architecture and domain model** — implementation boundaries and invariants.
5. **Module specifications** — behavior for a specific subsystem.
6. **Implementation plan and backlog** — sequencing and prioritization.
7. **Sprint documents** — the currently approved delivery slice.
8. **Screen and journey documents** — UX guidance; they do not override domain or architecture rules.

An `Open Questions` section records undecided options only. It must not be interpreted as an approved implementation choice. Any unresolved question that affects the current sprint must be settled in an ADR before implementation.

## Recommended reading order

### 1. Product intent

- [Product vision](00-overview/vision.md)
- [Core principles](00-overview/principles.md)
- [Product roadmap](00-overview/roadmap.md)
- [Product requirements](01-product/requirements.md)
- [Feature catalog](01-product/feature-catalog.md)
- [Glossary](01-product/glossary.md)

### 2. Architecture baseline

- [System architecture](02-architecture/architecture.md)
- [Domain model](02-architecture/domain-model.md)
- [Module dependencies](02-architecture/module-dependency.md)
- [Technology stack](02-architecture/tech-stack.md)
- [Architecture Decision Records](adr/README.md)

### 3. Core behavior specifications

- `03-library/` — initialization, scanning, discovery, thumbnails, and filesystem reconciliation.
- `04-reader/` — PDF and image-folder readers, bookmarks, and reading history.
- `05-notes/` — Markdown ownership, parsing, and Obsidian compatibility.
- `06-search/` — local search and rebuildable indexing.
- `07-ai/` — optional OCR, dictionary, AI assistant, and Anki capabilities.

### 4. Delivery documents

- [Implementation plan](../planning/implementation-plan.md)
- [Prioritized backlog](../planning/backlog.md)
- [Sprint 01](../planning/sprint-01.md)

## Current project status

| Area | Status | Notes |
|---|---|---|
| Product scope | Defined | Core reader and knowledge scope documented. |
| Architecture baseline | Accepted | Desktop/offline/filesystem-first boundaries are fixed. |
| Detailed module designs | Draft | Useful implementation guidance; remaining open questions require ADRs when scheduled. |
| Application scaffold | Not started | No Tauri, React, Rust, SQLite migration, or CI source exists yet. |
| Current delivery target | Sprint 01 | Engineering foundation and risk-reduction spikes. |

## Documentation responsibilities

Every implementation pull request must update the smallest relevant set of documents:

- change product behavior → requirements and feature catalog;
- introduce or reverse a technical decision → ADR;
- change module ownership or dependency direction → architecture documents;
- change persistence → schema/migration documentation and recovery notes;
- change delivery scope → implementation plan, backlog, or active sprint;
- complete a feature → feature catalog status.

Avoid copying the same rule into several documents. Prefer a short link to the authoritative document.

## Definition of an implementation-ready feature

A feature is ready for coding only when:

- it has a stable feature or backlog identifier;
- its user outcome and acceptance criteria are explicit;
- required dependencies are available in the current milestone;
- architecture ownership is clear;
- relevant open questions are either non-blocking or resolved by ADR;
- error, cancellation, recovery, and data-ownership behavior are specified.

## Document maintenance

Use concise headings, tables, and diagrams where they clarify behavior. Remove stale alternatives after a decision is accepted, or retain them only in the ADR's considered-options section. Documentation is part of the implementation and must not describe capabilities that the repository does not yet contain as if they were complete.