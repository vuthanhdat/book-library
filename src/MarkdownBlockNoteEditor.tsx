import "@blocknote/core/fonts/inter.css";
import { BlockNoteView } from "@blocknote/mantine";
import "@blocknote/mantine/style.css";
import { useCreateBlockNote } from "@blocknote/react";
import { useCallback, useLayoutEffect, useRef } from "react";

export interface MarkdownParts {
  frontmatter: string;
  body: string;
}

/**
 * Keep YAML frontmatter outside BlockNote's Markdown round-trip.
 * BlockNote intentionally supports a CommonMark/GFM subset and does not own
 * note metadata, so the original frontmatter must remain opaque to the editor.
 */
export function splitMarkdownFrontmatter(markdown: string): MarkdownParts {
  const firstLineEnd = markdown.indexOf("\n");
  const firstLine = (firstLineEnd === -1
    ? markdown
    : markdown.slice(0, firstLineEnd)
  ).replace(/\r$/, "");

  if (firstLine.trim() !== "---") {
    return { frontmatter: "", body: markdown };
  }

  let lineStart = firstLineEnd === -1 ? markdown.length : firstLineEnd + 1;
  while (lineStart <= markdown.length) {
    const lineEnd = markdown.indexOf("\n", lineStart);
    const rawLine = markdown.slice(
      lineStart,
      lineEnd === -1 ? markdown.length : lineEnd,
    );
    const line = rawLine.replace(/\r$/, "").trim();
    if (line === "---" || line === "...") {
      const contentStart = lineEnd === -1 ? markdown.length : lineEnd + 1;
      const frontmatter = markdown.slice(0, contentStart).replace(/(?:\r?\n)+$/, "");
      const body = markdown.slice(contentStart).replace(/^(?:\r?\n)+/, "");
      return { frontmatter, body };
    }
    if (lineEnd === -1) break;
    lineStart = lineEnd + 1;
  }

  return { frontmatter: "", body: markdown };
}

export function mergeMarkdownFrontmatter(
  frontmatter: string,
  body: string,
): string {
  if (!frontmatter) return body;
  const lineBreak = frontmatter.includes("\r\n") ? "\r\n" : "\n";
  const cleanBody = body.replace(/^(?:\r?\n)+/, "");
  const bodyLineBreak = cleanBody.endsWith("\n") || cleanBody.endsWith("\r")
    ? ""
    : lineBreak;
  return cleanBody
    ? `${frontmatter}${lineBreak}${lineBreak}${cleanBody}${bodyLineBreak}`
    : `${frontmatter}${lineBreak}${lineBreak}`;
}

interface MarkdownBlockNoteEditorProps {
  initialMarkdown: string;
  noteId: string;
  onChange: (markdown: string) => void;
  theme: "dark" | "light";
}

function MarkdownBlockNoteEditorCanvas({
  initialMarkdown,
  noteId,
  onChange,
  theme,
}: MarkdownBlockNoteEditorProps) {
  const editor = useCreateBlockNote({}, [noteId]);
  const syncingRef = useRef(true);
  const { frontmatter, body } = splitMarkdownFrontmatter(initialMarkdown);

  useLayoutEffect(() => {
    syncingRef.current = true;
    try {
      const blocks = editor.tryParseMarkdownToBlocks(body);
      editor.replaceBlocks(
        editor.document,
        blocks.length > 0 ? blocks : [{ type: "paragraph" }],
      );
    } catch {
      // Keep a malformed or unsupported note editable as plain paragraph text.
      editor.replaceBlocks(editor.document, [{ type: "paragraph", content: body }]);
    }
    syncingRef.current = false;
  }, [body, editor, noteId]);

  const handleChange = useCallback(
    (changedEditor: typeof editor) => {
      if (syncingRef.current) return;
      onChange(
        mergeMarkdownFrontmatter(
          frontmatter,
          changedEditor.blocksToMarkdownLossy(changedEditor.document),
        ),
      );
    },
    [frontmatter, onChange],
  );

  return (
    <BlockNoteView
      aria-label="Markdown note editor"
      className="book-note-editor-view"
      editor={editor}
      onChange={handleChange}
      theme={theme}
    />
  );
}

export function MarkdownBlockNoteEditor({
  initialMarkdown,
  noteId,
  onChange,
  theme,
}: MarkdownBlockNoteEditorProps) {
  // BlockNote is a browser editor. Keep static rendering and non-browser test
  // output usable without instantiating its DOM-dependent editor core.
  if (typeof window === "undefined") {
    return (
      <textarea
        aria-label="Markdown note editor"
        className="book-note-editor-fallback"
        readOnly
        value={initialMarkdown}
      />
    );
  }

  return (
    <div className="book-note-editor" data-editor="blocknote">
      <MarkdownBlockNoteEditorCanvas
        initialMarkdown={initialMarkdown}
        noteId={noteId}
        onChange={onChange}
        theme={theme}
      />
    </div>
  );
}
