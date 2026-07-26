import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";

const navigation = ["Library", "Recent", "Notes", "Search", "Settings"];

export interface ApplicationStatus {
  databaseHealthy: boolean;
  libraryConfigured: boolean;
  platform: {
    os: string;
    architecture: string;
    supported: boolean;
  };
}

export interface DesktopError {
  code: string;
  message: string;
}

interface LibraryConfiguration {
  displayName: string;
}

interface ScanProgress {
  visitedEntries: number;
  discoveredBooks: number;
  currentRelativePath: string | null;
}

interface ScanSummary {
  discovered: number;
  added: number;
  updated: number;
  missing: number;
  issues: number;
  thumbnailsGenerated: number;
  thumbnailFailures: number;
  cancelled: boolean;
}

interface Book {
  id: string;
  title: string;
  kind: "pdf_file" | "image_folder";
  relativePath: string;
  status: "available" | "unavailable" | "missing" | "unsupported" | "error";
  pageCount: number | null;
  sizeBytes: number | null;
  modifiedAtMs: number | null;
  thumbnailDataUrl: string | null;
  thumbnailStatus: "pending" | "ready" | "error";
}

export type StartupState =
  | { kind: "loading" }
  | { kind: "healthy"; status: ApplicationStatus }
  | { kind: "error"; error: DesktopError };

function desktopError(error: unknown): DesktopError {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error
  ) {
    return error as DesktopError;
  }
  return {
    code: "operation_failed",
    message: "Book Library could not complete that operation.",
  };
}

export function App() {
  const [startup, setStartup] = useState<StartupState>({ kind: "loading" });
  const [configuration, setConfiguration] =
    useState<LibraryConfiguration | null>(null);
  const [books, setBooks] = useState<Book[]>([]);
  const [view, setView] = useState<"grid" | "list">("grid");
  const [scanProgress, setScanProgress] = useState<ScanProgress | null>(null);
  const [scanSummary, setScanSummary] = useState<ScanSummary | null>(null);
  const [operationError, setOperationError] = useState<DesktopError | null>(null);

  const loadBooks = useCallback(async () => {
    setBooks(await invoke<Book[]>("list_library_books"));
  }, []);

  useEffect(() => {
    void Promise.all([
      invoke<ApplicationStatus>("get_application_status"),
      invoke<LibraryConfiguration | null>("get_library_configuration"),
    ])
      .then(async ([status, configured]) => {
        setStartup({ kind: "healthy", status });
        setConfiguration(configured);
        if (configured) await loadBooks();
      })
      .catch((error: unknown) =>
        setStartup({ kind: "error", error: desktopError(error) }),
      );
  }, [loadBooks]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<ScanProgress>("library_scan_progressed", (event) => {
      setScanProgress(event.payload);
    }).then((stop) => {
      unlisten = stop;
    });
    return () => unlisten?.();
  }, []);

  const runScan = async (
    command: "initialize_library" | "rescan_library" | "repair_library",
  ) => {
    setOperationError(null);
    setScanSummary(null);
    setScanProgress({
      visitedEntries: 0,
      discoveredBooks: 0,
      currentRelativePath: null,
    });
    try {
      const summary = await invoke<ScanSummary>(command);
      setScanSummary(summary);
      await loadBooks();
    } catch (error) {
      setOperationError(desktopError(error));
    } finally {
      setScanProgress(null);
    }
  };

  const chooseLibrary = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected !== "string") return;
    try {
      const configured = await invoke<LibraryConfiguration>("configure_library", {
        selectedRoot: selected,
      });
      setConfiguration(configured);
      setStartup((current) =>
        current.kind === "healthy"
          ? {
              kind: "healthy",
              status: { ...current.status, libraryConfigured: true },
            }
          : current,
      );
      await runScan("initialize_library");
    } catch (error) {
      setOperationError(desktopError(error));
    }
  };

  const cancelScan = async () => {
    await invoke<boolean>("cancel_library_scan");
  };

  return (
    <main className="min-h-screen bg-stone-950 text-stone-100">
      <div className="mx-auto flex min-h-screen max-w-[1600px]">
        <aside className="w-60 shrink-0 border-r border-stone-800 px-5 py-7">
          <p className="text-xs font-semibold uppercase tracking-[0.28em] text-amber-400">
            Book Library
          </p>
          <nav className="mt-10 space-y-2" aria-label="Primary navigation">
            {navigation.map((item, index) => (
              <button
                className={`block w-full rounded-lg px-3 py-2 text-left text-sm ${
                  index === 0
                    ? "bg-stone-800 text-white"
                    : "text-stone-500"
                }`}
                disabled={index !== 0}
                key={item}
                type="button"
              >
                {item}
              </button>
            ))}
          </nav>
          {configuration && (
            <div className="mt-10 border-t border-stone-800 pt-5">
              <p className="text-xs uppercase tracking-wider text-stone-500">
                Current library
              </p>
              <p className="mt-2 truncate text-sm text-stone-300">
                {configuration.displayName}
              </p>
              <button
                className="mt-3 text-xs text-amber-400 hover:text-amber-300"
                onClick={() => void chooseLibrary()}
                type="button"
              >
                Change folder
              </button>
            </div>
          )}
        </aside>
        <section className="min-w-0 flex-1 px-7 py-8">
          <StartupPanel startup={startup} />
          {startup.kind === "healthy" && startup.status.platform.supported && (
            <>
              {!configuration ? (
                <SetupLibrary
                  error={operationError}
                  onChoose={() => void chooseLibrary()}
                />
              ) : (
                <LibraryWorkspace
                  books={books}
                  error={operationError}
                  onCancel={() => void cancelScan()}
                  onRepair={() => void runScan("repair_library")}
                  onRescan={() => void runScan("rescan_library")}
                  onViewChange={setView}
                  progress={scanProgress}
                  summary={scanSummary}
                  view={view}
                />
              )}
            </>
          )}
        </section>
      </div>
    </main>
  );
}

