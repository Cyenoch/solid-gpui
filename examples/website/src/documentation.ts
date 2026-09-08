export { markdownBlocks, type MarkdownBlock } from "./markdown";
const files = import.meta.glob("../../../docs/*.md", { query: "?raw", import: "default", eager: true });
export const referenceDocs = Object.entries(files)
  .filter(([path]) => !path.endsWith("/README.md") && !path.endsWith(".zh-CN.md"))
  .map(([path, contents]) => {
    const source = String(contents);
    const chinese = files[path.replace(/\.md$/, ".zh-CN.md")];
    if (typeof chinese !== "string") throw new Error(`Missing Chinese document: ${path}`);
    return {
      id: path.split("/").pop()!.replace(/\.md$/, ""),
      title: source.match(/^# (.+)/m)![1],
      titleChinese: chinese.match(/^# (.+)/m)![1],
      source,
      sourceChinese: chinese,
    };
  })
  .sort((a, b) => a.title.localeCompare(b.title));
