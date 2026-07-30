import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import {
  App,
  BookDetailPage,
  GlobalSearchWorkspace,
  NotesWorkspace,
  StudyReader,
  StudyWorkspace,
  StartupPanel,
  filterBookChoices,
  filterCatalogBooks,
  isValidBookDisplayTitle,
  parseBookTags,
  normalizeLookupSelection,
  resolveTheme,
  scanButtonLabels,
  safeSearchSnippet,
  type Book,
} from "./App";

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
    expect(markup).toContain("Light theme");
  });

  it("restores only a supported saved theme", () => {
    expect(resolveTheme("light")).toBe("light");
    expect(resolveTheme("dark")).toBe("dark");
    expect(resolveTheme("unknown")).toBe("dark");
    expect(resolveTheme(null)).toBe("dark");
  });

  it("accepts a bounded selected OCR term for instant lookup", () => {
    expect(normalizeLookupSelection("  画面  ")).toBe("画面");
    expect(normalizeLookupSelection("   ")).toBeNull();
    expect(normalizeLookupSelection("語".repeat(201))).toBeNull();
  });

  it("renders a two-column study reader with navigation, OCR, and dictionary", () => {
    const markup = renderToStaticMarkup(
      <StudyReader
        ankiEnabled
        busy={false}
        dictionaryEnabled
        dictionaryLookup={{
          query: "画面",
          entries: [
            {
              id: "entry-1",
              expression: "画面",
              reading: "がめん",
              partOfSpeech: "noun",
              meaningVi: "màn hình",
              hanViet: "HỌA DIỆN",
              packageName: "test",
              packageVersion: "1",
            },
          ],
          tokens: [],
        }}
        dictionaryQuery="画面"
        error={null}
        ocrEnabled
        ocrPage={{
          id: "ocr-1",
          bookId: books[0].id,
          bookTitle: books[0].title,
          pageIndex: 1,
          text: "画面を見ます。",
          confidence: 90,
          providerId: "test",
          providerVersion: "1",
          blocks: [],
        }}
        onBack={() => undefined}
        onCreateCard={() => undefined}
        onDictionaryQueryChange={() => undefined}
        onLookup={() => undefined}
        onNavigate={() => undefined}
        onOpenFolder={() => undefined}
        onRunOcr={() => undefined}
        page={{
          bookId: books[0].id,
          bookTitle: books[0].title,
          pageIndex: 1,
          pageCount: 56,
          width: 1200,
          height: 1800,
          imageDataUrl: "data:image/png;base64,AAAA",
        }}
      />,
    );

    expect(markup).toContain("Japanese → Vietnamese");
    expect(markup).toContain("Selectable page text");
    expect(markup).toContain("画面を見ます。");
    expect(markup).toContain("màn hình");
    expect(markup).toContain('value="2"');
    expect(markup).toContain("/ 56");
  });

  it("labels rescan and cover repair independently", () => {
    expect(scanButtonLabels("rescan")).toEqual({
      repair: "Repair covers",
      rescan: "Rescanning…",
    });
    expect(scanButtonLabels("repair")).toEqual({
      repair: "Repair running…",
      rescan: "Rescan",
    });
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

  it("filters related-book combobox choices by Unicode title and folder", () => {
    expect(filterBookChoices(books, "私 講義")).toEqual([books[0]]);
    expect(filterBookChoices(books, "programming systems")).toEqual([books[1]]);
    expect(filterBookChoices(books, "unknown")).toEqual([]);
    expect(filterBookChoices(books, " ")).toBe(books);
  });

  it("normalizes book hashtags and renders reading controls on book detail", () => {
    expect(parseBookTags("#心理学, japanese  #心理学 #to-read")).toEqual([
      "japanese",
      "to-read",
      "心理学",
    ]);
    const markup = renderToStaticMarkup(
      <BookDetailPage
        busy={false}
        coverProgress={[
          "Opening the source file and waiting for local availability…",
          "Source opened. Rendering the first page…",
        ]}
        detail={{
          ...books[0],
          readingStatus: "reading",
          tags: ["心理学"],
          notes: [{ id: "note-1", title: "Key ideas" }],
        }}
        error={null}
        onBack={() => undefined}
        onEditTitle={() => undefined}
        onForceCover={() => undefined}
        onNewNote={() => undefined}
        onOpenFolder={() => undefined}
        onOpenNote={() => undefined}
        onReadStudy={() => undefined}
        onSave={() => undefined}
      />,
    );

    expect(markup).toContain("Book detail");
    expect(markup).toContain("Force cover generation");
    expect(markup).toContain("Cover generation log");
    expect(markup).toContain("Rendering the first page");
    expect(markup).toContain("Reading status");
    expect(markup).toContain("Key ideas");
  });

  it("allows an unavailable cloud book to retry its cover", () => {
    const markup = renderToStaticMarkup(
      <BookDetailPage
        busy={false}
        coverProgress={[]}
        detail={{
          ...books[0],
          status: "unavailable",
          readingStatus: "unread",
          tags: [],
          notes: [],
        }}
        error={null}
        onBack={() => undefined}
        onEditTitle={() => undefined}
        onForceCover={() => undefined}
        onNewNote={() => undefined}
        onOpenFolder={() => undefined}
        onOpenNote={() => undefined}
        onReadStudy={() => undefined}
        onSave={() => undefined}
      />,
    );

    const forceButton = markup.match(
      /<button[^>]*>Force cover generation<\/button>/,
    )?.[0];
    expect(forceButton).toBeDefined();
    expect(forceButton).not.toContain(' disabled=""');
  });

  it("validates app-local Unicode display titles without allowing line breaks", () => {
    expect(isValidBookDisplayTitle(`  ${books[0].title}  `)).toBe(true);
    expect(isValidBookDisplayTitle("line\nbreak")).toBe(false);
    expect(isValidBookDisplayTitle(" ".repeat(20))).toBe(false);
    expect(isValidBookDisplayTitle("本".repeat(513))).toBe(false);
  });

  it("renders a portable Markdown notes workspace with explicit save and refresh", () => {
    const markup = renderToStaticMarkup(
      <NotesWorkspace
        books={books}
        busy={false}
        configuration={{ displayName: "My Notes" }}
        draft={"# Reading note\n\nChanged"}
        error={null}
        notes={[
          {
            id: "note-1",
            title: "Reading note",
            relativePath: "Reading note.md",
            status: "available",
            bookId: books[0].id,
            bookTitle: books[0].title,
            modifiedAtMs: 1,
          },
        ]}
        onChooseRoot={() => undefined}
        onCreate={() => undefined}
        onDraftChange={() => undefined}
        onOpenExternal={() => undefined}
        onOpenRoot={() => undefined}
        onRefresh={() => undefined}
        onSave={() => undefined}
        onSelect={() => undefined}
        selectedNote={{
          id: "note-1",
          title: "Reading note",
          relativePath: "Reading note.md",
          body: "# Reading note\n",
          bookId: books[0].id,
          bookTitle: books[0].title,
          backlinks: [
            {
              id: "note-2",
              title: "Related idea",
              relativePath: "Related idea.md",
            },
          ],
        }}
        summary={null}
      />,
    );

    expect(markup).toContain("My Notes");
    expect(markup).toContain("Refresh");
    expect(markup).toContain("Open externally");
    expect(markup).toContain("Related idea");
    expect(markup).toContain(">Save<");
  });

  it("renders filtered global-search results and escapes snippets", () => {
    const markup = renderToStaticMarkup(
      <GlobalSearchWorkspace
        busy={false}
        diagnostics={{
          documents: 12,
          failedJobs: 0,
          lastRebuildAt: "2026-07-26",
        }}
        error={null}
        onOpenResult={() => undefined}
        onQueryChange={() => undefined}
        onRebuild={() => undefined}
        onScopeChange={() => undefined}
        query="自動操縦"
        results={[
          {
            sourceKind: "note",
            sourceId: "note-1",
            scope: "headings",
            title: "読書メモ",
            snippet: "脳の<mark>自動操縦</mark>",
            relativePath: "読書メモ.md",
            status: "available",
            rank: -1,
          },
        ]}
        scope="headings"
      />,
    );

    expect(markup).toContain("Search everything");
    expect(markup).toContain("12 indexed documents");
    expect(markup).toContain("<mark>自動操縦</mark>");
    expect(safeSearchSnippet("<script>x</script><mark>ok</mark>")).toBe(
      "&lt;script&gt;x&lt;/script&gt;<mark>ok</mark>",
    );
  });

  it("renders the offline Japanese study workflow and explicit module states", () => {
    const markup = renderToStaticMarkup(
      <StudyWorkspace
        aiContext=""
        aiDrafts={[]}
        aiKind="explain"
        books={books}
        busy={false}
        dictionaryLookup={{
          query: "日本語",
          tokens: [
            { surface: "日本語", start: 0, end: 3, entries: [] },
          ],
          entries: [
            {
              id: "starter-002",
              expression: "日本語",
              reading: "にほんご",
              partOfSpeech: "danh từ",
              meaningVi: "tiếng Nhật",
              hanViet: "NHẬT BẢN NGỮ",
              packageName: "Book Library Japanese Starter",
              packageVersion: "1",
            },
          ],
        }}
        dictionaryQuery="日本語"
        error={null}
        notice="Imported 266,903 dictionary entries."
        learningDrafts={[]}
        modules={[
          {
            id: "dictionary",
            enabled: true,
            available: true,
            status: "ready",
          },
          {
            id: "ocr",
            enabled: false,
            available: false,
            status: "disabled",
          },
          {
            id: "anki",
            enabled: true,
            available: true,
            status: "ready",
          },
        ]}
        ocrBookId=""
        ocrPageNumber={1}
        ocrPages={[
          {
            id: "ocr-page-1",
            bookId: books[0].id,
            bookTitle: books[0].title,
            pageIndex: 0,
            text: "日本語 を 勉強 する",
            confidence: 0.91,
            providerId: "tesseract-cli",
            providerVersion: "system",
            blocks: [],
          },
        ]}
        onAiContextChange={() => undefined}
        onAiKindChange={() => undefined}
        onApproveDraft={() => undefined}
        onCancelOcr={() => undefined}
        onClearHistory={() => undefined}
        onCreateDictionaryDraft={() => undefined}
        onCreateOcrDraft={() => undefined}
        onDictionaryQueryChange={() => undefined}
        onExport={() => undefined}
        onGenerateAi={() => undefined}
        onImportDictionary={() => undefined}
        onLookup={() => undefined}
        onOcrBookChange={() => undefined}
        onOcrPageChange={() => undefined}
        onRunOcr={() => undefined}
        onSaveHistoryChange={() => undefined}
        onToggleModule={() => undefined}
        onTrimOcrPage={() => undefined}
        saveLookupHistory={false}
        trustedModules={[]}
      />,
    );

    expect(markup).toContain("Japanese study");
    expect(markup).toContain("日本語");
    expect(markup).toContain("にほんご");
    expect(markup).toContain("tiếng Nhật");
    expect(markup).toContain("Import dictionary ZIP/TSV");
    expect(markup).toContain("Imported 266,903 dictionary entries.");
    expect(markup).toContain("runtime missing");
    expect(markup).toContain("OCR is off. Enable it here");
    expect(markup).toContain("Enable OCR");
    expect(markup).toContain("Trim spaces");
    expect(markup).toContain(
      "Select Japanese text below to look it up instantly.",
    );
    expect(markup).toContain("Export approved TSV");
  });
});
