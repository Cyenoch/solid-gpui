export type MarkdownBlock =
  | { kind: "heading" | "paragraph"; text: string; level?: number }
  | { kind: "code"; text: string; language: string; level?: never }
  | { kind: "table"; text: string; header: string[]; rows: string[][]; align: ("left" | "center" | "right")[] };

function cells(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/(?<!\\)\|$/, "")
    .split(/(?<!\\)\|/)
    .map((cell) => cell.trim().replace(/\\\|/g, "|"));
}

/** Preserve fenced code before interpreting block syntax, including GFM tables. */
export function markdownBlocks(source: string): MarkdownBlock[] {
  const lines = source.replace(/\r\n/g, "\n").split("\n");
  const blocks: MarkdownBlock[] = [];
  let paragraph: string[] = [];
  const flush = () => {
    if (paragraph.length)
      blocks.push({
        kind: "paragraph",
        text: paragraph.join(paragraph.some((line) => /^\s*(?:[-*] |\d+\. |>|\|)/.test(line)) ? "\n" : " "),
      });
    paragraph = [];
  };
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const fence = line.match(/^\s{0,3}(`{3,}|~{3,})\s*(\S*)/);
    if (fence) {
      flush();
      const code: string[] = [];
      const close = new RegExp(`^\\s{0,3}${fence[1][0]}{${fence[1].length},}\\s*$`);
      while (++i < lines.length && !close.test(lines[i])) code.push(lines[i]);
      blocks.push({ kind: "code", text: code.join("\n"), language: fence[2] || "text" });
    } else if (
      line.includes("|") &&
      i + 1 < lines.length &&
      cells(lines[i + 1]).length === cells(line).length &&
      cells(lines[i + 1]).every((cell) => /^:?-{3,}:?$/.test(cell))
    ) {
      flush();
      const header = cells(line);
      const align = cells(lines[++i]).map((cell) =>
        cell.endsWith(":") ? (cell.startsWith(":") ? "center" : "right") : "left",
      );
      const rows: string[][] = [];
      const raw = [line, lines[i]];
      while (i + 1 < lines.length && lines[i + 1].trim() && lines[i + 1].includes("|")) {
        raw.push(lines[++i]);
        const row = cells(lines[i]);
        rows.push(header.map((_, index) => row[index] ?? ""));
      }
      blocks.push({ kind: "table", text: raw.join("\n"), header, rows, align });
    } else if (/^#{1,6} /.test(line)) {
      flush();
      const level = line.indexOf(" ");
      blocks.push({ kind: "heading", text: line.slice(level + 1), level });
    } else if (!line.trim()) flush();
    else paragraph.push(line);
  }
  flush();
  return blocks;
}
