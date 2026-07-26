import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  useCallback,
  useDeferredValue,
  useEffect,
  useMemo,
  useState,
} from "react";

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

export type ActiveScan = "initial" | "rescan" | "repair";

export function scanButtonLabels(activeScan: ActiveScan | null) {
  return {
    repair:
      activeScan === "repair" ? "Repair running…" : "Repair covers",
    rescan: activeScan === "rescan" ? "Rescanning…" : "Rescan",
  };
}

interface UpdatedBookTitle {
  title: string;
}

interface LinkedBookNote {
  id: string;
  title: string;
}

export interface BookDetail extends Book {
  readingStatus: "unread" | "reading" | "read";
  tags: string[];
  notes: LinkedBookNote[];
}

interface NotesConfiguration {
  displayName: string;
}

export interface NoteListItem {
  id: string;
  title: string;
  relativePath: string;
  status: "available" | "missing" | "error";
  bookId: string | null;
  bookTitle: string | null;
  modifiedAtMs: number | null;
}

interface NoteBacklink {
  id: string;
  title: string;
  relativePath: string;
}

interface NoteDetail {
  id: string;
  title: string;
  relativePath: string;
  body: string;
  bookId: string | null;
  bookTitle: string | null;
  backlinks: NoteBacklink[];
}

interface NotesRefreshSummary {
  discovered: number;
  added: number;
  updated: number;
  missing: number;
  issues: number;
}

interface SearchResult {
  sourceKind: "book" | "note";
  sourceId: string;
  scope: "books" | "notes" | "tags" | "headings";
  title: string;
  snippet: string;
  relativePath: string;
  status: string;
  rank: number;
}

interface SearchDiagnostics {
  documents: number;
  failedJobs: number;
  lastRebuildAt: string | null;
}

interface SearchRebuildSummary {
  indexed: number;
  failed: number;
}

