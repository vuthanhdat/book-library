import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { App, StartupPanel, filterCatalogBooks, type Book } from "./App";

const books: Book[] = [
  {
    id: "1",
    title: "「私」が主語になる人生のつくり方 脳の自動操縦から抜け出す7つの講義",
    kind: "image_folder",
    relativePath: "日本語/私/pages",
    status: "available",
    pageCount: 56,
    sizeBytes: null,
    modifiedAtMs: null,
    thumbnailDataUrl: null,
    thumbnailStatus: "pending",
  },
  {
    id: "2",
    title: "Rust Systems",
    kind: "pdf_file",
    relativePath: "Programming/Rust Systems.pdf",
    status: "missing",
    pageCount: null,
    sizeBytes: null,
    modifiedAtMs: null,
    thumbnailDataUrl: null,
    thumbnailStatus: "error",
  },
];

describe("App", () => {
  it("renders startup loading and navigation without a fake catalog", () => {
    const markup = renderToStaticMarkup(<App />);

    expect(markup).toContain("Starting Book Library");
    expect(markup).toContain("Library");
    expect(markup).toContain("Settings");
    expect(markup).not.toContain("sample book");
  });

  it("does not overlay a healthy supported workspace", () => {
    const markup = renderToStaticMarkup(
      <StartupPanel
        startup={{
          kind: "healthy",
          status: {
            databaseHealthy: true,
            libraryConfigured: false,
            platform: { os: "windows", architecture: "x86_64", supported: true },
          },
        }}
      />,
    );

    expect(markup).toBe("");
  });

  it("renders typed startup and unsupported-platform failures", () => {
    const startupError = renderToStaticMarkup(
      <StartupPanel
        startup={{
          kind: "error",
          error: { code: "database_unavailable", message: "Database unavailable" },
        }}
      />,
    );
    const unsupported = renderToStaticMarkup(
      <StartupPanel
        startup={{
          kind: "healthy",
          status: {
            databaseHealthy: true,
            libraryConfigured: false,
            platform: { os: "linux", architecture: "x86_64", supported: false },
          },
        }}
      />,
    );

    expect(startupError).toContain("database_unavailable");
    expect(unsupported).toContain("Unsupported platform");
  });

  it("filters the catalog live with Unicode and AND-combined terms", () => {
    expect(filterCatalogBooks(books, "私 脳")).toEqual([books[0]]);
    expect(filterCatalogBooks(books, "rust pdf missing")).toEqual([books[1]]);
    expect(filterCatalogBooks(books, "rust available")).toEqual([]);
    expect(filterCatalogBooks(books, "   ")).toBe(books);
  });
});
