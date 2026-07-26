# ADR-008: Define M1 library discovery and cache policies

- **Status:** Accepted
- **Date:** 2026-07-26

## Context

M1 requires deterministic identity, image-folder eligibility, case handling,
symlink containment, and thumbnail invalidation. Leaving these policies inside
scanner implementation would make catalog behavior platform-dependent and could
allow unsafe traversal or destructive rename guesses.

## Decision

### Identity and rename behavior

M1 uses the normalized relative path as stable catalog identity within one
library. A source fingerprint detects changes but is not identity.

When a path disappears it is marked `missing`. A newly observed path creates a
new book. M1 does not infer a rename from equal size, timestamps, or partial
fingerprints. Fingerprint-assisted relocation is deferred until it can handle
duplicates and false matches explicitly.

### Case handling

The domain `RelativePath` preserves spelling and case. Infrastructure stores:

- the exact normalized path for display and portable references;
- a platform comparison key for uniqueness.

Windows uses Unicode lowercase on the normalized path for its comparison key.
macOS uses the exact normalized path until real filesystem capability detection
is introduced. A collision produces a scan issue; the scanner does not merge or
overwrite competing entries. A Windows case-only rename updates the stored exact
path while retaining the existing book identity.

### Symlinks and root containment

M1 does not follow file or directory symlinks. The configured root is
canonicalized once for runtime access. Every opened candidate is canonicalized
and must remain beneath that authorized root. Escaping or unreadable
indirections produce per-entry issues.

### Image-folder eligibility

Supported page extensions are:

- `.jpg`;
- `.jpeg`;
- `.png`;
- `.webp`.

An image-folder book contains at least two readable, directly contained images.
Pages use deterministic, case-preserving natural filename ordering with a
relative-path tie-breaker.

Nested folders are evaluated independently; a parent never aggregates descendant
images as chapters. A mixed folder may contain unrelated files or PDF files and
still be an image-folder book when it has at least two direct images. Direct PDF
files are also discovered as their own books.

Hidden/system entries, temporary files, and application-owned artifact
directories are skipped. Unsupported ordinary files are ignored rather than
creating one issue per file.

### Thumbnails

Thumbnails are deterministic PNG files:

- maximum bounding box: 320 by 448 pixels;
- aspect ratio preserved;
- no upscaling;
- first readable image for image-folder books;
- first PDF page when the PDFium adapter is available;
- cache location under OS application data, never the library root;
- cache identity derived from book ID and source fingerprint;
- changed fingerprints invalidate and regenerate the thumbnail.

A failed thumbnail remains a cataloged book with a typed thumbnail error.

## Considered options

### Fingerprint-assisted rename in M1

Rejected because size/timestamp fingerprints are not unique and full hashing
would add expensive I/O before duplicate policy exists.

### Follow contained symlinks

Rejected for M1 because race-free containment differs across platforms and Drive
providers. Skipping is deterministic and safest.

### Aggregate nested images as chapters

Deferred because chapter boundaries and ordering require a product model not
present in M1.

### Store thumbnails beside books

Rejected by ADR-005 because it modifies and clutters user-owned source folders.

## Consequences

- rename appears as one missing record plus one new record in M1;
- category folders with fewer than two direct images are not books;
- symlinked libraries or books require a future explicit policy;
- thumbnail cache can be deleted and rebuilt without source loss;
- Windows uniqueness is case-insensitive without lowercasing domain values.

## Implementation constraints

- Test natural ordering, mixed folders, nested folders, collisions, idempotent
  rescans, missing transitions, and symlink skipping.
- Never persist an absolute book/page/cache identity.
- Never delete source books during reconciliation or repair.
- Record platform comparison keys as infrastructure data, not domain paths.

## Revisit when

Revisit after M1 when reliable relocation, chapter collections, additional image
formats, or contained symlink support becomes an approved product requirement.