export interface Book {
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

export type Theme = "dark" | "light";

export function resolveTheme(stored: string | null): Theme {
  return stored === "light" ? "light" : "dark";
}

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
  const [theme, setTheme] = useState<Theme>(() =>
    typeof window === "undefined"
      ? "dark"
      : resolveTheme(window.localStorage.getItem("book-library-theme")),
  );
  const [startup, setStartup] = useState<StartupState>({ kind: "loading" });
  const [configuration, setConfiguration] =
    useState<LibraryConfiguration | null>(null);
  const [books, setBooks] = useState<Book[]>([]);
  const [view, setView] = useState<"grid" | "list">("grid");
  const [scanProgress, setScanProgress] = useState<ScanProgress | null>(null);
  const [activeScan, setActiveScan] = useState<ActiveScan | null>(null);
  const [scanSummary, setScanSummary] = useState<ScanSummary | null>(null);
  const [operationError, setOperationError] = useState<DesktopError | null>(null);
  const [openingBookId, setOpeningBookId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [editingBook, setEditingBook] = useState<Book | null>(null);
  const [editError, setEditError] = useState<DesktopError | null>(null);
  const [isSavingTitle, setIsSavingTitle] = useState(false);
  const [selectedBookDetail, setSelectedBookDetail] =
    useState<BookDetail | null>(null);
  const [bookDetailBusy, setBookDetailBusy] = useState(false);
  const [bookDetailError, setBookDetailError] =
    useState<DesktopError | null>(null);
  const [activeSection, setActiveSection] = useState<
    "Library" | "Notes" | "Search"
  >("Library");
  const [notesConfiguration, setNotesConfiguration] =
    useState<NotesConfiguration | null>(null);
  const [notes, setNotes] = useState<NoteListItem[]>([]);
  const [selectedNote, setSelectedNote] = useState<NoteDetail | null>(null);
  const [noteDraft, setNoteDraft] = useState("");
  const [notesBusy, setNotesBusy] = useState(false);
  const [notesError, setNotesError] = useState<DesktopError | null>(null);
  const [notesSummary, setNotesSummary] =
    useState<NotesRefreshSummary | null>(null);
  const [globalQuery, setGlobalQuery] = useState("");
  const [globalScope, setGlobalScope] = useState("all");
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [searchDiagnostics, setSearchDiagnostics] =
    useState<SearchDiagnostics | null>(null);
  const [globalSearchBusy, setGlobalSearchBusy] = useState(false);
  const [globalSearchError, setGlobalSearchError] =
    useState<DesktopError | null>(null);
  const deferredSearchQuery = useDeferredValue(searchQuery);
  const visibleBooks = useMemo(
    () => filterCatalogBooks(books, deferredSearchQuery),
    [books, deferredSearchQuery],
  );

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
    window.localStorage.setItem("book-library-theme", theme);
  }, [theme]);

  const loadBooks = useCallback(async () => {
    setBooks(await invoke<Book[]>("list_library_books"));
  }, []);

  const loadNotes = useCallback(async () => {
    setNotes(await invoke<NoteListItem[]>("list_notes"));
  }, []);

  const readNote = useCallback(async (noteId: string) => {
    setNotesError(null);
    try {
      const detail = await invoke<NoteDetail>("read_note", { noteId });
      setSelectedNote(detail);
      setNoteDraft(detail.body);
    } catch (error) {
      setNotesError(desktopError(error));
    }
  }, []);

  useEffect(() => {
    void Promise.all([
      invoke<ApplicationStatus>("get_application_status"),
      invoke<LibraryConfiguration | null>("get_library_configuration"),
      invoke<NotesConfiguration | null>("get_notes_configuration"),
    ])
      .then(async ([status, configured, configuredNotes]) => {
        setStartup({ kind: "healthy", status });
        setConfiguration(configured);
        setNotesConfiguration(configuredNotes);
        if (configured) await loadBooks();
        if (configuredNotes) await loadNotes();
      })
      .catch((error: unknown) =>
        setStartup({ kind: "error", error: desktopError(error) }),
      );
  }, [loadBooks, loadNotes]);

  useEffect(() => {
    if (activeSection !== "Search") return;
    const timer = window.setTimeout(() => {
      const query = globalQuery.trim();
      if (!query) {
        setSearchResults([]);
        return;
      }
      setGlobalSearchBusy(true);
      setGlobalSearchError(null);
      void invoke<SearchResult[]>("search_library", {
        query,
        scope: globalScope,
      })
        .then(setSearchResults)
        .catch((error) => setGlobalSearchError(desktopError(error)))
        .finally(() => setGlobalSearchBusy(false));
    }, 180);
    return () => window.clearTimeout(timer);
  }, [activeSection, globalQuery, globalScope]);

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
    const scanKind: ActiveScan =
      command === "repair_library"
        ? "repair"
        : command === "rescan_library"
          ? "rescan"
          : "initial";
    setActiveScan(scanKind);
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
      setActiveScan(null);
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

  const openBookLocation = async (book: Book) => {
    setOperationError(null);
    setOpeningBookId(book.id);
    try {
      await invoke<void>("open_book_location", { bookId: book.id });
    } catch (error) {
      setOperationError(desktopError(error));
    } finally {
      setOpeningBookId(null);
    }
  };

  const relinkBook = async (book: Book) => {
    const selected = await open(
      book.kind === "pdf_file"
        ? {
            directory: false,
            multiple: false,
            filters: [{ name: "PDF books", extensions: ["pdf"] }],
          }
        : { directory: true, multiple: false },
    );
    if (typeof selected !== "string") return;
    setOperationError(null);
    try {
      await invoke<string>("relink_missing_book", {
        bookId: book.id,
        selectedPath: selected,
      });
      await loadBooks();
    } catch (error) {
      setOperationError(desktopError(error));
    }
  };

  const startEditingBook = (book: Book) => {
    setEditError(null);
    setEditingBook(book);
  };

  const saveBookTitle = async (title: string) => {
    if (!editingBook) return;
    setEditError(null);
    setIsSavingTitle(true);
    try {
      const updated = await invoke<UpdatedBookTitle>("update_book_display_title", {
        bookId: editingBook.id,
        title,
      });
      setBooks((current) =>
        current.map((book) =>
          book.id === editingBook.id ? { ...book, title: updated.title } : book,
        ),
      );
      setSelectedBookDetail((current) =>
        current?.id === editingBook.id
          ? { ...current, title: updated.title }
          : current,
      );
      setEditingBook(null);
    } catch (error) {
      setEditError(desktopError(error));
    } finally {
      setIsSavingTitle(false);
    }
  };

  const openBookDetail = async (bookId: string) => {
    setBookDetailBusy(true);
    setBookDetailError(null);
    try {
      setSelectedBookDetail(
        await invoke<BookDetail>("get_book_detail", { bookId }),
      );
    } catch (error) {
      setBookDetailError(desktopError(error));
    } finally {
      setBookDetailBusy(false);
    }
  };

  const updateBookDetail = async (
    readingStatus: BookDetail["readingStatus"],
    tags: string[],
  ) => {
    if (!selectedBookDetail) return;
    setBookDetailBusy(true);
    setBookDetailError(null);
    try {
      setSelectedBookDetail(
        await invoke<BookDetail>("update_book_detail", {
          bookId: selectedBookDetail.id,
          readingStatus,
          tags,
        }),
      );
    } catch (error) {
      setBookDetailError(desktopError(error));
    } finally {
      setBookDetailBusy(false);
    }
  };

  const forceBookCover = async () => {
    if (!selectedBookDetail) return;
    setBookDetailBusy(true);
    setBookDetailError(null);
    try {
      const detail = await invoke<BookDetail>("force_book_cover", {
        bookId: selectedBookDetail.id,
      });
      setSelectedBookDetail(detail);
      await loadBooks();
    } catch (error) {
      setBookDetailError(desktopError(error));
    } finally {
      setBookDetailBusy(false);
    }
  };

  const chooseNotesRoot = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected !== "string") return;
    setNotesBusy(true);
    setNotesError(null);
    try {
      const configured = await invoke<NotesConfiguration>(
        "configure_notes_root",
        { selectedRoot: selected },
      );
      setNotesConfiguration(configured);
      setNotesSummary(await invoke<NotesRefreshSummary>("refresh_notes"));
      await loadNotes();
      setSelectedNote(null);
      setNoteDraft("");
    } catch (error) {
      setNotesError(desktopError(error));
    } finally {
      setNotesBusy(false);
    }
  };

  const refreshNotes = async () => {
    setNotesBusy(true);
    setNotesError(null);
    try {
      setNotesSummary(await invoke<NotesRefreshSummary>("refresh_notes"));
      await loadNotes();
      if (selectedNote) await readNote(selectedNote.id);
    } catch (error) {
      setNotesError(desktopError(error));
    } finally {
      setNotesBusy(false);
    }
  };

  const createNote = async (title: string, bookId: string | null) => {
    setNotesBusy(true);
    setNotesError(null);
    try {
      const detail = await invoke<NoteDetail>("create_note", { title, bookId });
      await loadNotes();
      setSelectedNote(detail);
      setNoteDraft(detail.body);
    } catch (error) {
      setNotesError(desktopError(error));
    } finally {
      setNotesBusy(false);
    }
  };

  const saveNote = async () => {
    if (!selectedNote) return;
    setNotesBusy(true);
    setNotesError(null);
    try {
      const detail = await invoke<NoteDetail>("save_note", {
        noteId: selectedNote.id,
        body: noteDraft,
      });
      setSelectedNote(detail);
      setNoteDraft(detail.body);
      await loadNotes();
    } catch (error) {
      setNotesError(desktopError(error));
    } finally {
      setNotesBusy(false);
    }
  };

  const rebuildSearch = async () => {
    setGlobalSearchBusy(true);
    setGlobalSearchError(null);
    try {
      const rebuilt = await invoke<SearchRebuildSummary>("rebuild_search_index");
      setSearchDiagnostics(
        await invoke<SearchDiagnostics>("get_search_diagnostics"),
      );
      if (globalQuery.trim()) {
        setSearchResults(
          await invoke<SearchResult[]>("search_library", {
            query: globalQuery.trim(),
            scope: globalScope,
          }),
        );
      }
      if (rebuilt.failed > 0) {
        setGlobalSearchError({
          code: "search_rebuild_partial",
          message: `${rebuilt.failed} documents could not be indexed.`,
        });
      }
    } catch (error) {
      setGlobalSearchError(desktopError(error));
    } finally {
      setGlobalSearchBusy(false);
    }
  };

  return (
    <main className="min-h-screen bg-stone-950 text-stone-100">
      <div className="mx-auto min-h-screen max-w-[1920px]">
        <header className="sticky top-0 z-40 flex min-h-16 flex-wrap items-center gap-x-6 gap-y-2 border-b border-stone-800 bg-stone-950/95 px-5 py-3 backdrop-blur md:flex-nowrap md:px-7 md:py-0">
          <p className="text-xs font-semibold uppercase tracking-[0.28em] text-amber-400">
            Book Library
          </p>
          <nav className="order-3 flex w-full min-w-0 items-center justify-between gap-0 md:order-none md:w-auto md:flex-1 md:justify-start md:gap-1" aria-label="Primary navigation">
            {navigation.map((item) => {
              const enabled =
                item === "Library" || item === "Notes" || item === "Search";
              return (
              <button
                className={`shrink-0 rounded-lg px-2 py-2 text-sm md:px-3 ${
                  item === activeSection
                    ? "bg-stone-800 text-white"
                    : enabled
                      ? "text-stone-300 hover:bg-stone-900"
                      : "text-stone-600"
                }`}
                disabled={!enabled}
                key={item}
                onClick={() =>
                  enabled &&
                  setActiveSection(item as "Library" | "Notes" | "Search")
                }
                type="button"
              >
                {item}
              </button>
              );
            })}
          </nav>
          <button
            aria-label={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
            className="shrink-0 rounded-lg border border-stone-700 px-3 py-2 text-xs text-stone-300 hover:bg-stone-900 hover:text-stone-100"
            onClick={() =>
              setTheme((current) => (current === "dark" ? "light" : "dark"))
            }
            type="button"
          >
            {theme === "dark" ? "Light theme" : "Dark theme"}
          </button>
          {configuration && (
            <div className="hidden min-w-0 items-center gap-3 border-l border-stone-800 pl-5 lg:flex">
              <p className="max-w-44 truncate text-xs text-stone-400" title={configuration.displayName}>
                {configuration.displayName}
              </p>
              <button
                className="shrink-0 text-xs text-amber-400 hover:text-amber-300"
                onClick={() => void chooseLibrary()}
                type="button"
              >
                Change folder
              </button>
            </div>
          )}
        </header>
        <section className="min-w-0 px-5 py-7 md:px-7">
          <StartupPanel startup={startup} />
          {startup.kind === "healthy" && startup.status.platform.supported && (
            <>
              {activeSection === "Search" ? (
                <GlobalSearchWorkspace
                  busy={globalSearchBusy}
                  diagnostics={searchDiagnostics}
                  error={globalSearchError}
                  onOpenResult={(result) => {
                    if (result.sourceKind === "book") {
                      setActiveSection("Library");
                      void openBookDetail(result.sourceId);
                    } else {
                      setActiveSection("Notes");
                      void readNote(result.sourceId);
                    }
                  }}
                  onQueryChange={setGlobalQuery}
                  onRebuild={() => void rebuildSearch()}
                  onScopeChange={setGlobalScope}
                  query={globalQuery}
                  results={searchResults}
                  scope={globalScope}
                />
              ) : activeSection === "Notes" ? (
                <NotesWorkspace
                  books={books}
                  busy={notesBusy}
                  configuration={notesConfiguration}
                  draft={noteDraft}
                  error={notesError}
                  notes={notes}
                  onChooseRoot={() => void chooseNotesRoot()}
                  onCreate={(title, bookId) => void createNote(title, bookId)}
                  onDraftChange={setNoteDraft}
                  onOpenExternal={() =>
                    selectedNote &&
                    void invoke("open_note_external", {
                      noteId: selectedNote.id,
                    }).catch((error) => setNotesError(desktopError(error)))
                  }
                  onOpenRoot={() =>
                    void invoke("open_notes_root").catch((error) =>
                      setNotesError(desktopError(error)),
                    )
                  }
                  onRefresh={() => void refreshNotes()}
                  onSave={() => void saveNote()}
                  onSelect={(noteId) => void readNote(noteId)}
                  selectedNote={selectedNote}
                  summary={notesSummary}
                />
              ) : !configuration ? (
                <SetupLibrary
                  error={operationError}
                  onChoose={() => void chooseLibrary()}
                />
              ) : selectedBookDetail ? (
                <BookDetailPage
                  busy={bookDetailBusy}
                  detail={selectedBookDetail}
                  error={bookDetailError}
                  onBack={() => setSelectedBookDetail(null)}
                  onEditTitle={() =>
                    startEditingBook(
                      books.find((book) => book.id === selectedBookDetail.id) ??
                        selectedBookDetail,
                    )
                  }
                  onForceCover={() => void forceBookCover()}
                  onNewNote={() => {
                    void createNote(
                      `Notes for ${selectedBookDetail.title}`,
                      selectedBookDetail.id,
                    );
                    setActiveSection("Notes");
                  }}
                  onOpenFolder={() =>
                    void openBookLocation(selectedBookDetail)
                  }
                  onOpenNote={(noteId) => {
                    setActiveSection("Notes");
                    void readNote(noteId);
                  }}
                  onSave={(readingStatus, tags) =>
                    void updateBookDetail(readingStatus, tags)
                  }
                />
              ) : (
                <LibraryWorkspace
                  activeScan={activeScan}
                  books={visibleBooks}
                  error={operationError}
                  onCancel={() => void cancelScan()}
                  onEditBook={startEditingBook}
                  onOpenDetail={(book) => void openBookDetail(book.id)}
                  onOpenBook={(book) => void openBookLocation(book)}
                  onRepair={() => void runScan("repair_library")}
                  onRelinkBook={(book) => void relinkBook(book)}
                  onRescan={() => void runScan("rescan_library")}
                  onSearchChange={setSearchQuery}
                  onViewChange={setView}
                  openingBookId={openingBookId}
                  progress={scanProgress}
                  searchQuery={searchQuery}
                  summary={scanSummary}
                  totalBooks={books.length}
                  view={view}
                />
              )}
            </>
          )}
        </section>
      </div>
      {editingBook && (
        <EditBookDialog
          book={editingBook}
          error={editError}
          isSaving={isSavingTitle}
          key={editingBook.id}
          onCancel={() => setEditingBook(null)}
          onSave={(title) => void saveBookTitle(title)}
        />
      )}
    </main>
  );
}

