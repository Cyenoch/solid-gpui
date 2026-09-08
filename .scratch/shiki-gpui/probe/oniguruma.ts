import { createHighlighterCore } from 'shiki/core';
import { createOnigurumaEngine } from 'shiki/engine/oniguruma';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';
import ts from 'shiki/langs/typescript.mjs';
import tsx from 'shiki/langs/tsx.mjs';
import rust from 'shiki/langs/rust.mjs';
import json from 'shiki/langs/json.mjs';
import markdown from 'shiki/langs/markdown.mjs';
import dark from 'shiki/themes/github-dark.mjs';

const langs = [ts, tsx, rust, json, markdown];
const onig = await createHighlighterCore({ langs, themes: [dark], engine: createOnigurumaEngine(import('shiki/wasm')) });
const target = process.argv[2] ?? 'auto';
const js = await createHighlighterCore({ langs, themes: [dark], engine: createJavaScriptRegexEngine({ target: target as 'auto' | 'ES2018' }) });
const fixtures = [
  ['typescript', 'const 名称 = "😀e\u0301";\r\n/* open\r\nclose */ const n = 42;\r\n'],
  ['tsx', 'export const App = () => <div title="你好">{42}</div>;'],
  ['rust', 'fn main() { let text = r#"你好😀"#; println!("{text}"); }'],
  ['json', '{"emoji":"😀", "enabled":true}'],
  ['markdown', '# Heading\n\n```typescript\nconst n = 1;\n```\n'],
] as const;
const cases = fixtures.map(([lang, source]) => {
  const options = { lang, theme: 'github-dark' };
  const tokens = onig.codeToTokens(source, options).tokens;
  const jsTokens = js.codeToTokens(source, options).tokens;
  const parity = JSON.stringify(tokens) === JSON.stringify(jsTokens);
  let cursor = 0;
  const runs: { start: number; end: number; color?: string }[] = [];
  for (const token of tokens.flat()) {
    if (source.slice(token.offset, token.offset + token.content.length) !== token.content) throw new Error('source mismatch');
    const start = Buffer.byteLength(source.slice(0, token.offset));
    const end = start + Buffer.byteLength(token.content);
    if (start > cursor) runs.push({ start: cursor, end: start });
    if (end > start) runs.push({ start, end, color: token.color });
    cursor = end;
  }
  if (cursor < Buffer.byteLength(source)) runs.push({ start: cursor, end: Buffer.byteLength(source) });
  const bytes = Buffer.from(source);
  const reconstructed = runs.map(run => bytes.subarray(run.start, run.end).toString('utf8')).join('');
  if (reconstructed !== source) throw new Error(`${lang}: UTF-8 reconstruction mismatch`);
  return { lang, parity, sourcePreserved: true, utf16Length: source.length, utf8Bytes: bytes.length, nativeRuns: runs.length, differences: parity ? undefined : { oniguruma: tokens, javascript: jsTokens } };
});
console.log(JSON.stringify({ runtime: typeof Bun === 'undefined' ? `Node ${process.version}` : `Bun ${Bun.version}`, target, shiki: '4.4.2', cases }));
onig.dispose();
js.dispose();
