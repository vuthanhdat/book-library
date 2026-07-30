import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  useCallback,
  useDeferredValue,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";

const navigation = ["Library", "Study", "Recent", "Notes", "Search", "Settings"];

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

interface CoverProgress {
  bookId: string;
  stage:
    | "opening_source"
    | "rendering_first_page"
    | "saving_cover"
    | "completed";
}

export function coverProgressMessage(stage: CoverProgress["stage"]) {
  switch (stage) {
    case "opening_source":
      return "Opening the source file and waiting for local availability…";
    case "rendering_first_page":
      return "Source opened. Rendering the first page…";
    case "saving_cover":
      return "First page rendered. Saving the cover to app data…";
    case "completed":
      return "Cover generated successfully.";
  }
}

interface ScanSummary {
  discovered: number;
  added: number;
  updated: number;
  missing: number;
  issues: number;
  thumbnailsRecovered: number;
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

export function normalizeLookupSelection(value: string): string | null {
  const selection = value.trim();
  return selection && selection.length <= 200 ? selection : null;
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
  sourceKind: "book" | "note" | "ocr_page";
  sourceId: string;
  scope: "books" | "notes" | "tags" | "headings" | "ocr";
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

interface StudyModule {
  id: "dictionary" | "ocr" | "anki" | "ai" | "trusted_modules";
  enabled: boolean;
  available: boolean;
  status: "disabled" | "ready" | "unavailable";
}

interface DictionaryEntry {
  id: string;
  expression: string;
  reading: string;
  partOfSpeech: string;
  meaningVi: string;
  hanViet: string | null;
  packageName: string;
  packageVersion: string;
}

interface JapaneseToken {
  surface: string;
  start: number;
  end: number;
  entries: DictionaryEntry[];
}

interface DictionaryLookup {
  query: string;
  entries: DictionaryEntry[];
  tokens: JapaneseToken[];
}

interface DictionaryImportSummary {
  packageId: string;
  imported: number;
  skipped: number;
}

interface OcrBlock {
  blockIndex: number;
  text: string;
  confidence: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

interface OcrPage {
  id: string;
  bookId: string;
  bookTitle: string;
  pageIndex: number;
  text: string;
  confidence: number;
  providerId: string;
  providerVersion: string;
  blocks: OcrBlock[];
}

export interface StudyReaderPage {
  bookId: string;
  bookTitle: string;
  pageIndex: number;
  pageCount: number;
  width: number;
  height: number;
  imageDataUrl: string;
}

interface LearningDraft {
  id: string;
  sourceKind: string;
  sourceId: string;
  bookRelativePath: string | null;
  pageIndex: number | null;
  front: string;
  back: string;
  tags: string[];
  status: "draft" | "approved" | "exported";
}

interface AiDraft {
  id: string;
  kind: string;
  context: string;
  content: string;
  accepted: boolean;
}

interface TrustedModule {
  id: string;
  version: string;
  capabilities: string[];
  permissions: string[];
  compatible: boolean;
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
  const [scanSummaryKind, setScanSummaryKind] = useState<ActiveScan | null>(null);
  const [operationError, setOperationError] = useState<DesktopError | null>(null);
  const [openingBookId, setOpeningBookId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [editingBook, setEditingBook] = useState<Book | null>(null);
  const [editError, setEditError] = useState<DesktopError | null>(null);
  const [isSavingTitle, setIsSavingTitle] = useState(false);
  const [selectedBookDetail, setSelectedBookDetail] =
    useState<BookDetail | null>(null);
  const catalogScrollY = useRef(0);
  const restoreCatalogScroll = useRef(false);
  const [bookDetailBusy, setBookDetailBusy] = useState(false);
  const [bookDetailError, setBookDetailError] =
    useState<DesktopError | null>(null);
  const [coverProgress, setCoverProgress] = useState<string[]>([]);
  const [activeSection, setActiveSection] = useState<
    "Library" | "Study" | "Notes" | "Search"
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
  const [studyModules, setStudyModules] = useState<StudyModule[]>([]);
  const [studyBusy, setStudyBusy] = useState(false);
  const [studyError, setStudyError] = useState<DesktopError | null>(null);
  const [studyNotice, setStudyNotice] = useState<string | null>(null);
  const [dictionaryQuery, setDictionaryQuery] = useState("");
  const [dictionaryLookup, setDictionaryLookup] =
    useState<DictionaryLookup | null>(null);
  const [saveLookupHistory, setSaveLookupHistory] = useState(false);
  const [ocrBookId, setOcrBookId] = useState("");
  const [ocrPageNumber, setOcrPageNumber] = useState(1);
  const [ocrPages, setOcrPages] = useState<OcrPage[]>([]);
  const [readerPage, setReaderPage] = useState<StudyReaderPage | null>(null);
  const [readerBusy, setReaderBusy] = useState(false);
  const [readerError, setReaderError] = useState<DesktopError | null>(null);
  const [learningDrafts, setLearningDrafts] = useState<LearningDraft[]>([]);
  const [aiKind, setAiKind] = useState("explain");
  const [aiContext, setAiContext] = useState("");
  const [aiDrafts, setAiDrafts] = useState<AiDraft[]>([]);
  const [trustedModules, setTrustedModules] = useState<TrustedModule[]>([]);
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

  const loadStudy = useCallback(async () => {
    const [modules, pages, drafts, assistantDrafts, manifests] =
      await Promise.all([
        invoke<StudyModule[]>("get_study_modules"),
        invoke<OcrPage[]>("list_ocr_pages", { bookId: null }),
        invoke<LearningDraft[]>("list_learning_drafts"),
        invoke<AiDraft[]>("list_ai_drafts"),
        invoke<TrustedModule[]>("list_trusted_modules"),
      ]);
    setStudyModules(modules);
    setOcrPages(pages);
    setLearningDrafts(drafts);
    setAiDrafts(assistantDrafts);
    setTrustedModules(manifests);
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<CoverProgress>("book_cover_progressed", (event) => {
      if (event.payload.bookId !== selectedBookDetail?.id) return;
      setCoverProgress((current) => [
        ...current,
        `${selectedBookDetail.title}: ${coverProgressMessage(event.payload.stage)}`,
      ]);
    }).then((stop) => {
      unlisten = stop;
    });
    return () => unlisten?.();
  }, [selectedBookDetail?.id]);

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
      invoke<StudyModule[]>("get_study_modules"),
    ])
      .then(async ([status, configured, configuredNotes, modules]) => {
        setStartup({ kind: "healthy", status });
        setConfiguration(configured);
        setNotesConfiguration(configuredNotes);
        setStudyModules(modules);
        if (configured) await loadBooks();
        if (configuredNotes) await loadNotes();
      })
      .catch((error: unknown) =>
        setStartup({ kind: "error", error: desktopError(error) }),
      );
  }, [loadBooks, loadNotes]);