function SetupLibrary({
  error,
  onChoose,
}: {
  error: DesktopError | null;
  onChoose: () => void;
}) {
  return (
    <div className="mx-auto mt-28 max-w-xl">
      <p className="text-sm text-amber-400">Library setup</p>
      <h1 className="mt-3 text-4xl font-semibold tracking-tight">
        Choose the folder that already owns your books.
      </h1>
      <p className="mt-5 leading-7 text-stone-400">
        Book Library scans in place. It does not rename, move, delete, or rewrite
        source books.
      </p>
      <button
        className="mt-8 rounded-lg bg-amber-400 px-5 py-3 font-medium text-stone-950 hover:bg-amber-300"
        onClick={onChoose}
        type="button"
      >
        Choose library folder
      </button>
      {error && <ErrorPanel error={error} />}
    </div>
  );
}

function LibraryWorkspace({
  books,
  error,
  onCancel,
  onRepair,
  onRescan,
  onViewChange,
  progress,
  summary,
  view,
}: {
  books: Book[];
  error: DesktopError | null;
  onCancel: () => void;
  onRepair: () => void;
  onRescan: () => void;
  onViewChange: (view: "grid" | "list") => void;
  progress: ScanProgress | null;
  summary: ScanSummary | null;
  view: "grid" | "list";
}) {
  return (
    <>
      <header className="flex flex-wrap items-end justify-between gap-4 border-b border-stone-800 pb-6">
        <div>
          <p className="text-sm text-amber-400">Local catalog</p>
          <h1 className="mt-2 text-3xl font-semibold">
            {books.length} {books.length === 1 ? "book" : "books"}
          </h1>
        </div>
        <div className="flex gap-2">
          <button
            className="rounded-lg border border-stone-700 px-3 py-2 text-sm"
            onClick={onRepair}
            type="button"
          >
            Repair covers
          </button>
          <button
            className="rounded-lg bg-amber-400 px-4 py-2 text-sm font-medium text-stone-950"
            onClick={onRescan}
            type="button"
          >
            Rescan
          </button>
          <div className="ml-2 flex rounded-lg border border-stone-700 p-1">
            {(["grid", "list"] as const).map((choice) => (
              <button
                aria-pressed={view === choice}
                className={`rounded px-2 py-1 text-xs ${
                  view === choice ? "bg-stone-700" : "text-stone-400"
                }`}
                key={choice}
                onClick={() => onViewChange(choice)}
                type="button"
              >
                {choice}
              </button>
            ))}
          </div>
        </div>
      </header>

      {progress && (
        <div className="mt-5 rounded-xl border border-amber-900/60 bg-amber-950/20 p-4">
          <div className="flex items-center justify-between gap-4">
            <p className="text-sm text-amber-200">
              Scanned {progress.visitedEntries.toLocaleString()} entries · found{" "}
              {progress.discoveredBooks.toLocaleString()} books
            </p>
            <button
              className="text-sm text-amber-300 underline"
              onClick={onCancel}
              type="button"
            >
              Cancel
            </button>
          </div>
          {progress.currentRelativePath && (
            <p className="mt-2 truncate text-xs text-amber-400/70">
              {progress.currentRelativePath}
            </p>
          )}
        </div>
      )}
      {summary && <SummaryPanel summary={summary} />}
      {error && <ErrorPanel error={error} />}

      {books.length === 0 && !progress ? (
        <div className="mt-24 text-center text-stone-500">
          No supported books found yet. Try a rescan.
        </div>
      ) : (
        <div className={view === "grid" ? "book-grid mt-7" : "mt-7 space-y-2"}>
          {books.map((book) => (
            <BookCard book={book} key={book.id} view={view} />
          ))}
        </div>
      )}
    </>
  );
}