export function GlobalSearchWorkspace({
  busy,
  diagnostics,
  error,
  onOpenResult,
  onQueryChange,
  onRebuild,
  onScopeChange,
  query,
  results,
  scope,
}: {
  busy: boolean;
  diagnostics: SearchDiagnostics | null;
  error: DesktopError | null;
  onOpenResult: (result: SearchResult) => void;
  onQueryChange: (query: string) => void;
  onRebuild: () => void;
  onScopeChange: (scope: string) => void;
  query: string;
  results: SearchResult[];
  scope: string;
}) {
  return (
    <>
      <header className="flex flex-wrap items-end justify-between gap-4 border-b border-stone-800 pb-6">
        <div>
          <p className="text-sm text-amber-400">Offline full-text search</p>
          <h1 className="mt-2 text-3xl font-semibold">Search everything</h1>
          <p className="mt-2 text-sm text-stone-500">
            Books, Markdown notes, headings, and tags.
          </p>
        </div>
        <button
          className="rounded-lg border border-stone-700 px-4 py-2 text-sm disabled:opacity-50"
          disabled={busy}
          onClick={onRebuild}
          type="button"
        >
          {busy ? "Indexing…" : "Rebuild index"}
        </button>
      </header>

      <div className="mt-6 flex gap-3">
        <input
          aria-label="Search everything"
          autoFocus
          className="min-w-0 flex-1 rounded-xl border border-stone-700 bg-stone-900 px-4 py-3 text-lg outline-none focus:border-amber-500"
          onChange={(event) => onQueryChange(event.target.value)}
          placeholder="Search books, notes, headings, or tags…"
          type="search"
          value={query}
        />
        <select
          aria-label="Search scope"
          className="rounded-xl border border-stone-700 bg-stone-900 px-4"
          onChange={(event) => onScopeChange(event.target.value)}
          value={scope}
        >
          <option value="all">Everything</option>
          <option value="books">Books</option>
          <option value="notes">Notes</option>
          <option value="headings">Headings</option>
          <option value="tags">Tags</option>
        </select>
      </div>

      {diagnostics && (
        <p className="mt-3 text-xs text-stone-600">
          {diagnostics.documents} indexed documents
          {diagnostics.lastRebuildAt
            ? ` · last rebuilt ${diagnostics.lastRebuildAt}`
            : ""}
          {diagnostics.failedJobs
            ? ` · ${diagnostics.failedJobs} indexing issues`
            : ""}
        </p>
      )}
      {error && <ErrorPanel error={error} />}

      <div className="mt-6 space-y-2">
        {!query.trim() ? (
          <p className="py-24 text-center text-stone-600">
            Enter a query, or rebuild the index after your first upgrade.
          </p>
        ) : results.length === 0 && !busy ? (
          <p className="py-24 text-center text-stone-600">
            No results found. Try another term or rebuild the index.
          </p>
        ) : (
          results.map((result, index) => (
            <button
              className="block w-full rounded-xl border border-stone-800 bg-stone-900/60 p-4 text-left hover:border-amber-700"
              key={`${result.scope}-${result.sourceId}-${index}`}
              onClick={() => onOpenResult(result)}
              type="button"
            >
              <div className="flex items-center gap-2">
                <span className="rounded-full bg-stone-800 px-2 py-1 text-[10px] uppercase tracking-wider text-amber-300">
                  {result.scope}
                </span>
                <span className="truncate font-medium">{result.title}</span>
                {result.status !== "available" && (
                  <span className="ml-auto text-xs text-red-300">
                    {result.status}
                  </span>
                )}
              </div>
              {result.snippet && (
                <p
                  className="mt-2 line-clamp-2 text-sm leading-6 text-stone-400"
                  dangerouslySetInnerHTML={{ __html: safeSearchSnippet(result.snippet) }}
                />
              )}
              <p className="mt-2 truncate text-xs text-stone-600">
                {result.relativePath}
              </p>
            </button>
          ))
        )}
      </div>
    </>
  );
}