  useEffect(() => {
    if (activeSection !== "Study") return;
    setStudyBusy(true);
    setStudyError(null);
    void loadStudy()
      .catch((error) => setStudyError(desktopError(error)))
      .finally(() => setStudyBusy(false));
  }, [activeSection, loadStudy]);

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
    setScanSummaryKind(scanKind);
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
    catalogScrollY.current = window.scrollY;
    setBookDetailBusy(true);
    setBookDetailError(null);
    setCoverProgress([]);
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

  const closeBookDetail = () => {
    restoreCatalogScroll.current = true;
    setSelectedBookDetail(null);
  };

  useLayoutEffect(() => {
    if (selectedBookDetail || !restoreCatalogScroll.current) return;
    restoreCatalogScroll.current = false;
    const frame = window.requestAnimationFrame(() => {
      window.scrollTo({ top: catalogScrollY.current, behavior: "auto" });
    });
    return () => window.cancelAnimationFrame(frame);
  }, [selectedBookDetail]);

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
    setCoverProgress([
      `Cover generation requested for “${selectedBookDetail.title}”.`,
    ]);
    try {
      const detail = await invoke<BookDetail>("force_book_cover", {
        bookId: selectedBookDetail.id,
      });
      setSelectedBookDetail(detail);
      setBooks((current) =>
        current.map((book) =>
          book.id === detail.id
            ? {
                ...book,
                pageCount: detail.pageCount,
                status: detail.status,
                thumbnailDataUrl: detail.thumbnailDataUrl,
                thumbnailStatus: detail.thumbnailStatus,
              }
            : book,
        ),
      );
      await loadBooks();
    } catch (error) {
      const detailError = desktopError(error);
      setBookDetailError(detailError);
      setCoverProgress((current) => [
        ...current,
        `Failed: ${detailError.message}`,
      ]);
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

  const toggleStudyModule = async (module: StudyModule) => {
    setStudyBusy(true);
    setStudyError(null);
    try {
      setStudyModules(
        await invoke<StudyModule[]>("set_study_module_enabled", {
          moduleId: module.id,
          enabled: !module.enabled,
        }),
      );
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
    }
  };

  const lookupJapanese = async (query = dictionaryQuery) => {
    setStudyBusy(true);
    setStudyError(null);
    try {
      const result = await invoke<DictionaryLookup>("lookup_japanese", {
        query,
        saveHistory: saveLookupHistory,
      });
      setDictionaryQuery(result.query);
      setDictionaryLookup(result);
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
    }
  };

  const importDictionaryPackage = async () => {
    const selectedPath = await open({
      directory: false,
      multiple: false,
      filters: [
        { name: "Dictionary package", extensions: ["zip", "tsv"] },
      ],
    });
    if (typeof selectedPath !== "string") return;
    const isTsv = selectedPath.toLowerCase().endsWith(".tsv");
    const name = isTsv ? window.prompt("Dictionary package name") : null;
    const version = isTsv ? window.prompt("Package version", "1") : null;
    const licenseId = window.prompt(
      "License identifier or provenance",
      "user-provided",
    );
    if (
      (isTsv && (!name?.trim() || !version?.trim())) ||
      !licenseId?.trim()
    )
      return;
    setStudyBusy(true);
    setStudyError(null);
    setStudyNotice(null);
    try {
      const summary = await invoke<DictionaryImportSummary>(
        "import_dictionary_package",
        {
        selectedPath,
        name: name?.trim() || null,
        version: version?.trim() || null,
        licenseId,
        },
      );
      setStudyNotice(
        `Imported ${summary.imported.toLocaleString()} dictionary entries.${
          summary.skipped > 0
            ? ` Skipped ${summary.skipped.toLocaleString()} entries without usable definitions.`
            : ""
        }`,
      );
      if (dictionaryQuery.trim()) await lookupJapanese(dictionaryQuery);
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
    }
  };

  const runPageOcr = async () => {
    if (!ocrBookId) return;
    setStudyBusy(true);
    setStudyError(null);
    try {
      const page = await invoke<OcrPage>("run_page_ocr", {
        bookId: ocrBookId,
        pageIndex: Math.max(0, ocrPageNumber - 1),
      });
      setOcrPages((current) => [
        page,
        ...current.filter((item) => item.id !== page.id),
      ]);
      setDictionaryQuery(page.text);
      await invoke<SearchRebuildSummary>("rebuild_search_index");
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
    }
  };

  const openStudyReader = async (book: Book, pageIndex = 0) => {
    setReaderBusy(true);
    setReaderError(null);
    setOperationError(null);
    try {
      const page = await invoke<StudyReaderPage>("get_study_reader_page", {
        bookId: book.id,
        pageIndex,
      });
      setReaderPage(page);
      const storedPages = await invoke<OcrPage[]>("list_ocr_pages", {
        bookId: book.id,
      });
      setOcrPages((current) => [
        ...storedPages,
        ...current.filter((item) => item.bookId !== book.id),
      ]);
    } catch (error) {
      const nextError = desktopError(error);
      setReaderError(nextError);
      if (!readerPage) setOperationError(nextError);
    } finally {
      setReaderBusy(false);
    }
  };

  const runReaderOcr = async () => {
    if (!readerPage) return;
    setReaderBusy(true);
    setReaderError(null);
    try {
      const page = await invoke<OcrPage>("run_page_ocr", {
        bookId: readerPage.bookId,
        pageIndex: readerPage.pageIndex,
      });
      setOcrPages((current) => [
        page,
        ...current.filter((item) => item.id !== page.id),
      ]);
      await invoke<SearchRebuildSummary>("rebuild_search_index");
    } catch (error) {
      setReaderError(desktopError(error));
    } finally {
      setReaderBusy(false);
    }
  };

  const trimOcrPage = async (page: OcrPage) => {
    setStudyBusy(true);
    setStudyError(null);
    try {
      const updated = await invoke<OcrPage>("trim_ocr_page", {
        pageId: page.id,
      });
      setOcrPages((current) =>
        current.map((item) => (item.id === updated.id ? updated : item)),
      );
      if (dictionaryQuery === page.text) {
        setDictionaryQuery(updated.text);
      }
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
    }
  };

  const createDictionaryDraft = async (entry: DictionaryEntry) => {
    setStudyBusy(true);
    setStudyError(null);
    try {
      const draft = await invoke<LearningDraft>("create_learning_draft", {
        sourceKind: "dictionary_lookup",
        sourceId: entry.id,
        bookRelativePath: null,
        pageIndex: null,
        front: entry.expression,
        back: `${entry.reading}\n${entry.meaningVi}${
          entry.hanViet ? `\nHán–Việt: ${entry.hanViet}` : ""
        }`,
        tags: ["japanese", entry.partOfSpeech.replaceAll(" ", "-")],
      });
      setLearningDrafts((current) => [draft, ...current]);
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
    }
  };

  const createOcrDraft = async (page: OcrPage) => {
    const book = books.find((item) => item.id === page.bookId);
    setStudyBusy(true);
    setStudyError(null);
    try {
      const draft = await invoke<LearningDraft>("create_learning_draft", {
        sourceKind: "ocr_page",
        sourceId: page.id,
        bookRelativePath: book?.relativePath ?? null,
        pageIndex: page.pageIndex,
        front: page.text,
        back: "Bổ sung cách đọc và nghĩa đã kiểm chứng.",
        tags: ["japanese", "ocr"],
      });
      setLearningDrafts((current) => [draft, ...current]);
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
    }
  };

  const approveLearningDraft = async (draftId: string) => {
    setStudyBusy(true);
    setStudyError(null);
    try {
      const approved = await invoke<LearningDraft>("approve_learning_draft", {
        draftId,
      });
      setLearningDrafts((current) =>
        current.map((draft) => (draft.id === approved.id ? approved : draft)),
      );
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
    }
  };

  const exportAnki = async () => {
    const selectedPath = await save({
      defaultPath: "book-library-anki.tsv",
      filters: [{ name: "Anki TSV", extensions: ["tsv"] }],
    });
    if (typeof selectedPath !== "string") return;
    setStudyBusy(true);
    setStudyError(null);
    try {
      await invoke<{ exported: number }>("export_anki_tsv", { selectedPath });
      setLearningDrafts(await invoke("list_learning_drafts"));
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
    }
  };

  const generateAiDraft = async () => {
    setStudyBusy(true);
    setStudyError(null);
    try {
      const draft = await invoke<AiDraft>("generate_ai_draft", {
        kind: aiKind,
        context: aiContext,
      });
      setAiDrafts((current) => [draft, ...current]);
    } catch (error) {
      setStudyError(desktopError(error));
    } finally {
      setStudyBusy(false);
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
                item === "Library" ||
                item === "Study" ||
                item === "Notes" ||
                item === "Search";
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
                  (setReaderPage(null),
                  setActiveSection(
                    item as "Library" | "Study" | "Notes" | "Search",
                  ))
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
              {readerPage ? (
                <StudyReader
                  busy={readerBusy || studyBusy}
                  ankiEnabled={studyModules.some(
                    (module) => module.id === "anki" && module.enabled,
                  )}
                  dictionaryEnabled={studyModules.some(
                    (module) => module.id === "dictionary" && module.enabled,
                  )}
                  dictionaryLookup={dictionaryLookup}
                  dictionaryQuery={dictionaryQuery}
                  error={readerError ?? studyError}
                  ocrEnabled={studyModules.some(
                    (module) =>
                      module.id === "ocr" &&
                      module.enabled &&
                      module.available,
                  )}
                  ocrPage={
                    ocrPages.find(
                      (page) =>
                        page.bookId === readerPage.bookId &&
                        page.pageIndex === readerPage.pageIndex,
                    ) ?? null
                  }
                  onBack={() => {
                    setReaderPage(null);
                    setReaderError(null);
                  }}
                  onCreateCard={(entry) => void createDictionaryDraft(entry)}
                  onDictionaryQueryChange={setDictionaryQuery}
                  onLookup={(query) => void lookupJapanese(query)}
                  onNavigate={(pageIndex) => {
                    const book = books.find(
                      (item) => item.id === readerPage.bookId,
                    );
                    if (book) void openStudyReader(book, pageIndex);
                  }}
                  onOpenFolder={() => {
                    const book = books.find(
                      (item) => item.id === readerPage.bookId,
                    );
                    if (book) void openBookLocation(book);
                  }}
                  onRunOcr={() => void runReaderOcr()}
                  page={readerPage}
                />
              ) : activeSection === "Study" ? (
                <StudyWorkspace
                  aiContext={aiContext}
                  aiDrafts={aiDrafts}
                  aiKind={aiKind}
                  books={books}
                  busy={studyBusy}
                  dictionaryLookup={dictionaryLookup}
                  dictionaryQuery={dictionaryQuery}
                  error={studyError}
                  notice={studyNotice}
                  learningDrafts={learningDrafts}
                  modules={studyModules}
                  ocrBookId={ocrBookId}
                  ocrPageNumber={ocrPageNumber}
                  ocrPages={ocrPages}
                  onAiContextChange={setAiContext}
                  onAiKindChange={setAiKind}
                  onApproveDraft={(draftId) =>
                    void approveLearningDraft(draftId)
                  }
                  onCancelOcr={() => void invoke("cancel_page_ocr")}
                  onClearHistory={() =>
                    void invoke("clear_dictionary_history").catch((error) =>
                      setStudyError(desktopError(error)),
                    )
                  }
                  onCreateDictionaryDraft={(entry) =>
                    void createDictionaryDraft(entry)
                  }
                  onCreateOcrDraft={(page) => void createOcrDraft(page)}
                  onDictionaryQueryChange={setDictionaryQuery}
                  onExport={() => void exportAnki()}
                  onGenerateAi={() => void generateAiDraft()}
                  onImportDictionary={() => void importDictionaryPackage()}
                  onLookup={(query) => void lookupJapanese(query)}
                  onOcrBookChange={setOcrBookId}
                  onOcrPageChange={setOcrPageNumber}
                  onRunOcr={() => void runPageOcr()}
                  onSaveHistoryChange={setSaveLookupHistory}
                  onToggleModule={(module) => void toggleStudyModule(module)}
                  onTrimOcrPage={(page) => void trimOcrPage(page)}
                  saveLookupHistory={saveLookupHistory}
                  trustedModules={trustedModules}
                />
              ) : activeSection === "Search" ? (
                <GlobalSearchWorkspace
                  busy={globalSearchBusy}
                  diagnostics={searchDiagnostics}
                  error={globalSearchError}
                  onOpenResult={(result) => {
                    if (result.sourceKind === "book") {
                      setActiveSection("Library");
                      void openBookDetail(result.sourceId);
                    } else if (result.sourceKind === "note") {
                      setActiveSection("Notes");
                      void readNote(result.sourceId);
                    } else {
                      setActiveSection("Study");
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
                  coverProgress={coverProgress}
                  detail={selectedBookDetail}
                  error={bookDetailError}
                  onBack={closeBookDetail}
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
                  onReadStudy={() =>
                    void openStudyReader(selectedBookDetail)
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
                  onReadStudy={(book) => void openStudyReader(book)}
                  onRepair={() => void runScan("repair_library")}
                  onRelinkBook={(book) => void relinkBook(book)}
                  onRescan={() => void runScan("rescan_library")}
                  onSearchChange={setSearchQuery}
                  onViewChange={setView}
                  openingBookId={openingBookId}
                  progress={scanProgress}
                  searchQuery={searchQuery}
                  summary={scanSummary}
                  summaryKind={scanSummaryKind}
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

export function StudyReader({
  ankiEnabled,
  busy,
  dictionaryEnabled,
  dictionaryLookup,
  dictionaryQuery,
  error,
  ocrEnabled,
  ocrPage,
  onBack,
  onCreateCard,
  onDictionaryQueryChange,
  onLookup,
  onNavigate,
  onOpenFolder,
  onRunOcr,
  page,
}: {
  ankiEnabled: boolean;
  busy: boolean;
  dictionaryEnabled: boolean;
  dictionaryLookup: DictionaryLookup | null;
  dictionaryQuery: string;
  error: DesktopError | null;
  ocrEnabled: boolean;
  ocrPage: OcrPage | null;
  onBack: () => void;
  onCreateCard: (entry: DictionaryEntry) => void;
  onDictionaryQueryChange: (query: string) => void;
  onLookup: (query: string) => void;
  onNavigate: (pageIndex: number) => void;
  onOpenFolder: () => void;
  onRunOcr: () => void;
  page: StudyReaderPage;
}) {
  const [zoom, setZoom] = useState(100);
  const [pageDraft, setPageDraft] = useState(page.pageIndex + 1);
  const [dictionaryCollapsed, setDictionaryCollapsed] = useState(false);
  const transcriptRef = useRef<HTMLDivElement>(null);
  const canGoBack = page.pageIndex > 0;
  const canGoForward = page.pageIndex + 1 < page.pageCount;

  useEffect(() => {
    setPageDraft(page.pageIndex + 1);
  }, [page.pageIndex]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (
        event.target instanceof HTMLInputElement ||
        event.target instanceof HTMLTextAreaElement
      ) {
        return;
      }
      if (event.key === "ArrowLeft" && canGoBack && !busy) {
        onNavigate(page.pageIndex - 1);
      } else if (event.key === "ArrowRight" && canGoForward && !busy) {
        onNavigate(page.pageIndex + 1);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [busy, canGoBack, canGoForward, onNavigate, page.pageIndex]);

  const lookupSelection = () => {
    if (!dictionaryEnabled || busy || !transcriptRef.current) return;
    const selection = window.getSelection();
    if (
      !selection?.anchorNode ||
      !selection.focusNode ||
      !transcriptRef.current.contains(selection.anchorNode) ||
      !transcriptRef.current.contains(selection.focusNode)
    ) {
      return;
    }
    const query = normalizeLookupSelection(selection.toString());
    if (!query) return;
    onDictionaryQueryChange(query);
    onLookup(query);
  };

  return (
    <div className="reader-shell -mx-5 -my-7 md:-mx-7">
      <header className="reader-toolbar sticky top-16 z-30 flex min-h-16 flex-wrap items-center gap-3 border-b border-stone-800 bg-stone-950/95 px-5 py-3 backdrop-blur md:px-7">
        <button
          className="rounded-lg border border-stone-700 px-3 py-2 text-sm hover:border-amber-500"
          onClick={onBack}
          type="button"
        >
          ← Library
        </button>
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-sm font-semibold text-stone-100">
            {page.bookTitle}
          </h1>
          <p className="mt-0.5 text-xs text-stone-500">
            Page {page.pageIndex + 1} of {page.pageCount}
          </p>
        </div>
        <div className="flex items-center gap-1 rounded-lg border border-stone-700 p-1">
          <button
            aria-label="Zoom out"
            className="rounded px-2 py-1 text-sm hover:bg-stone-800"
            onClick={() => setZoom((value) => Math.max(40, value - 10))}
            type="button"
          >
            −
          </button>
          <span className="w-12 text-center text-xs text-stone-400">
            {zoom}%
          </span>
          <button
            aria-label="Zoom in"
            className="rounded px-2 py-1 text-sm hover:bg-stone-800"
            onClick={() => setZoom((value) => Math.min(220, value + 10))}
            type="button"
          >
            +
          </button>
        </div>
        <button
          className="rounded-lg border border-stone-700 px-3 py-2 text-sm text-stone-300 hover:border-amber-500"
          onClick={onOpenFolder}
          type="button"
        >
          Open externally
        </button>
        <button
          className="rounded-lg border border-stone-700 px-3 py-2 text-sm text-stone-300"
          onClick={() => setDictionaryCollapsed((value) => !value)}
          type="button"
        >
          {dictionaryCollapsed ? "Show dictionary" : "Hide dictionary"}
        </button>
      </header>

      {error && (
        <div className="px-5 pt-4 md:px-7">
          <ErrorPanel error={error} />
        </div>
      )}

      <div
        className={`reader-layout grid min-h-[calc(100dvh-8rem)] ${
          dictionaryCollapsed
            ? "grid-cols-1"
            : "xl:grid-cols-[minmax(0,1fr)_clamp(360px,28vw,500px)]"
        }`}
      >
        <section className="min-w-0 bg-stone-900/30">
          <div className="reader-canvas overflow-auto p-4 md:p-7">
            <div
              className="mx-auto w-fit transition-[width] duration-150"
              style={{ width: `${zoom}%`, maxWidth: zoom <= 100 ? "100%" : "none" }}
            >
              <img
                alt={`${page.bookTitle}, page ${page.pageIndex + 1}`}
                className="block h-auto w-full rounded-sm bg-white shadow-2xl"
                draggable={false}
                height={page.height}
                src={page.imageDataUrl}
                width={page.width}
              />
            </div>
          </div>
        </section>

        {!dictionaryCollapsed && (
          <aside className="reader-dictionary border-l border-stone-800 bg-stone-950 p-5 xl:sticky xl:top-32 xl:h-[calc(100dvh-8rem)] xl:overflow-auto">
            <div className="flex items-center justify-between gap-3">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.18em] text-amber-400">
                  Offline dictionary
                </p>
                <h2 className="mt-1 text-xl font-semibold text-white">
                  Japanese → Vietnamese
                </h2>
              </div>
              <button
                aria-label="Collapse dictionary"
                className="hidden rounded-lg border border-stone-700 px-2 py-1 text-stone-400 xl:block"
                onClick={() => setDictionaryCollapsed(true)}
                type="button"
              >
                →
              </button>
            </div>
            <form
              className="mt-5 flex gap-2"
              onSubmit={(event) => {
                event.preventDefault();
                onLookup(dictionaryQuery);
              }}
            >
              <input
                aria-label="Dictionary query"
                className="min-w-0 flex-1 rounded-lg border border-stone-700 bg-stone-900 px-3 py-2 text-stone-100"
                disabled={!dictionaryEnabled || busy}
                onChange={(event) => onDictionaryQueryChange(event.target.value)}
                placeholder="Bôi đen chữ hoặc nhập từ…"
                value={dictionaryQuery}
              />
              <button
                className="rounded-lg bg-amber-400 px-4 py-2 font-semibold text-stone-950 disabled:opacity-40"
                disabled={!dictionaryEnabled || busy || !dictionaryQuery.trim()}
                type="submit"
              >
                Look up
              </button>
            </form>
            <section className="mt-5 border-t border-stone-800 pt-5">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <h3 className="text-sm font-semibold text-stone-200">
                    Selectable page text
                  </h3>
                  <p className="mt-1 text-xs leading-5 text-stone-500">
                    Select a word or phrase below for instant lookup.
                  </p>
                </div>
                {!ocrPage && (
                  <button
                    className="shrink-0 rounded-lg bg-amber-400 px-3 py-2 text-xs font-semibold text-stone-950 disabled:opacity-40"
                    disabled={!ocrEnabled || busy}
                    onClick={onRunOcr}
                    type="button"
                  >
                    {busy ? "Recognizing…" : "OCR page"}
                  </button>
                )}
              </div>
              {ocrPage ? (
                <div
                  className="reader-transcript mt-3 max-h-64 select-text overflow-auto whitespace-pre-wrap rounded-lg border border-stone-700 bg-stone-900/70 p-4 text-base leading-8 text-stone-100"
                  onMouseUp={lookupSelection}
                  ref={transcriptRef}
                >
                  {ocrPage.text}
                </div>
              ) : (
                <p className="mt-3 rounded-lg border border-dashed border-stone-700 p-4 text-sm leading-6 text-stone-500">
                  {ocrEnabled
                    ? "This page has no saved OCR text yet."
                    : "Enable the local OCR module in Study to recognize this page."}
                </p>
              )}
            </section>
            {!dictionaryEnabled ? (
              <p className="mt-4 rounded-lg border border-dashed border-stone-700 p-4 text-sm text-stone-500">
                Enable Dictionary in the Study workspace to use instant lookup.
              </p>
            ) : (
              <DictionaryResults
                ankiEnabled={ankiEnabled}
                busy={busy}
                lookup={dictionaryLookup}
                onCreateCard={onCreateCard}
                onLookup={onLookup}
              />
            )}
          </aside>
        )}
      </div>

      <footer className="reader-pager sticky bottom-0 z-20 flex items-center justify-center gap-4 border-t border-stone-800 bg-stone-950/95 px-4 py-3 backdrop-blur">
        <button
          className="rounded-lg border border-stone-700 px-4 py-2 text-sm disabled:opacity-30"
          disabled={!canGoBack || busy}
          onClick={() => onNavigate(page.pageIndex - 1)}
          type="button"
        >
          ← Previous
        </button>
        <form
          className="flex items-center gap-2 text-sm text-stone-400"
          onSubmit={(event) => {
            event.preventDefault();
            const target = Math.min(
              page.pageCount,
              Math.max(1, Math.trunc(pageDraft)),
            );
            if (target !== page.pageIndex + 1) onNavigate(target - 1);
          }}
        >
          <input
            aria-label="Go to page"
            className="w-16 rounded-md border border-stone-700 bg-stone-900 px-2 py-1 text-center text-stone-100"
            max={page.pageCount}
            min={1}
            onChange={(event) => setPageDraft(Number(event.target.value))}
            type="number"
            value={pageDraft}
          />
          <span>/ {page.pageCount}</span>
        </form>
        <button
          className="rounded-lg border border-stone-700 px-4 py-2 text-sm disabled:opacity-30"
          disabled={!canGoForward || busy}
          onClick={() => onNavigate(page.pageIndex + 1)}
          type="button"
        >
          Next →
        </button>
      </footer>
    </div>
  );
}

function DictionaryResults({
  ankiEnabled,
  busy,
  lookup,
  onCreateCard,
  onLookup,
}: {
  ankiEnabled: boolean;
  busy: boolean;
  lookup: DictionaryLookup | null;
  onCreateCard: (entry: DictionaryEntry) => void;
  onLookup: (query: string) => void;
}) {
  if (!lookup) {
    return (
      <p className="mt-5 text-sm leading-6 text-stone-500">
        Select Japanese text on the current page. Results will appear here
        without covering the book.
      </p>
    );
  }
  return (
    <div className="mt-5 space-y-3">
      {lookup.tokens.length > 0 && (
        <div className="flex flex-wrap gap-2 border-b border-stone-800 pb-4">
          {lookup.tokens.map((token) => (
            <button
              className="rounded-full border border-stone-700 px-3 py-1 text-sm text-amber-300 hover:border-amber-500"
              key={`${token.start}-${token.end}-${token.surface}`}
              onClick={() => onLookup(token.surface)}
              type="button"
            >
              {token.surface}
            </button>
          ))}
        </div>
      )}
      {lookup.entries.length === 0 ? (
        <p className="text-sm text-stone-400">
          No entry in the installed dictionaries.
        </p>
      ) : (
        lookup.entries.map((entry) => (
          <article
            className="rounded-lg border border-stone-800 bg-stone-900/60 p-4"
            key={entry.id}
          >
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="text-xl font-semibold text-white">
                  {entry.expression}
                </p>
                <p className="text-sm text-amber-300">{entry.reading}</p>
              </div>
              <button
                className="text-xs text-amber-400 disabled:text-stone-600"
                disabled={!ankiEnabled || busy}
                onClick={() => onCreateCard(entry)}
                type="button"
              >
                Make card
              </button>
            </div>
            <p className="mt-3 text-sm leading-6 text-stone-200">
              {entry.meaningVi}
            </p>
            <p className="mt-2 text-xs text-stone-500">
              {entry.partOfSpeech}
              {entry.hanViet ? ` · Hán–Việt: ${entry.hanViet}` : ""}
            </p>
          </article>
        ))
      )}
    </div>
  );
}

export function StudyWorkspace({
  aiContext,
  aiDrafts,
  aiKind,
  books,
  busy,
  dictionaryLookup,
  dictionaryQuery,
  error,
  notice,
  learningDrafts,
  modules,
  ocrBookId,
  ocrPageNumber,
  ocrPages,
  onAiContextChange,
  onAiKindChange,
  onApproveDraft,
  onCancelOcr,
  onClearHistory,
  onCreateDictionaryDraft,
  onCreateOcrDraft,
  onDictionaryQueryChange,
  onExport,
  onGenerateAi,
  onImportDictionary,
  onLookup,
  onOcrBookChange,
  onOcrPageChange,
  onRunOcr,
  onSaveHistoryChange,
  onToggleModule,
  onTrimOcrPage,
  saveLookupHistory,
  trustedModules,
}: {
  aiContext: string;
  aiDrafts: AiDraft[];
  aiKind: string;
  books: Book[];
  busy: boolean;
  dictionaryLookup: DictionaryLookup | null;
  dictionaryQuery: string;
  error: DesktopError | null;
  notice: string | null;
  learningDrafts: LearningDraft[];
  modules: StudyModule[];
  ocrBookId: string;
  ocrPageNumber: number;
  ocrPages: OcrPage[];
  onAiContextChange: (value: string) => void;
  onAiKindChange: (value: string) => void;
  onApproveDraft: (draftId: string) => void;
  onCancelOcr: () => void;
  onClearHistory: () => void;
  onCreateDictionaryDraft: (entry: DictionaryEntry) => void;
  onCreateOcrDraft: (page: OcrPage) => void;
  onDictionaryQueryChange: (query: string) => void;
  onExport: () => void;
  onGenerateAi: () => void;
  onImportDictionary: () => void;
  onLookup: (query: string) => void;
  onOcrBookChange: (bookId: string) => void;
  onOcrPageChange: (page: number) => void;
  onRunOcr: () => void;
  onSaveHistoryChange: (enabled: boolean) => void;
  onToggleModule: (module: StudyModule) => void;
  onTrimOcrPage: (page: OcrPage) => void;
  saveLookupHistory: boolean;
  trustedModules: TrustedModule[];
}) {
  const dictionaryEnabled = modules.some(
    (module) => module.id === "dictionary" && module.enabled,
  );
  const ocr = modules.find((module) => module.id === "ocr");
  const ankiEnabled = modules.some(
    (module) => module.id === "anki" && module.enabled,
  );
  const ai = modules.find((module) => module.id === "ai");
  const lookupSelectedOcrText = (container: HTMLElement) => {
    if (!dictionaryEnabled || busy) return;
    const selection = window.getSelection();
    if (
      !selection?.anchorNode ||
      !selection.focusNode ||
      !container.contains(selection.anchorNode) ||
      !container.contains(selection.focusNode)
    ) {
      return;
    }
    const query = normalizeLookupSelection(selection.toString());
    if (!query) return;
    onDictionaryQueryChange(query);
    onLookup(query);
  };

  return (
    <div className="w-full">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.22em] text-amber-400">
            Japanese study
          </p>
          <h1 className="mt-2 text-3xl font-semibold text-white">
            Read, recognize, remember
          </h1>
          <p className="mt-2 max-w-2xl text-sm leading-6 text-stone-400">
            Offline dictionary lookup, explicit page OCR, reviewable learning
            drafts, and Anki-compatible TSV export. Optional modules never alter
            source books.
          </p>
        </div>
        {busy && <p className="text-sm text-amber-300">Working locally…</p>}
      </div>

      {error && <div className="mt-5"><ErrorPanel error={error} /></div>}
      {notice && !error && (
        <p className="mt-5 rounded-xl border border-emerald-700/50 bg-emerald-950/30 px-4 py-3 text-sm text-emerald-300">
          {notice}
        </p>
      )}

      <section className="mt-7 grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
        {modules.map((module) => (
          <button
            className={`rounded-xl border p-4 text-left transition ${
              module.enabled
                ? "border-amber-500/60 bg-amber-500/10"
                : "border-stone-800 bg-stone-900/40 hover:border-stone-700"
            }`}
            disabled={busy}
            key={module.id}
            onClick={() => onToggleModule(module)}
            type="button"
          >
            <span className="block text-sm font-semibold capitalize text-stone-100">
              {module.id.replace("_", " ")}
            </span>
            <span className="mt-1 block text-xs text-stone-400">
              {module.status}
              {!module.available ? " · runtime missing" : ""}
            </span>
          </button>
        ))}
      </section>

      <div className="mt-7 grid items-start gap-6 xl:grid-cols-[minmax(420px,0.85fr)_minmax(620px,1.35fr)]">
        <section className="rounded-xl border border-stone-800 bg-stone-900/40 p-5 xl:sticky xl:top-24 xl:max-h-[calc(100dvh-7rem)] xl:overflow-auto">
          <div className="flex items-center justify-between gap-3">
            <h2 className="text-lg font-semibold text-white">
              Offline Japanese dictionary
            </h2>
            <button
              className="text-xs text-amber-400 hover:text-amber-300 disabled:text-stone-600"
              disabled={!dictionaryEnabled || busy}
              onClick={onImportDictionary}
              type="button"
            >
              Import dictionary ZIP/TSV
            </button>
          </div>
          <form
            className="mt-4 flex gap-2"
            onSubmit={(event) => {
              event.preventDefault();
              onLookup(dictionaryQuery);
            }}
          >
            <input
              className="min-w-0 flex-1 rounded-lg border border-stone-700 bg-stone-950 px-3 py-2 text-stone-100"
              disabled={!dictionaryEnabled || busy}
              onChange={(event) => onDictionaryQueryChange(event.target.value)}
              placeholder="日本語、読む、にほん…"
              value={dictionaryQuery}
            />
            <button
              className="rounded-lg bg-amber-500 px-4 py-2 font-semibold text-stone-950 disabled:opacity-40"
              disabled={!dictionaryEnabled || busy || !dictionaryQuery.trim()}
              type="submit"
            >
              Look up
            </button>
          </form>
          {!dictionaryEnabled && (
            <p className="mt-3 text-sm text-stone-500">
              Enable the dictionary module above to begin.
            </p>
          )}
          {dictionaryEnabled && (
            <div className="mt-3 flex flex-wrap items-center gap-4 text-xs text-stone-400">
              <label className="flex items-center gap-2">
                <input
                  checked={saveLookupHistory}
                  onChange={(event) =>
                    onSaveHistoryChange(event.target.checked)
                  }
                  type="checkbox"
                />
                Save lookup history
              </label>
              <button
                className="text-stone-500 hover:text-stone-300"
                onClick={onClearHistory}
                type="button"
              >
                Clear history
              </button>
            </div>
          )}
          {dictionaryLookup && (
            <div className="mt-5 space-y-3">
              {dictionaryLookup.tokens.length > 0 && (
                <div className="flex flex-wrap gap-2 border-b border-stone-800 pb-4">
                  {dictionaryLookup.tokens.map((token) => (
                    <button
                      className="rounded-full border border-stone-700 px-3 py-1 text-sm text-amber-300 hover:border-amber-500"
                      key={`${token.start}-${token.end}-${token.surface}`}
                      onClick={() => onLookup(token.surface)}
                      type="button"
                    >
                      {token.surface}
                    </button>
                  ))}
                </div>
              )}
              {dictionaryLookup.entries.length === 0 ? (
                <p className="text-sm text-stone-400">
                  No entry in the installed starter dictionary.
                </p>
              ) : (
                dictionaryLookup.entries.map((entry) => (
                  <article
                    className="rounded-lg border border-stone-800 bg-stone-950/70 p-4"
                    key={entry.id}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <p className="text-xl font-semibold text-white">
                          {entry.expression}
                        </p>
                        <p className="text-sm text-amber-300">{entry.reading}</p>
                      </div>
                      <button
                        className="text-xs text-amber-400 hover:text-amber-300 disabled:text-stone-600"
                        disabled={!ankiEnabled || busy}
                        onClick={() => onCreateDictionaryDraft(entry)}
                        type="button"
                      >
                        Make card
                      </button>
                    </div>
                    <p className="mt-3 text-sm text-stone-200">
                      {entry.meaningVi}
                    </p>
                    <p className="mt-1 text-xs text-stone-500">
                      {entry.partOfSpeech}
                      {entry.hanViet ? ` · Hán–Việt: ${entry.hanViet}` : ""}
                    </p>
                  </article>
                ))
              )}
            </div>
          )}
        </section>

        <section className="rounded-xl border border-stone-800 bg-stone-900/40 p-6 lg:p-7">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <h2 className="text-xl font-semibold text-white">
                Explicit page OCR
              </h2>
              <p className="mt-2 text-base leading-6 text-stone-400">
                {!ocr?.enabled
                  ? "OCR is off. Enable it here before choosing a book."
                  : !ocr.available
                    ? "Tesseract was not detected. Restart the app after installing it."
                    : "Tesseract with Japanese language data is ready."}
              </p>
              {dictionaryEnabled && (
                <p className="mt-1 text-sm text-amber-400/70">
                  Select Japanese text below to look it up instantly.
                </p>
              )}
            </div>
            {ocr && !ocr.enabled && (
              <button
                className="rounded-lg border border-amber-500/70 px-4 py-2 text-sm font-semibold text-amber-300 hover:bg-amber-500/10 disabled:opacity-40"
                disabled={busy}
                onClick={() => onToggleModule(ocr)}
                type="button"
              >
                Enable OCR
              </button>
            )}
          </div>
          <div className="mt-6 grid gap-4 md:grid-cols-[minmax(0,1fr)_120px_150px]">
            <select
              aria-label="Book to recognize"
              className="min-h-14 min-w-0 rounded-lg border border-stone-700 bg-stone-950 px-4 py-3 text-base"
              disabled={!ocr?.enabled || busy}
              onChange={(event) => onOcrBookChange(event.target.value)}
              value={ocrBookId}
            >
              <option value="">Choose a book…</option>
              {books
                .filter((book) => book.status === "available")
                .map((book) => (
                  <option key={book.id} value={book.id}>
                    {book.title}
                  </option>
                ))}
            </select>
            <input
              aria-label="Page number"
              className="min-h-14 rounded-lg border border-stone-700 bg-stone-950 px-4 py-3 text-base"
              disabled={!ocr?.enabled || busy}
              min={1}
              onChange={(event) =>
                onOcrPageChange(Math.max(1, Number(event.target.value)))
              }
              type="number"
              value={ocrPageNumber}
            />
            {busy && ocr?.enabled ? (
              <button
                className="min-h-14 whitespace-nowrap rounded-lg border border-red-500/70 px-5 py-3 text-base font-semibold text-red-300 active:translate-y-px"
                onClick={onCancelOcr}
                type="button"
              >
                Cancel OCR
              </button>
            ) : (
              <button
                className="min-h-14 whitespace-nowrap rounded-lg bg-amber-500 px-5 py-3 text-base font-semibold text-stone-950 active:translate-y-px disabled:opacity-40"
                disabled={!ocr?.enabled || !ocr.available || !ocrBookId}
                onClick={onRunOcr}
                type="button"
              >
                OCR page
              </button>
            )}
          </div>
          <div className="mt-7 max-h-[clamp(620px,72dvh,980px)] space-y-4 overflow-auto pr-1">
            {ocrPages.map((page) => (
              <article
                className="rounded-xl border border-stone-800 bg-stone-950/70 p-5 sm:p-6"
                key={page.id}
              >
                <div className="grid gap-5 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
                  <div className="min-w-0">
                    <p className="text-base font-semibold leading-7 text-white">
                      {page.bookTitle} · page {page.pageIndex + 1}
                    </p>
                    <p className="mt-1 text-sm text-stone-500">
                      {Math.round(page.confidence * 100)}% · {page.providerId} ·{" "}
                      {page.providerVersion}
                    </p>
                  </div>
                  <div className="flex flex-wrap gap-3">
                    <button
                      className="whitespace-nowrap rounded-lg border border-stone-700 px-4 py-2 text-sm font-semibold text-stone-300 hover:bg-stone-800 active:translate-y-px disabled:opacity-40"
                      disabled={busy}
                      onClick={() => onTrimOcrPage(page)}
                      type="button"
                    >
                      Trim spaces
                    </button>
                    <button
                      className="whitespace-nowrap rounded-lg border border-amber-500/60 px-4 py-2 text-sm font-semibold text-amber-400 hover:bg-amber-500/10 active:translate-y-px disabled:opacity-40"
                      disabled={!dictionaryEnabled}
                      onClick={() => onLookup(page.text)}
                      type="button"
                    >
                      Look up
                    </button>
                    <button
                      className="whitespace-nowrap rounded-lg border border-amber-500/60 px-4 py-2 text-sm font-semibold text-amber-400 hover:bg-amber-500/10 active:translate-y-px disabled:opacity-40"
                      disabled={!ankiEnabled}
                      onClick={() => onCreateOcrDraft(page)}
                      type="button"
                    >
                      Make card
                    </button>
                  </div>
                </div>
                <p
                  className="mt-5 cursor-text select-text whitespace-pre-wrap rounded-lg text-base leading-8 text-stone-200 outline-none ring-amber-500/60 selection:bg-amber-400/30 selection:text-inherit focus-visible:ring-2"
                  onPointerUp={(event) =>
                    lookupSelectedOcrText(event.currentTarget)
                  }
                  tabIndex={0}
                  title="Select Japanese text to look it up"
                >
                  {page.text}
                </p>
              </article>
            ))}
          </div>
        </section>
      </div>

      <div className="mt-6 grid gap-6 xl:grid-cols-2">
        <section className="rounded-xl border border-stone-800 bg-stone-900/40 p-5">
          <div className="flex items-center justify-between gap-3">
            <h2 className="text-lg font-semibold text-white">
              Learning drafts
            </h2>
            <button
              className="rounded-lg border border-amber-500/70 px-3 py-2 text-sm text-amber-300 disabled:opacity-40"
              disabled={
                busy ||
                !ankiEnabled ||
                !learningDrafts.some((draft) => draft.status === "approved")
              }
              onClick={onExport}
              type="button"
            >
              Export approved TSV
            </button>
          </div>
          <div className="mt-4 space-y-3">
            {learningDrafts.length === 0 && (
              <p className="text-sm text-stone-500">No learning drafts yet.</p>
            )}
            {learningDrafts.map((draft) => (
              <article
                className="rounded-lg border border-stone-800 bg-stone-950/70 p-4"
                key={draft.id}
              >
                <div className="flex justify-between gap-3">
                  <p className="font-semibold text-white">{draft.front}</p>
                  <span className="text-xs uppercase tracking-wide text-stone-500">
                    {draft.status}
                  </span>
                </div>
                <p className="mt-2 whitespace-pre-wrap text-sm text-stone-300">
                  {draft.back}
                </p>
                {draft.status === "draft" && (
                  <button
                    className="mt-3 text-xs text-amber-400"
                    onClick={() => onApproveDraft(draft.id)}
                    type="button"
                  >
                    Approve for export
                  </button>
                )}
              </article>
            ))}
          </div>
        </section>

        <section className="rounded-xl border border-stone-800 bg-stone-900/40 p-5">
          <h2 className="text-lg font-semibold text-white">
            Optional study assistant
          </h2>
          <div className="mt-4 flex gap-2">
            <select
              className="rounded-lg border border-stone-700 bg-stone-950 px-3 py-2"
              disabled={!ai?.enabled || busy}
              onChange={(event) => onAiKindChange(event.target.value)}
              value={aiKind}
            >
              <option value="explain">Explain</option>
              <option value="translate">Translate draft</option>
              <option value="summarize">Summarize</option>
              <option value="flashcard">Flashcard draft</option>
            </select>
            <button
              className="rounded-lg bg-amber-500 px-4 py-2 font-semibold text-stone-950 disabled:opacity-40"
              disabled={!ai?.enabled || busy || !aiContext.trim()}
              onClick={onGenerateAi}
              type="button"
            >
              Generate draft
            </button>
          </div>
          <textarea
            className="mt-3 min-h-28 w-full rounded-lg border border-stone-700 bg-stone-950 p-3 text-sm"
            disabled={!ai?.enabled || busy}
            onChange={(event) => onAiContextChange(event.target.value)}
            placeholder="Context is processed locally by the built-in draft provider."
            value={aiContext}
          />
          <div className="mt-4 space-y-3">
            {aiDrafts.map((draft) => (
              <article
                className="rounded-lg border border-stone-800 bg-stone-950/70 p-4"
                key={draft.id}
              >
                <p className="text-xs uppercase tracking-wide text-amber-400">
                  {draft.kind} draft
                </p>
                <p className="mt-2 whitespace-pre-wrap text-sm text-stone-300">
                  {draft.content}
                </p>
              </article>
            ))}
          </div>
        </section>
      </div>

      <details className="mt-6 rounded-xl border border-stone-800 bg-stone-900/40 p-5">
        <summary className="cursor-pointer text-sm font-semibold text-stone-200">
          Trusted module manifests
        </summary>
        <div className="mt-4 grid gap-3 md:grid-cols-3">
          {trustedModules.map((module) => (
            <article
              className="rounded-lg border border-stone-800 bg-stone-950/70 p-4 text-xs"
              key={module.id}
            >
              <p className="font-semibold text-stone-200">{module.id}</p>
              <p className="mt-1 text-stone-500">
                v{module.version} · {module.compatible ? "compatible" : "unavailable"}
              </p>
              <p className="mt-2 text-stone-400">
                {module.capabilities.join(", ")}
              </p>
            </article>
          ))}
        </div>
      </details>
    </div>
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
          <option value="ocr">OCR text</option>
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
  coverProgress,
  detail,
  error,
  onBack,
  onEditTitle,
  onForceCover,
  onNewNote,
  onOpenFolder,
  onOpenNote,
  onReadStudy,
  onSave,
}: {
  busy: boolean;
  coverProgress: string[];
  detail: BookDetail;
  error: DesktopError | null;
  onBack: () => void;
  onEditTitle: () => void;
  onForceCover: () => void;
  onNewNote: () => void;
  onOpenFolder: () => void;
  onOpenNote: (noteId: string) => void;
  onReadStudy: () => void;
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
          {coverProgress.length > 0 && (
            <div
              aria-live="polite"
              className="mt-3 rounded-lg border border-stone-800 bg-stone-900/60 p-3"
            >
              <p className="text-xs font-medium text-stone-300">
                Cover generation log
              </p>
              <ol className="mt-2 space-y-1 text-xs leading-5 text-stone-500">
                {coverProgress.map((entry, index) => (
                  <li key={`${index}-${entry}`}>
                    {index + 1}. {entry}
                  </li>
                ))}
              </ol>
            </div>
          )}
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
                  className="rounded-lg bg-amber-400 px-4 py-2 text-sm font-semibold text-stone-950 disabled:opacity-40"
                  disabled={
                    busy ||
                    !["available", "unavailable"].includes(detail.status)
                  }
                  onClick={onReadStudy}
                  type="button"
                >
                  Read &amp; Study
                </button>
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
  onReadStudy,
  onRepair,
  onRelinkBook,
  onRescan,
  onSearchChange,
  onViewChange,
  openingBookId,
  progress,
  searchQuery,
  summary,
  summaryKind,
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
  onReadStudy: (book: Book) => void;
  onRepair: () => void;
  onRelinkBook: (book: Book) => void;
  onRescan: () => void;
  onSearchChange: (query: string) => void;
  onViewChange: (view: "grid" | "list") => void;
  openingBookId: string | null;
  progress: ScanProgress | null;
  searchQuery: string;
  summary: ScanSummary | null;
  summaryKind: ActiveScan | null;
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
              {activeScan === "repair"
                ? `Processed ${progress.visitedEntries.toLocaleString()} of ${progress.discoveredBooks.toLocaleString()} missing covers`
                : `Scanned ${progress.visitedEntries.toLocaleString()} entries · found ${progress.discoveredBooks.toLocaleString()} books`}
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
      {summary && <SummaryPanel kind={summaryKind} summary={summary} />}
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
              onReadStudy={() => onReadStudy(book)}
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
  onReadStudy,
  onRelink,
  view,
}: {
  book: Book;
  isOpening: boolean;
  onEdit: () => void;
  onDetail: () => void;
  onOpen: () => void;
  onReadStudy: () => void;
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
          onReadStudy={onReadStudy}
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
          onReadStudy={onReadStudy}
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
  onReadStudy,
  onRelink,
  overlay = false,
}: {
  book: Book;
  canOpen: boolean;
  isOpening: boolean;
  onEdit: () => void;
  onOpen: () => void;
  onReadStudy: () => void;
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
          className="block w-full rounded-md px-3 py-2 text-left text-sm font-medium text-amber-300 enabled:hover:bg-stone-800 disabled:text-stone-600"
          disabled={!canOpen || isOpening || book.status === "missing"}
          onClick={onReadStudy}
          type="button"
        >
          Read &amp; Study
        </button>
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

function SummaryPanel({
  kind,
  summary,
}: {
  kind: ActiveScan | null;
  summary: ScanSummary;
}) {
  if (kind === "repair") {
    return (
      <div className="mt-5 rounded-xl border border-stone-800 bg-stone-900/60 p-4 text-sm text-stone-300">
        {summary.cancelled ? "Cover repair cancelled." : "Cover repair complete."}{" "}
        {summary.thumbnailsRecovered} recovered from cache. {summary.discovered}{" "}
        missing covers queued: {summary.thumbnailsGenerated} generated,{" "}
        {summary.thumbnailFailures} failed.
      </div>
    );
  }
  return (
    <div className="mt-5 rounded-xl border border-stone-800 bg-stone-900/60 p-4 text-sm text-stone-300">
      {summary.cancelled ? "Scan cancelled." : "Scan complete."}{" "}
      {summary.discovered} discovered, {summary.added} added, {summary.updated}{" "}
      updated, {summary.missing} missing. Covers:{" "}
      {summary.thumbnailsRecovered} recovered, {summary.thumbnailsGenerated}{" "}
      generated, {summary.thumbnailFailures} failed. {summary.issues} scan issues.
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