function BookCard({ book, view }: { book: Book; view: "grid" | "list" }) {
  const details = [
    book.kind === "pdf_file" ? "PDF" : "Images",
    book.pageCount ? `${book.pageCount} pages` : null,
    book.sizeBytes ? formatBytes(book.sizeBytes) : null,
  ]
    .filter(Boolean)
    .join(" · ");
  if (view === "list") {
    return (
      <article className="book-virtual-row flex items-center gap-4 rounded-lg border border-stone-800 bg-stone-900/50 p-3">
        <Cover book={book} compact />
        <div className="min-w-0 flex-1">
          <h2 className="truncate font-medium">{book.title}</h2>
          <p className="mt-1 truncate text-xs text-stone-500">
            {book.relativePath}
          </p>
        </div>
        <p className="text-xs text-stone-400">{details}</p>
      </article>
    );
  }
  return (
    <article className="book-virtual-card min-w-0">
      <Cover book={book} />
      <h2 className="mt-3 truncate font-medium">{book.title}</h2>
      <p className="mt-1 text-xs text-stone-500">{details}</p>
      {book.status !== "available" && (
        <p className="mt-2 text-xs text-red-300">{book.status}</p>
      )}
    </article>
  );
}

function Cover({ book, compact = false }: { book: Book; compact?: boolean }) {
  const size = compact ? "h-16 w-12" : "aspect-[5/7] w-full";
  return (
    <div
      className={`${size} shrink-0 overflow-hidden rounded-md border border-stone-800 bg-stone-900`}
    >
      {book.thumbnailDataUrl ? (
        <img
          alt=""
          className="h-full w-full object-cover"
          loading="lazy"
          src={book.thumbnailDataUrl}
        />
      ) : (
        <div className="flex h-full items-center justify-center p-2 text-center text-xs text-stone-600">
          {book.thumbnailStatus === "error" ? "Cover unavailable" : "Cover pending"}
        </div>
      )}
    </div>
  );
}

function SummaryPanel({ summary }: { summary: ScanSummary }) {
  return (
    <div className="mt-5 rounded-xl border border-stone-800 bg-stone-900/60 p-4 text-sm text-stone-300">
      {summary.cancelled ? "Scan cancelled." : "Scan complete."}{" "}
      {summary.discovered} discovered, {summary.added} added, {summary.updated}{" "}
      updated, {summary.missing} missing, {summary.issues} issues.
    </div>
  );
}

function ErrorPanel({ error }: { error: DesktopError }) {
  return (
    <div className="mt-5 rounded-xl border border-red-900 bg-red-950/30 p-4" role="alert">
      <p className="text-sm text-red-200">{error.message}</p>
      <p className="mt-2 text-xs text-red-400">{error.code}</p>
    </div>
  );
}

function formatBytes(bytes: number) {
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function StartupPanel({ startup }: { startup: StartupState }) {
  if (startup.kind === "loading") {
    return (
      <div className="mt-8 text-sm text-stone-400" role="status">
        Starting Book Library…
      </div>
    );
  }
  if (startup.kind === "error") return <ErrorPanel error={startup.error} />;
  if (!startup.status.platform.supported) {
    return (
      <div className="mt-8 rounded-xl border border-amber-800 bg-amber-950/20 p-5" role="alert">
        <p className="font-medium text-amber-200">Unsupported platform</p>
        <p className="mt-2 text-sm text-amber-300">
          This build targets Windows 11 x64 and macOS Intel x64.
        </p>
      </div>
    );
  }
  return null;
}
