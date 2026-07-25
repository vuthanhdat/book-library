# ADR-007 — Support Windows 11 x64 and macOS Intel x64

- **Status:** Accepted
- **Date:** 2026-07-25

## Context

Book Library is intended for the same user on both a Windows 11 computer and an Intel-based Mac. Treating macOS as a later port would allow Windows-only path, filesystem, native-library, and packaging assumptions to enter the core implementation during M0–M2.

The chosen Tauri, React, Rust, SQLite, and adapter-based architecture is intended to support one shared codebase. Platform differences still exist around filesystem behavior, application-data locations, native PDFium binaries, Google Drive Desktop placeholders, packaging, signing, and installer behavior.

## Decision

Book Library has two required desktop platforms from M0:

- Windows 11 x64 (`x86_64-pc-windows-msvc`);
- macOS Intel x64 (`x86_64-apple-darwin`).

The project will maintain one Tauri/React/Rust codebase. Domain and application behavior must remain platform-independent. Platform-specific behavior belongs in infrastructure adapters or the desktop composition boundary.

A milestone is not complete until its applicable core outcome has been validated on both supported platforms. Automated tests and builds should run on both platforms where runner support is available. A smoke test on a real Intel Mac is required before closing milestones that introduce or change desktop, filesystem, SQLite, reader, or packaging behavior.

Persisted content references remain normalized relative paths using `/`. Absolute library and notes roots remain machine-local settings. The domain must not force lowercase path identity or otherwise encode Windows-only case behavior; filesystem-specific comparison, Unicode, symlink, and availability handling belongs at the infrastructure boundary.

M0 must prove development builds, application-data resolution, SQLite migrations, typed Tauri communication, and core tests on both platforms. PDFium, Google Drive Desktop, and SQLite spikes must record findings for both Windows x64 and macOS Intel x64.

Public release packaging, Windows installer validation, macOS code signing, notarization, and distributable `.app`/`.dmg` artifacts are release concerns for M5. Local development and personal builds on the owner's Intel Mac are required earlier.

Apple Silicon and universal macOS binaries are not committed release targets. They may be added later when there is a real user or distribution need.

## Considered options

### Windows-only first

Rejected because the owner needs to use the application on both computers and because a late macOS port would expose platform assumptions only after the filesystem and reader modules were already established.

### Windows primary, macOS best-effort later

Rejected because “best effort” does not create a test or milestone gate and therefore does not prevent regressions.

### Two required platforms from M0

Accepted because it preserves one architecture, exposes native integration risks early, and matches actual usage.

## Consequences

- CI, local validation instructions, fixtures, and release checklists must distinguish Windows x64 and macOS Intel x64.
- Path handling cannot rely on drive letters, backslashes, case-insensitive comparison, or Windows-only Unicode behavior.
- Filesystem and Google Drive behavior must be tested separately on both operating systems.
- PDFium bindings and native binaries must support and be packaged for both targets before the PDF reader is considered production-ready.
- Some platform-specific implementation is expected, but it must remain behind shared ports.
- M0 and later milestones require more validation than a Windows-only application, reducing the risk of a costly later port.

## Implementation constraints

- Do not create separate product repositories or duplicate domain/application implementations per operating system.
- Do not persist platform-specific absolute source paths as content identity.
- Keep conditional compilation narrow and primarily inside `infrastructure` or `desktop`.
- Tests for `RelativePath` must cover Windows separators, POSIX-rooted paths, Unicode, traversal, and exact normalized representation.
- CI configuration must not claim Intel-macOS coverage unless the selected runner or documented environment actually executes the required target.
- A release artifact must be built on a compatible macOS toolchain before it is presented as a macOS Intel release.

## Revisit when

Revisit this decision when:

- Apple Silicon becomes a required user platform;
- an upstream native dependency drops Intel macOS support;
- maintaining both required platforms would block core development for a sustained period;
- a move away from Tauri or PDFium materially changes portability constraints.