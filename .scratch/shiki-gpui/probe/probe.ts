import { createHighlighterCoreSync } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';
import ts from 'shiki/langs/typescript.mjs';
import tsx from 'shiki/langs/tsx.mjs';
import rust from 'shiki/langs/rust.mjs';
import json from 'shiki/langs/json.mjs';
import markdown from 'shiki/langs/markdown.mjs';
import dark from 'shiki/themes/github-dark.mjs';
import light from 'shiki/themes/github-light.mjs';

const assert = (value: unknown, message: string) => { if (!value) throw new Error(message); };
const started = Date.now();
const target = (globalThis as any).probeTarget ?? 'ES2018';
const highlighter = createHighlighterCoreSync({
  langs: [ts, tsx, rust, json, markdown], themes: [dark, light],
  engine: createJavaScriptRegexEngine({ target }),
});
const initMs = Date.now() - started;
const samples = [
  ['typescript', 'const 名称 = "😀e\u0301";\r\n/* open\r\nclose */ const n = 42;\r\n'],
  ['tsx', 'export const App = () => <div title="你好">{42}</div>;'],
  ['rust', 'fn main() { let text = r#"你好😀"#; println!("{text}"); }'],
  ['json', '{"emoji":"😀", "enabled":true}'],
  ['markdown', '# Heading\n\n```typescript\nconst n = 1;\n```\n'],
] as const;
const cases = [];
for (const [lang, source] of samples) {
  try {
  const start = Date.now();
  const result = highlighter.codeToTokens(source, { lang, theme: 'github-dark' });
  const lines = source.split(/\r\n|\n/);
  assert(result.tokens.length === lines.length, `${lang}: line count`);
  result.tokens.forEach((line, i) => assert(line.map(t => t.content).join('') === lines[i], `${lang}: source fidelity`));
  cases.push({ lang, utf16Length: source.length, lines: lines.length, tokens: result.tokens.flat().length, elapsedMs: Date.now() - start });
  } catch (error) {
    cases.push({ lang, error: String(error) });
  }
}
let stateContinuation = false;
let themeSwitch = false;
let contractError: string | undefined;
let offsets: unknown[] = [];
try {
const source = samples[0][1];
const tokens = highlighter.codeToTokens(source, { lang: 'typescript', theme: 'github-dark' }).tokens;
const tokenOffsets = tokens.flat().filter(t => t.content.includes('42')).map(t => ({ offset: t.offset, expectedUtf16: source.indexOf(t.content) }));
offsets = tokenOffsets;
assert(tokenOffsets.every(t => t.offset === t.expectedUtf16), 'UTF-16 offsets');
const prefix = '/* open';
const suffix = 'close */ const n = 42;';
const first = highlighter.codeToTokens(prefix, { lang: 'typescript', theme: 'github-dark' });
const state = highlighter.getLastGrammarState(first.tokens)!;
const continued = highlighter.codeToTokens(suffix, { lang: 'typescript', theme: 'github-dark', grammarState: state });
const whole = highlighter.codeToTokens(`${prefix}\n${suffix}`, { lang: 'typescript', theme: 'github-dark' });
const styles = (line: typeof tokens[number]) => line.map(({ content, color, fontStyle }) => ({ content, color, fontStyle }));
assert(JSON.stringify(styles(continued.tokens[0])) === JSON.stringify(styles(whole.tokens[1])), 'multiline state continuation');
stateContinuation = true;
const lightTokens = highlighter.codeToTokens(source, { lang: 'typescript', theme: 'github-light' });
assert(JSON.stringify(styles(lightTokens.tokens[0])) !== JSON.stringify(styles(tokens[0])), 'theme switch');
themeSwitch = true;
} catch (error) { contractError = String(error); }
const flags: Record<string, boolean> = {};
for (const flag of ['d', 'v']) { try { new RegExp('', flag); flags[flag] = true; } catch { flags[flag] = false; } }
(globalThis as any).probeResult = JSON.stringify({ shiki: '4.4.2', engine: 'javascript', target, wasm: typeof WebAssembly, flags, initMs, cases, offsets, stateContinuation, themeSwitch, contractError });
highlighter.dispose();