export function safeSearchSnippet(snippet: string): string {
  return snippet
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("&lt;mark&gt;", "<mark>")
    .replaceAll("&lt;/mark&gt;", "</mark>");
}

export function NotesWorkspace({
  books,
  busy,
  configuration,
  draft,
  error,
  notes,
  onChooseRoot,
  onCreate,
  onDraftChange,
  onOpenExternal,
  onOpenRoot,
  onRefresh,
  onSave,
  onSelect,
  selectedNote,
  summary,
}: {
  books: Book[];
  busy: boolean;
  configuration: NotesConfiguration | null;
  draft: string;
  error: DesktopError | null;
  notes: NoteListItem[];
  onChooseRoot: () => void;
  onCreate: (title: string, bookId: string | null) => void;
  onDraftChange: (body: string) => void;
  onOpenExternal: () => void;
  onOpenRoot: () => void;
  onRefresh: () => void;
  onSave: () => void;
  onSelect: (noteId: string) => void;
  selectedNote: NoteDetail | null;
  summary: NotesRefreshSummary | null;
}) {
  const [creating, setCreating] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newBookId, setNewBookId] = useState("");
  const [bookQuery, setBookQuery] = useState("");
  const [bookPickerOpen, setBookPickerOpen] = useState(false);
  const matchingBooks = useMemo(
    () => filterBookChoices(books, bookQuery),
    [books, bookQuery],
  );
  const selectedBook = books.find((book) => book.id === newBookId) ?? null;

  if (!configuration) {
    return (
      <div className="mx-auto mt-28 max-w-xl">
        <p className="text-sm text-amber-400">Markdown notes</p>
        <h1 className="mt-3 text-4xl font-semibold tracking-tight">
          Choose the folder that owns your notes.
        </h1>
        <p className="mt-5 leading-7 text-stone-400">
          Notes stay as portable Markdown files. Refresh only reads existing
          files; saving changes only the note currently open in the editor.
        </p>
        <button
          className="mt-8 rounded-lg bg-amber-400 px-5 py-3 font-medium text-stone-950 disabled:opacity-50"
          disabled={busy}
          onClick={onChooseRoot}
          type="button"
        >
          Choose notes folder
        </button>
        {error && <ErrorPanel error={error} />}
      </div>
    );
  }

  const changed = selectedNote !== null && draft !== selectedNote.body;
  return (
    <>
      <header className="flex flex-wrap items-end justify-between gap-4 border-b border-stone-800 pb-6">
        <div>
          <p className="text-sm text-amber-400">Markdown notes</p>
          <h1 className="mt-2 text-3xl font-semibold">
            {notes.length} {notes.length === 1 ? "note" : "notes"}
          </h1>
          <p className="mt-1 text-xs text-stone-500">
            {configuration.displayName}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            className="rounded-lg border border-stone-700 px-3 py-2 text-sm"
            disabled={busy}
            onClick={onOpenRoot}
            type="button"
          >
            Open folder
          </button>
          <button
            className="rounded-lg border border-stone-700 px-3 py-2 text-sm"
            disabled={busy}
            onClick={onChooseRoot}
            type="button"
          >
            Change folder
          </button>
          <button
            className="rounded-lg border border-stone-700 px-3 py-2 text-sm"
            disabled={busy}
            onClick={onRefresh}
            type="button"
          >
            {busy ? "Working…" : "Refresh"}
          </button>
          <button
            className="rounded-lg bg-amber-400 px-4 py-2 text-sm font-medium text-stone-950"
            onClick={() => setCreating(true)}
            type="button"
          >
            New note
          </button>
        </div>
      </header>

      {summary && (
        <p className="mt-4 text-xs text-stone-500">
          Refresh: {summary.discovered} found, {summary.added} added,{" "}
          {summary.updated} updated, {summary.missing} missing, {summary.issues}{" "}
          issues.
        </p>
      )}
      {error && <ErrorPanel error={error} />}

      <div className="mt-6 grid min-h-[70vh] grid-cols-[minmax(220px,0.8fr)_minmax(360px,2fr)] overflow-hidden rounded-xl border border-stone-800 bg-stone-900/40">
        <aside className="border-r border-stone-800 p-2">
          {notes.length === 0 ? (
            <p className="p-4 text-sm text-stone-500">
              No Markdown notes found. Create one or add .md files to this folder.
            </p>
          ) : (
            notes.map((note) => (
              <button
                className={`mb-1 block w-full rounded-lg p-3 text-left ${
                  selectedNote?.id === note.id
                    ? "bg-stone-800"
                    : "hover:bg-stone-900"
                }`}
                key={note.id}
                onClick={() => onSelect(note.id)}
                type="button"
              >
                <span className="block truncate text-sm text-stone-100">
                  {note.title}
                </span>
                <span className="mt-1 block truncate text-xs text-stone-500">
                  {note.bookTitle ?? note.relativePath}
                </span>
                {note.status !== "available" && (
                  <span className="mt-1 block text-xs text-red-300">
                    {note.status}
                  </span>
                )}
              </button>
            ))
          )}
        </aside>

        {selectedNote ? (
          <section className="flex min-w-0 flex-col p-5">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="min-w-0">
                <h2 className="truncate text-xl font-semibold">
                  {selectedNote.title}
                </h2>
                <p className="mt-1 truncate text-xs text-stone-500">
                  {selectedNote.relativePath}
                  {selectedNote.bookTitle && ` · ${selectedNote.bookTitle}`}
                </p>
              </div>
              <div className="flex gap-2">
                <button
                  className="rounded-lg border border-stone-700 px-3 py-2 text-xs"
                  onClick={onOpenExternal}
                  type="button"
                >
                  Open externally
                </button>
                <button
                  className="rounded-lg bg-amber-400 px-4 py-2 text-xs font-medium text-stone-950 disabled:opacity-40"
                  disabled={!changed || busy}
                  onClick={onSave}
                  type="button"
                >
                  {busy ? "Saving…" : changed ? "Save" : "Saved"}
                </button>
              </div>
            </div>
            <textarea
              aria-label="Markdown note"
              className="mt-5 min-h-[480px] flex-1 resize-y rounded-xl border border-stone-700 bg-stone-950 p-4 font-mono text-sm leading-6 text-stone-200 outline-none focus:border-amber-500"
              onChange={(event) => onDraftChange(event.target.value)}
              spellCheck={false}
              value={draft}
            />
            <div className="mt-5 border-t border-stone-800 pt-4">
              <p className="text-xs font-semibold uppercase tracking-wider text-stone-500">
                Backlinks
              </p>
              {selectedNote.backlinks.length === 0 ? (
                <p className="mt-2 text-sm text-stone-600">
                  No other note links here yet.
                </p>
              ) : (
                <div className="mt-2 flex flex-wrap gap-2">
                  {selectedNote.backlinks.map((backlink) => (
                    <button
                      className="rounded-full border border-stone-700 px-3 py-1 text-xs text-stone-300 hover:border-amber-500"
                      key={backlink.id}
                      onClick={() => onSelect(backlink.id)}
                      type="button"
                    >
                      {backlink.title}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </section>
        ) : (
          <div className="flex items-center justify-center p-10 text-sm text-stone-600">
            Select a note to read or edit it.
          </div>
        )}
      </div>

      {creating && (
        <div
          aria-label="Create note"
          aria-modal="true"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-5"
          role="dialog"
        >
          <form
            className="w-full max-w-lg rounded-2xl border border-stone-700 bg-stone-900 p-6"
            onSubmit={(event) => {
              event.preventDefault();
              if (!newTitle.trim()) return;
              onCreate(newTitle, newBookId || null);
              setCreating(false);
              setNewTitle("");
              setNewBookId("");
            }}
          >
            <p className="text-sm text-amber-400">Portable Markdown</p>
            <h2 className="mt-2 text-2xl font-semibold">Create note</h2>
            <label className="mt-6 block text-sm text-stone-300">
              Title
              <input
                autoFocus
                className="mt-2 w-full rounded-lg border border-stone-700 bg-stone-950 px-3 py-2 outline-none focus:border-amber-500"
                maxLength={200}
                onChange={(event) => setNewTitle(event.target.value)}
                value={newTitle}
              />
            </label>
            <label className="mt-4 block text-sm text-stone-300">
              Related book (optional)
              <div className="relative mt-2">
                <input
                  aria-autocomplete="list"
                  aria-controls="related-book-options"
                  aria-expanded={bookPickerOpen}
                  aria-label="Search related book"
                  className="w-full rounded-lg border border-stone-700 bg-stone-950 px-3 py-2 pr-10 outline-none focus:border-amber-500"
                  onChange={(event) => {
                    setBookQuery(event.target.value);
                    setNewBookId("");
                    setBookPickerOpen(true);
                  }}
                  onFocus={() => setBookPickerOpen(true)}
                  placeholder="Type a title or folder…"
                  role="combobox"
                  value={selectedBook ? selectedBook.title : bookQuery}
                />
                {(bookQuery || selectedBook) && (
                  <button
                    aria-label="Clear related book"
                    className="absolute right-2 top-1/2 -translate-y-1/2 px-2 text-stone-500 hover:text-stone-200"
                    onClick={() => {
                      setNewBookId("");
                      setBookQuery("");
                      setBookPickerOpen(true);
                    }}
                    type="button"
                  >
                    ×
                  </button>
                )}
                {bookPickerOpen && (
                  <div
                    className="absolute z-10 mt-1 max-h-64 w-full overflow-y-auto rounded-lg border border-stone-700 bg-stone-950 p-1 shadow-2xl"
                    id="related-book-options"
                    role="listbox"
                  >
                    <button
                      aria-selected={!newBookId}
                      className="block w-full rounded-md px-3 py-2 text-left text-sm hover:bg-stone-800"
                      onClick={() => {
                        setNewBookId("");
                        setBookQuery("");
                        setBookPickerOpen(false);
                      }}
                      role="option"
                      type="button"
                    >
                      General note
                      <span className="mt-0.5 block text-xs text-stone-500">
                        Not linked to a book
                      </span>
                    </button>
                    {matchingBooks.map((book) => (
                      <button
                        aria-selected={newBookId === book.id}
                        className="block w-full rounded-md px-3 py-2 text-left hover:bg-stone-800"
                        key={book.id}
                        onClick={() => {
                          setNewBookId(book.id);
                          setBookQuery("");
                          setBookPickerOpen(false);
                        }}
                        role="option"
                        type="button"
                      >
                        <span className="block truncate text-sm">
                          {book.title}
                        </span>
                        <span className="mt-0.5 block truncate text-xs text-stone-500">
                          {book.relativePath}
                        </span>
                      </button>
                    ))}
                    {matchingBooks.length === 0 && (
                      <p className="px-3 py-4 text-center text-sm text-stone-500">
                        No matching books.
                      </p>
                    )}
                  </div>
                )}
              </div>
            </label>
            <div className="mt-6 flex justify-end gap-2">
              <button
                className="rounded-lg border border-stone-700 px-4 py-2 text-sm"
              onClick={() => setCreating(false)}
                type="button"
              >
                Cancel
              </button>
              <button
                className="rounded-lg bg-amber-400 px-4 py-2 text-sm font-medium text-stone-950 disabled:opacity-40"
                disabled={!newTitle.trim() || busy}
                type="submit"
              >
                Create
              </button>
            </div>
          </form>
        </div>
      )}
    </>
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

export function BookDetailPage({
  busy,
  detail,
  error,
  onBack,
  onEditTitle,
  onForceCover,
  onNewNote,
  onOpenFolder,
  onOpenNote,
  onSave,
}: {
  busy: boolean;
  detail: BookDetail;
  error: DesktopError | null;
  onBack: () => void;
  onEditTitle: () => void;
  onForceCover: () => void;
  onNewNote: () => void;
  onOpenFolder: () => void;
  onOpenNote: (noteId: string) => void;
  onSave: (
    readingStatus: BookDetail["readingStatus"],
    tags: string[],
  ) => void;
}) {
  const [readingStatus, setReadingStatus] = useState(detail.readingStatus);
  const [tagDraft, setTagDraft] = useState(
    detail.tags.map((tag) => `#${tag}`).join(" "),
  );
  const tags = parseBookTags(tagDraft);
  const changed =
    readingStatus !== detail.readingStatus ||
    tags.join("\u0000") !== detail.tags.join("\u0000");

  return (
    <>
      <button
        className="text-sm text-stone-400 hover:text-amber-300"
        onClick={onBack}
        type="button"
      >
        ← Back to library
      </button>
      <div className="mt-6 grid gap-8 lg:grid-cols-[minmax(220px,320px)_minmax(0,1fr)]">
        <section>
          <Cover book={detail} />
          <button
            className="mt-4 w-full rounded-lg border border-stone-700 px-4 py-2 text-sm text-stone-200 hover:border-amber-600 disabled:opacity-50"
            disabled={
              busy ||
              !["available", "unavailable"].includes(detail.status)
            }
            onClick={onForceCover}
            type="button"
          >
            {busy ? "Working…" : "Force cover generation"}
          </button>
          <p className="mt-2 text-xs leading-5 text-stone-500">
            Waits up to 30 seconds so a cloud file can download locally. The
            previous cover remains visible if this attempt fails. Generated
            covers are stored in the app data folder across restarts.
          </p>
        </section>

        <section className="min-w-0">
          <div className="border-b border-stone-800 pb-6">
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div className="min-w-0">
                <p className="text-sm text-amber-400">Book detail</p>
                <h1 className="mt-2 max-w-4xl text-3xl font-semibold leading-tight">
                  {detail.title}
                </h1>
                <p className="mt-3 break-all text-sm text-stone-500">
                  {detail.relativePath}
                </p>
              </div>
              <div className="flex gap-2">
                <button
                  className="rounded-lg border border-stone-700 px-3 py-2 text-sm"
                  onClick={onEditTitle}
                  type="button"
                >
                  Edit title
                </button>
                <button
                  className="rounded-lg border border-stone-700 px-3 py-2 text-sm"
                  onClick={onOpenFolder}
                  type="button"
                >
                  Open folder
                </button>
              </div>
            </div>
          </div>

          <div className="grid gap-7 py-7 xl:grid-cols-2">
            <div>
              <h2 className="text-sm font-semibold text-stone-200">
                Reading status
              </h2>
              <div className="mt-3 grid grid-cols-3 rounded-lg border border-stone-700 p-1">
                {(
                  [
                    ["unread", "Unread"],
                    ["reading", "Reading"],
                    ["read", "Read"],
                  ] as const
                ).map(([value, label]) => (
                  <button
                    aria-pressed={readingStatus === value}
                    className={`rounded-md px-3 py-2 text-sm ${
                      readingStatus === value
                        ? "bg-amber-400 font-medium text-stone-950"
                        : "text-stone-400 hover:text-stone-100"
                    }`}
                    key={value}
                    onClick={() => setReadingStatus(value)}
                    type="button"
                  >
                    {label}
                  </button>
                ))}
              </div>
            </div>

            <div>
              <label className="text-sm font-semibold text-stone-200">
                Hashtags
                <input
                  className="mt-3 w-full rounded-lg border border-stone-700 bg-stone-900 px-3 py-2 font-normal outline-none placeholder:text-stone-600 focus:border-amber-500"
                  onChange={(event) => setTagDraft(event.target.value)}
                  placeholder="#japanese #psychology #to-read"
                  value={tagDraft}
                />
              </label>
              <p className="mt-2 text-xs text-stone-500">
                Separate tags with spaces or commas. These tags are included in
                global search.
              </p>
            </div>
          </div>

          <div className="flex justify-end border-b border-stone-800 pb-7">
            <button
              className="rounded-lg bg-amber-400 px-5 py-2 text-sm font-medium text-stone-950 disabled:opacity-40"
              disabled={!changed || busy}
              onClick={() => onSave(readingStatus, tags)}
              type="button"
            >
              {busy ? "Saving…" : changed ? "Save details" : "Saved"}
            </button>
          </div>

          <div className="py-7">
            <div className="flex items-center justify-between gap-4">
              <div>
                <h2 className="text-lg font-semibold">Markdown notes</h2>
                <p className="mt-1 text-sm text-stone-500">
                  Notes remain portable files in your notes folder.
                </p>
              </div>
              <button
                className="rounded-lg bg-amber-400 px-4 py-2 text-sm font-medium text-stone-950"
                onClick={onNewNote}
                type="button"
              >
                New note
              </button>
            </div>
            {detail.notes.length === 0 ? (
              <p className="mt-6 rounded-lg border border-dashed border-stone-800 p-5 text-sm text-stone-500">
                No Markdown notes are linked to this book yet.
              </p>
            ) : (
              <div className="mt-5 space-y-2">
                {detail.notes.map((note) => (
                  <button
                    className="block w-full rounded-lg border border-stone-800 px-4 py-3 text-left text-sm hover:border-amber-700"
                    key={note.id}
                    onClick={() => onOpenNote(note.id)}
                    type="button"
                  >
                    {note.title}
                  </button>
                ))}
              </div>
            )}
          </div>

          <dl className="grid grid-cols-2 gap-x-8 gap-y-4 border-t border-stone-800 py-6 text-sm">
            <div>
              <dt className="text-stone-500">Type</dt>
              <dd className="mt-1">{detail.kind === "pdf_file" ? "PDF" : "Images"}</dd>
            </div>
            <div>
              <dt className="text-stone-500">Source status</dt>
              <dd className="mt-1">{detail.status}</dd>
            </div>
            <div>
              <dt className="text-stone-500">Pages</dt>
              <dd className="mt-1">{detail.pageCount ?? "Unknown"}</dd>
            </div>
            <div>
              <dt className="text-stone-500">Size</dt>
              <dd className="mt-1">
                {detail.sizeBytes ? formatBytes(detail.sizeBytes) : "Unknown"}
              </dd>
            </div>
          </dl>
          {error && <ErrorPanel error={error} />}
        </section>
      </div>
    </>
  );
}

export function parseBookTags(value: string): string[] {
  return Array.from(
    new Set(
      value
        .split(/[\s,]+/)
        .map((tag) => tag.trim().replace(/^#+/, ""))
        .filter(Boolean),
    ),
  ).sort();
}

function LibraryWorkspace({
  activeScan,
  books,
  error,
  onCancel,
  onEditBook,
  onOpenDetail,
  onOpenBook,
  onRepair,
  onRelinkBook,
  onRescan,
  onSearchChange,
  onViewChange,
  openingBookId,
  progress,
  searchQuery,
  summary,
  totalBooks,
  view,
}: {
  activeScan: ActiveScan | null;
  books: Book[];
  error: DesktopError | null;
  onCancel: () => void;
  onEditBook: (book: Book) => void;
  onOpenDetail: (book: Book) => void;
  onOpenBook: (book: Book) => void;
  onRepair: () => void;
  onRelinkBook: (book: Book) => void;
  onRescan: () => void;
  onSearchChange: (query: string) => void;
  onViewChange: (view: "grid" | "list") => void;
  openingBookId: string | null;
  progress: ScanProgress | null;
  searchQuery: string;
  summary: ScanSummary | null;
  totalBooks: number;
  view: "grid" | "list";
}) {
  const scanLabels = scanButtonLabels(activeScan);
  return (
    <>
      <header className="flex flex-wrap items-end justify-between gap-4 border-b border-stone-800 pb-6">
        <div>
          <p className="text-sm text-amber-400">Local catalog</p>
          <h1 className="mt-2 text-3xl font-semibold">
            {searchQuery.trim()
              ? `${books.length} of ${totalBooks} books`
              : `${totalBooks} ${totalBooks === 1 ? "book" : "books"}`}
          </h1>
        </div>
        <div className="flex gap-2">
          <button
            className="rounded-lg border border-stone-700 px-3 py-2 text-sm disabled:opacity-50"
            disabled={activeScan !== null}
            onClick={onRepair}
            type="button"
          >
            {scanLabels.repair}
          </button>
          <button
            className="rounded-lg bg-amber-400 px-4 py-2 text-sm font-medium text-stone-950 disabled:opacity-50"
            disabled={activeScan !== null}
            onClick={onRescan}
            type="button"
          >
            {scanLabels.rescan}
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

      <div className="mt-6 flex items-center gap-3 rounded-xl border border-stone-700 bg-stone-900/70 px-4 py-3 focus-within:border-amber-500">
        <span aria-hidden="true" className="text-stone-500">
          ⌕
        </span>
        <input
          aria-label="Search catalog"
          autoComplete="off"
          className="min-w-0 flex-1 bg-transparent text-base outline-none placeholder:text-stone-600"
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder="Search title, folder, type, or status…"
          spellCheck={false}
          type="search"
          value={searchQuery}
        />
        {searchQuery && (
          <button
            className="text-xs text-stone-400 hover:text-stone-200"
            onClick={() => onSearchChange("")}
            type="button"
          >
            Clear
          </button>
        )}
      </div>

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
          {searchQuery.trim()
            ? `No books match “${searchQuery.trim()}”.`
            : "No supported books found yet. Try a rescan."}
        </div>
      ) : (
        <div className={view === "grid" ? "book-grid mt-7" : "mt-7 space-y-2"}>
          {books.map((book) => (
            <BookCard
              book={book}
              isOpening={openingBookId === book.id}
              key={book.id}
              onEdit={() => onEditBook(book)}
              onDetail={() => onOpenDetail(book)}
              onOpen={() => onOpenBook(book)}
              onRelink={() => onRelinkBook(book)}
              view={view}
            />
          ))}
        </div>
      )}
    </>
  );
}

function BookCard({
  book,
  isOpening,
  onEdit,
  onDetail,
  onOpen,
  onRelink,
  view,
}: {
  book: Book;
  isOpening: boolean;
  onEdit: () => void;
  onDetail: () => void;
  onOpen: () => void;
  onRelink: () => void;
  view: "grid" | "list";
}) {
  const canOpen =
    book.status === "available" ||
    book.status === "unavailable" ||
    book.status === "missing";
  const details = [
    book.kind === "pdf_file" ? "PDF" : "Images",
    book.pageCount ? `${book.pageCount} pages` : null,
    book.sizeBytes ? formatBytes(book.sizeBytes) : null,
  ]
    .filter(Boolean)
    .join(" · ");
  if (view === "list") {
    return (
      <article className="book-virtual-row relative flex items-center gap-4 rounded-lg border border-stone-800 bg-stone-900/50 p-3">
        <button
          aria-label={`View details for ${book.title}`}
          className="shrink-0 rounded-md text-left outline-none ring-amber-500 focus-visible:ring-2"
          onClick={onDetail}
          type="button"
        >
          <Cover book={book} compact />
        </button>
        <div className="min-w-0 flex-1">
          <button
            className="block max-w-full truncate text-left font-medium hover:text-amber-300"
            onClick={onDetail}
            type="button"
          >
            {book.title}
          </button>
          <p className="mt-1 truncate text-xs text-stone-500">
            {book.relativePath}
          </p>
        </div>
        <p className="text-xs text-stone-400">{details}</p>
        <BookActionsMenu
          book={book}
          canOpen={canOpen}
          isOpening={isOpening}
          onEdit={onEdit}
          onOpen={onOpen}
          onRelink={onRelink}
        />
      </article>
    );
  }
  return (
    <article className="book-virtual-card group relative flex h-full min-w-0 flex-col">
      <button
        aria-label={`View details for ${book.title}`}
        className="rounded-md text-left outline-none ring-amber-500 transition enabled:hover:brightness-110 focus-visible:ring-2"
        onClick={onDetail}
        type="button"
      >
        <Cover book={book} />
      </button>
      <div className="absolute right-2 top-2">
        <BookActionsMenu
          book={book}
          canOpen={canOpen}
          isOpening={isOpening}
          onEdit={onEdit}
          onOpen={onOpen}
          onRelink={onRelink}
          overlay
        />
      </div>
      <button
        className="mt-3 truncate text-left font-medium hover:text-amber-300 disabled:text-stone-400"
        onClick={onDetail}
        title={book.title}
        type="button"
      >
        {book.title}
      </button>
      <p className="mt-1 min-h-5 truncate text-xs text-stone-500">{details}</p>
      <p
        aria-hidden={book.status === "available"}
        className={`mt-1 min-h-5 text-xs ${
          book.status === "available" ? "invisible" : "text-red-300"
        }`}
      >
        {book.status === "available" ? "available" : book.status}
      </p>
    </article>
  );
}

function BookActionsMenu({
  book,
  canOpen,
  isOpening,
  onEdit,
  onOpen,
  onRelink,
  overlay = false,
}: {
  book: Book;
  canOpen: boolean;
  isOpening: boolean;
  onEdit: () => void;
  onOpen: () => void;
  onRelink: () => void;
  overlay?: boolean;
}) {
  return (
    <details className="book-actions relative">
      <summary
        aria-label={`Actions for ${book.title}`}
        className={`flex h-8 w-9 cursor-pointer list-none items-center justify-center rounded-lg border text-sm tracking-widest ${
          overlay
            ? "border-stone-600 bg-stone-950/90 text-stone-200 shadow-lg backdrop-blur"
            : "border-stone-700 bg-stone-900 text-stone-300"
        }`}
        title="Book actions"
      >
        •••
      </summary>
      <div className="absolute right-0 z-30 mt-1 w-40 overflow-hidden rounded-lg border border-stone-700 bg-stone-900 p-1 shadow-2xl">
        <button
          className="block w-full rounded-md px-3 py-2 text-left text-sm text-stone-200 enabled:hover:bg-stone-800 disabled:text-stone-600"
          disabled={!canOpen || isOpening}
          onClick={onOpen}
          type="button"
        >
          {isOpening ? "Opening…" : "Open folder"}
        </button>
        <button
          className="block w-full rounded-md px-3 py-2 text-left text-sm text-stone-200 hover:bg-stone-800"
          onClick={onEdit}
          type="button"
        >
          Edit title
        </button>
        {book.status === "missing" && (
          <button
            className="block w-full rounded-md px-3 py-2 text-left text-sm text-amber-300 hover:bg-stone-800"
            onClick={onRelink}
            type="button"
          >
            Locate source…
          </button>
        )}
      </div>
    </details>
  );
}

function EditBookDialog({
  book,
  error,
  isSaving,
  onCancel,
  onSave,
}: {
  book: Book;
  error: DesktopError | null;
  isSaving: boolean;
  onCancel: () => void;
  onSave: (title: string) => void;
}) {
  const [title, setTitle] = useState(book.title);
  const valid = isValidBookDisplayTitle(title);

  return (
    <div
      aria-label="Edit book title"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-5"
      role="dialog"
    >
      <form
        className="w-full max-w-xl rounded-2xl border border-stone-700 bg-stone-900 p-6 shadow-2xl"
        onSubmit={(event) => {
          event.preventDefault();
          if (valid && !isSaving) onSave(title);
        }}
      >
        <p className="text-sm text-amber-400">App-local metadata</p>
        <h2 className="mt-2 text-2xl font-semibold">Edit display title</h2>
        <p className="mt-2 text-sm text-stone-400">
          This changes only the catalog title. The source file or folder name is
          never changed.
        </p>
        <label className="mt-6 block text-xs font-medium uppercase tracking-wider text-stone-500">
          Title
          <textarea
            autoFocus
            className="mt-2 min-h-28 w-full resize-y rounded-xl border border-stone-700 bg-stone-950 p-3 text-base normal-case tracking-normal text-stone-100 outline-none focus:border-amber-500"
            maxLength={512}
            onChange={(event) => setTitle(event.target.value)}
            value={title}
          />
        </label>
        <p className="mt-2 text-right text-xs text-stone-600">
          {bookTitleCharacterCount(title)}/512
        </p>
        {error && <ErrorPanel error={error} />}
        <div className="mt-6 flex justify-end gap-2">
          <button
            className="rounded-lg border border-stone-700 px-4 py-2 text-sm text-stone-300"
            disabled={isSaving}
            onClick={onCancel}
            type="button"
          >
            Cancel
          </button>
          <button
            className="rounded-lg bg-amber-400 px-4 py-2 text-sm font-medium text-stone-950 disabled:cursor-not-allowed disabled:opacity-40"
            disabled={!valid || isSaving}
            type="submit"
          >
            {isSaving ? "Saving…" : "Save title"}
          </button>
        </div>
      </form>
    </div>
  );
}

export function bookTitleCharacterCount(title: string): number {
  return Array.from(title).length;
}

export function isValidBookDisplayTitle(title: string): boolean {
  return (
    title.trim().length > 0 &&
    bookTitleCharacterCount(title) <= 512 &&
    !/[\u0000-\u001f\u007f-\u009f]/u.test(title)
  );
}

export function filterCatalogBooks(books: Book[], query: string): Book[] {
  const terms = query
    .normalize("NFKC")
    .toLocaleLowerCase()
    .trim()
    .split(/\s+/)
    .filter(Boolean);
  if (terms.length === 0) return books;

  return books.filter((book) => {
    const kind = book.kind === "pdf_file" ? "pdf" : "images image folder";
    const searchable = `${book.title} ${book.relativePath} ${kind} ${book.status}`
      .normalize("NFKC")
      .toLocaleLowerCase();
    return terms.every((term) => searchable.includes(term));
  });
}

export function filterBookChoices(books: Book[], query: string): Book[] {
  const terms = query
    .normalize("NFKC")
    .toLocaleLowerCase()
    .trim()
    .split(/\s+/)
    .filter(Boolean);
  if (terms.length === 0) return books;

  return books.filter((book) => {
    const searchable = `${book.title} ${book.relativePath}`
      .normalize("NFKC")
      .toLocaleLowerCase();
    return terms.every((term) => searchable.includes(term));
  });
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
      updated, {summary.missing} missing. Covers:{" "}
      {summary.thumbnailsGenerated} generated, {summary.thumbnailFailures}{" "}
      failed. {summary.issues} scan issues.
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
