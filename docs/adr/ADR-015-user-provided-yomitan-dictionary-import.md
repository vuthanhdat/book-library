# ADR-015: Import user-provided Yomitan dictionaries without bundling their data

- **Status:** Accepted
- **Date:** 2026-07-30

## Context

ADR-014 established a small original Japanese–Vietnamese starter dictionary and
a bounded TSV importer. The starter proves the workflow but is not useful as a
comprehensive dictionary. Existing Japanese dictionary tools commonly exchange
Yomitan-compatible ZIP packages, including community conversions containing
Vietnamese meanings.

The application cannot assume that a community package or its upstream database
can be redistributed. Copying such data into the source tree or installer would
create provenance and licensing risk.

## Decision

Accept an explicitly selected, user-provided Yomitan term dictionary ZIP in
addition to the existing TSV format:

- read `index.json` and root-level `term_bank_*.json` members directly from the
  archive without extracting files;
- derive package name and revision from `index.json`;
- normalize string and supported structured-content glossaries into plain text;
- skip and report entries whose package contains no usable textual definition;
- preserve the user-declared license or provenance label with the imported
  package;
- validate compressed size, archive member count, expanded size, bank size,
  entry count, field sizes, UTF-8 JSON, and required metadata before writing;
- commit package metadata and entries in one SQLite transaction;
- do not bundle, download, update, or grant redistribution rights for the
  user-provided dictionary.

Lookup candidate selection is performed in SQLite against the current query so
a large imported package is not copied wholesale into application memory for
every lookup.

## Considered options

### Bundle a community Mazii-derived package

Rejected because the public converter does not establish redistribution rights
for the extracted upstream application database.

### Add a Mazii-specific database reader

Rejected. A standard Yomitan import boundary is reusable and avoids coupling the
application to another product's private database schema.

### Extract ZIP members into application data

Rejected. Streaming members directly avoids archive path traversal and leaves
no partially extracted package to recover.

## Consequences

- Users can install a substantially larger offline Japanese–Vietnamese
  dictionary when they already have a lawful Yomitan package.
- Package quality, accuracy, and redistribution rights remain the user's
  responsibility.
- Unsupported glossary media or markup is not executed; supported textual
  content is stored and rendered as plain text.
- Importing a large package can take time. Empty-definition entries are counted
  and reported, while a package-level failure leaves no partial installation.

## Implementation constraints

- ZIP imports are explicit and local; no network access is added.
- Archive members are never written into a source-book folder.
- Only root `term_bank_<number>.json` members are treated as term banks.
- Import limits and transactional behavior require infrastructure tests.
- macOS Intel validation remains required before M6 is complete.

## Revisit when

Add a separate decision before bundling any comprehensive dictionary, adding
automatic downloads or updates, or executing rich glossary content.
