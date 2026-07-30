# ADR-016: Add a bounded integrated Study Reader

- **Status:** Accepted
- **Date:** 2026-07-30
- **Supersedes:** [ADR-009](ADR-009-external-reading-and-live-catalog-search.md)

## Context

ADR-009 kept Book Library as a catalog and launcher because external readers
already provide strong general reading experiences. M6 introduced offline
Japanese OCR and dictionary lookup, but placing those tools in a separate Study
workspace forces the user to leave the current page and manually transfer
context. External applications cannot provide an app-local selectable OCR layer,
Vietnamese dictionary results, or learning-draft actions beside the source page.

The maintainer explicitly requested dictionary lookup while a book is open.
This satisfies ADR-009's revisit condition for a reader experience that an
external application cannot provide.

## Decision

Add a bounded `Read & Study` reader while retaining `Open externally` as a
separate action.

The Study Reader:

- opens one cataloged PDF or image-folder book by book ID;
- resolves and canonicalizes each requested page beneath the configured library
  root through application and infrastructure ports;
- renders one page at a time to bounded app-data/cache output;
- supports previous, next, direct page requests, and presentation-only zoom;
- shows a persistent adjacent offline Japanese-to-Vietnamese dictionary;
- reuses saved OCR text for the current page;
- runs OCR only after an explicit `OCR this page` action;
- immediately sends a bounded selection from the OCR transcript to the existing
  dictionary lookup use case;
- never modifies the source PDF or image files.

This decision does not add reading progress, bookmarks, annotations,
continuous-scroll caching, or automatic whole-book OCR. Those remain separate
features and require their own acceptance criteria.

## Considered options

### Keep study tools separate from external readers

Rejected because selecting text, maintaining page context, and creating learning
drafts require repeated manual copy/paste and window switching.

### Replace external opening with the integrated reader

Rejected. External applications remain useful for general reading, printing,
native text extraction, and workflows outside Japanese study.

### OCR every page when a book opens

Rejected because it would violate offline optional-module isolation, hydrate
sources implicitly, and create an unbounded background workload.

## Consequences

- Book Library becomes both a catalog launcher and a bounded study reader.
- PDFium page rendering is used by an interactive application use case as well
  as thumbnail and OCR materialization.
- React owns reader layout, zoom state, selection handling, and transient page
  navigation; source authorization and rendering remain outside React.
- The current page image and OCR text remain derived, rebuildable app data.
- `Open externally` remains available from book and reader actions.
- Cross-platform completion still requires Windows 11 x64 and real macOS Intel
  x64 reader smoke evidence.

## Implementation constraints

- The desktop boundary accepts a book ID and zero-based page index, never an
  arbitrary source path.
- The application use case obtains page identity from the catalog repository.
- Infrastructure rechecks canonical root containment before reading a page.
- At most one requested page is returned to the presentation at a time.
- Page rendering and OCR failures use typed, user-safe errors and do not close
  the catalog.
- Normal scanning and ordinary book opening never start OCR.
- Keyboard navigation ignores focused text inputs and respects page bounds.
- Frontend tests cover the two-column layout, navigation controls, OCR action,
  and dictionary result area.

## Revisit when

Revisit the bounded reader if representative use requires continuous scrolling,
large prefetch caches, durable reading progress, bookmarks, annotations, native
PDF text layers, or additional book formats.
