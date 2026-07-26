# PDFium Native Smoke

This standalone program validates dynamic PDFium loading and one-page rendering
without adding PDFium to the production application.

Windows x64 example:

```text
cargo run --manifest-path spikes/pdfium-smoke/Cargo.toml -- <pdfium-bin-directory> tests/fixtures/pdfium-smoke.pdf <output.png>
```

Use the pinned binary version and checksum recorded in
`docs/02-architecture/pdfium-spike.md`. The output is a rebuildable QA artifact
and must remain outside source-book folders.
