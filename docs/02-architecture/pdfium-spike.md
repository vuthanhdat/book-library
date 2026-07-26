# PDFium Spike

## Status

- **Windows 11 x64:** native load and one-page render passed
- **macOS Intel x64:** dependency and packaging path researched; real render pending
- **Production reader:** out of scope for Sprint 01

## Decision for the reader milestone

Use `pdfium-render` `0.9.3` as the Rust adapter candidate and dynamically load a
pinned PDFium native binary. Keep the adapter behind an application-owned reader
port so the domain and application layers never depend on PDFium.

The evaluated binary is the bblanchon `chromium/7961` Windows x64 build:

```text
PDFium version: 152.0.7961.0
Archive: pdfium-win-x64.tgz
SHA-256: 88276459349B291C41F10422DAD0210F007C04D919C8FA56472B6B7C6406ADF4
```

The repository contains a standalone smoke program under `spikes/pdfium-smoke`
and a one-page fixture under `tests/fixtures`. The Windows smoke loaded
`pdfium.dll`, rendered page zero through `pdfium-render`, saved a PNG, and passed
visual inspection without clipping or missing content.

## Packaging proposal

### Windows x64

- place the pinned `pdfium.dll` next to the packaged executable;
- resolve it from the application resource/runtime directory, never from the
  source-book directory or an arbitrary `PATH` entry;
- record and verify the binary checksum during dependency preparation;
- include upstream license and applicable third-party notices in distribution.

### macOS Intel x64

- use the matching `pdfium-mac-x64.tgz` release;
- place `libpdfium.dylib` in the application bundle's `Contents/Frameworks`;
- set an application-relative runtime lookup path and verify it on a real Intel Mac;
- include the dylib in code-signing and notarization inputs;
- do not claim support until the same fixture renders from a packaged `.app`.

The bblanchon binary project publishes both Windows x64 and macOS x64 archives.
Its packaging repository is MIT-licensed, while PDFium and bundled third-party
code carry their own notices. Distribution must ship the PDFium license and the
notices present in the pinned archive rather than treating the wrapper's license
as the only obligation.

## Page transfer proposal

PDFium rendering stays in Rust. The adapter returns bounded page bitmaps plus
dimensions and render identity. For the first reader:

- encode rendered pages as PNG or WebP in a bounded cache outside source folders;
- expose bytes through a Tauri custom protocol or binary response;
- avoid JSON arrays or base64 for full pages;
- cancel stale renders and cap concurrent pages;
- never load every page of a large book into WebView memory.

## Sources

- [pdfium-render project](https://github.com/ajrcarey/pdfium-render)
- [PDFium binary distributions](https://github.com/bblanchon/pdfium-binaries)
- [PDFium upstream license](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/LICENSE)

## Remaining gate

Run the committed smoke program with the pinned macOS x64 binary on the owner's
real Intel Mac, then repeat from the intended application-bundle layout.
