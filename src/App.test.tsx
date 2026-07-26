import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { App, StartupPanel } from "./App";

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
});
