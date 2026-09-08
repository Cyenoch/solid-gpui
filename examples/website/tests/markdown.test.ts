import { expect, test } from "bun:test";
import { markdownBlocks } from "../src/markdown";

test("GFM tables preserve inline code, escaped pipes, alignment and missing cells", () => {
  const [table, paragraph] = markdownBlocks(
    `| Syntax | Meaning |\n| :--- | ---: |\n| \`ctrl-o\` | Control+O |\n| A\\|B |\n\nAfter the table.`,
  );
  expect(table).toMatchObject({
    kind: "table",
    header: ["Syntax", "Meaning"],
    align: ["left", "right"],
    rows: [
      ["`ctrl-o`", "Control+O"],
      ["A|B", ""],
    ],
  });
  expect(paragraph).toEqual({ kind: "paragraph", text: "After the table." });
});

test("table-looking fenced code remains code and ordinary pipes remain prose", () => {
  expect(markdownBlocks("~~~md\nA | B\n--- | ---\n~~~\n\nA | B")).toEqual([
    { kind: "code", text: "A | B\n--- | ---", language: "md" },
    { kind: "paragraph", text: "A | B" },
  ]);
});
