# ADR-013: Treat cover repair as a missing-cover batch retry

- **Status:** Accepted
- **Date:** 2026-07-27

## Context

ADR-012 introduced a longer retry window for bulk cover repair, but selecting
targets by thumbnail error state made Repair behave differently from a batch
version of Force cover generation. A book can retain a usable last-known-good
cover while its latest attempt is in an error state; retrying that book is
unnecessary. Conversely, a cloud-backed book without a cover can be classified
as unavailable during discovery and still be a valid explicit retry target.

## Decision

Repair generates covers only for catalog books that have no usable current
cover. A usable current cover is a decodable app-data cover referenced by the
catalog, regardless of the latest thumbnail status.

Before generation, Repair restores a valid historical app-data cover whose
catalog link was lost. It also treats a missing or undecodable referenced cache
file as no cover. Remaining books without a cover are retried in bulk with the
same source eligibility and thirty-second per-book window as Force cover
generation: both available and cloud-unavailable sources may be attempted.

Repair reads its targets from the existing catalog and app-data cover cache. It
does not scan the library filesystem or reconcile book discovery; manual Rescan
remains the only command for that work.

Repair does not regenerate a usable cover merely because its thumbnail status
is pending or error. Force cover generation remains the explicit way to replace
the cover of one selected book.

Repair starts cover attempts sequentially. PDFium rendering is process-serialized,
so parallel workers would consume later books' retry windows while waiting for
the earlier render lock and could turn one slow attempt into a batch of false
timeouts.

## Consequences

- Repair is the batch equivalent of Force for books visibly lacking a cover.
- Repair no longer reports filesystem entries or rediscovered book counts.
- Healthy and last-known-good covers avoid unnecessary source hydration and
  rendering.
- Cloud-unavailable books without covers can enter the Repair generation queue.
- Missing or corrupt cache files are detected before target selection.
- Each target receives its own full retry window after the previous target
  finishes.
- Scan, source-safety, serialization, timeout, and replacement rules from
  ADR-012 remain unchanged.

## Implementation constraints

- source books remain read-only;
- a valid referenced cover must not be cleared or replaced by Repair;
- an unavailable source may be promoted to available only after generation
  succeeds;
- one failed target must not stop the remaining batch;
- recovered, generated, and failed counts remain separate.

## Revisit when

Revisit if Repair becomes a general cache verifier, if users can select bulk
regeneration policies, or if cover generation moves to persistent background
jobs.
