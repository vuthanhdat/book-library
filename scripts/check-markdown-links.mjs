import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, extname, resolve } from "node:path";
import process from "node:process";

const root = process.cwd();
const ignoredDirectories = new Set([
  ".git",
  "node_modules",
  "dist",
  "target",
  "tmp",
]);

function markdownFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    if (entry.isDirectory()) {
      return ignoredDirectories.has(entry.name)
        ? []
        : markdownFiles(resolve(directory, entry.name));
    }
    return extname(entry.name).toLowerCase() === ".md"
      ? [resolve(directory, entry.name)]
      : [];
  });
}

const failures = [];
const linkPattern = /\[[^\]]*]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;

for (const file of markdownFiles(root)) {
  const markdown = readFileSync(file, "utf8")
    .replace(/```[\s\S]*?```/g, "")
    .replace(/`[^`\n]*`/g, "");
  for (const match of markdown.matchAll(linkPattern)) {
    const rawTarget = match[1].replace(/^<|>$/g, "");
    if (
      rawTarget.startsWith("#") ||
      /^[a-z][a-z\d+.-]*:/i.test(rawTarget)
    ) {
      continue;
    }

    const pathPart = decodeURIComponent(rawTarget.split("#", 1)[0]);
    if (!pathPart) continue;

    const target = resolve(dirname(file), pathPart);
    if (!existsSync(target) || (!statSync(target).isFile() && !statSync(target).isDirectory())) {
      failures.push(`${file}: missing ${rawTarget}`);
    }
  }
}

if (failures.length > 0) {
  process.stderr.write(`${failures.join("\n")}\n`);
  process.exit(1);
}

process.stdout.write("Markdown links are valid.\n");
