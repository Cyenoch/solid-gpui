# Shiki upstream research for GPUI integration

Research date: 2026-09-07. This note investigates upstream behavior and alternatives; local runtime and rendering feasibility are covered by the companion report. Findings are based on official documentation and source checkouts. Recommendations below are engineering inferences, distinguished from upstream guarantees.

## Source baseline

- Shiki documentation reported **4.4.2** during research. The `v4.4.2` annotated tag resolves to source commit [`2aec8566cf186a92f4e094e6a92c335de3181ecd`](https://github.com/shikijs/shiki/tree/2aec8566cf186a92f4e094e6a92c335de3181ecd). All Shiki source links below use this commit, rather than floating `main`.
- Giallo source examined: [`148fec043603627a226115880efd79a73bb454cd`](https://github.com/getzola/giallo/tree/148fec043603627a226115880efd79a73bb454cd), whose manifest declares 0.5.2.
- Grammar/theme collection inspected independently: [`fde14af8c3612d6b966f2d3b7a86d3a79bb396e9`](https://github.com/shikijs/textmate-grammars-themes/tree/fde14af8c3612d6b966f2d3b7a86d3a79bb396e9). This is an inventory snapshot, **not a claim that these exact assets are bundled in Shiki 4.4.2**. Shiki's source catalog specifies `tm-grammars ^1.32.0`, `tm-themes ^1.12.3`, and `@shikijs/vscode-textmate ^10.0.2`. [Dependency catalog](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/pnpm-workspace.yaml)

## Direct integration boundary

Shiki explicitly offers `codeToTokens` and `codeToHast` for custom rendering. Its highlighter can be initialized asynchronously and then called synchronously, with explicitly loaded languages/themes. Upstream recommends a long-lived highlighter instance. This permits native text rendering without a DOM or an HTML parser. [Official installation guide](https://shiki.style/guide/install)

The dependency layers are narrower than the top-level `shiki` package:

| Layer | Upstream responsibility | GPUI implication |
| --- | --- | --- |
| `@shikijs/primitive` | Registry, grammar/theme resolution, theme normalization, tokenization, grammar state | Potential smallest token-only integration surface |
| `@shikijs/core` | Higher-level highlighter and rendering APIs, including ANSI handling | Convenient stable entry point; bundler should remove unused HTML/HAST code where possible |
| `@shikijs/engine-javascript` | Oniguruma-pattern conversion and native JS regex scanning | Needs JS regex features; no WASM |
| `@shikijs/engine-oniguruma` | Oniguruma WASM binding | Needs working WebAssembly integration |
| Languages and themes | TextMate grammar and theme assets | Select explicitly; include embedded-language dependencies |

These boundaries are visible in the [primitive exports](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/primitive/src/index.ts), [primitive manifest](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/primitive/package.json), [core manifest](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/core/package.json), and [core ANSI adapter](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/core/src/highlight/code-to-tokens-base.ts).

### Tokens, styles, and coordinates

`TokensResult` contains `tokens: ThemedToken[][]`, root foreground/background colors, theme metadata, and optional final grammar state. Tokens contain `content`, input-relative `offset`, optional foreground/background colors, and a font-style bitfield. HTML-specific attributes and styles also exist but are not a portable native styling contract. Multi-theme token variants are supported. [Token types](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/types/src/tokens.ts)

**Offsets are JavaScript UTF-16 code-unit coordinates**, inferred directly from `substring`, `length`, and line-offset accumulation. They must not be passed as Rust string byte indices. LF and CRLF are split out of token content, while input offsets count their original lengths; a lone CR is not a newline in this splitter. Empty lines can have empty token arrays. Preserve the original input and its line endings. Either convert boundaries once to UTF-8 or construct per-line byte spans by accumulating the UTF-8 lengths of token content, accounting separately for original line separators. [Tokenizer](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/primitive/src/highlight/code-to-tokens-base.ts), [line splitter](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/primitive/src/utils/strings.ts)

Although the generic token type supports `bgColor`, the ordinary TextMate tokenization path examined extracts foreground and font style from encoded metadata and does not populate per-token background. Do not promise full VS Code visual equivalence merely because the token type has a field. The root theme background is distinct. [Token construction](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/primitive/src/highlight/code-to-tokens-base.ts)

### Grammar state and editing

The public grammar-state API supports continuing from a previous grammar context. This is useful for snippets or complete-line chunks. It does not itself provide a document editor's invalidation, checkpoint, cancellation, or viewport scheduling model. [Grammar-state guide](https://shiki.style/guide/grammar-state)

At the examined commit, state is an actual `GrammarState` instance holding TextMate stacks by theme. `toJSON()` emits descriptive language/theme/scope information, not a resumable stack serialization. `getGrammarStack()` rejects objects that are not instances of the class. The tokenizer also checks language and theme compatibility. Therefore state should remain inside the runtime/worker that owns the highlighter; expose opaque session/checkpoint handles if a native boundary needs references. Reloading grammars, replacing the runtime, or changing themes should invalidate applicable checkpoints. [State implementation](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/primitive/src/textmate/grammar-state.ts), [state validation](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/primitive/src/highlight/code-to-tokens-base.ts)

Engineering inference: for edits, restart at the nearest valid preceding checkpoint and continue until text and downstream state converge. For append-only streaming, retain the state before the unfinished final line and retokenize that line when more text arrives. Simply feeding arbitrary character chunks as independent lines can change regex behavior. A production editor needs parity tests across multiline comments, template strings, heredocs, embedded languages, and blank lines.

### Latency and incomplete work

The default tokenization budget is **500 ms per line** and the default line-length limit is unlimited. The wrapper passes the budget into TextMate calls; it does not report `stoppedEarly` to the caller. A configured long-line cutoff skips tokenization without advancing the grammar stack. This may affect following lines. Scope explanations trigger both `tokenizeLine` and `tokenizeLine2`, plus optional theme matching work. These are source observations, not performance measurements. [Tokenizer implementation](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/primitive/src/highlight/code-to-tokens-base.ts)

Engineering inference: highlighting should run outside GPUI render/layout and return immutable prepared results tagged with a document revision. Disable explanations for display. Do not treat the per-line budget as a strict interruption mechanism for one native regex match, or silently mark truncated work as fully highlighted. A service exposing completion status may need the lower-level TextMate result or a carefully maintained upstream change. Use bounded scheduling and discard stale results.

## Engine and runtime choices

The default Oniguruma engine runs compiled WASM. The JavaScript engine transpiles Oniguruma patterns to native regexes, can be smaller, and is faster for some languages. Documentation says all bundled languages were supported as of 3.9.1, while custom grammar compatibility is strongest with Oniguruma. JS mode is strict by default; `forgiving` suppresses conversion failures and can cause mismatches. Runtime target selection is automatic or explicit (`ES2018`, `ES2024`, `ES2025`); newer targets use Unicode sets (`v`). Precompiled-language documentation currently warns of a known issue affecting many languages. [Official engine guide](https://shiki.style/guide/regex-engines)

There is an additional embedded-runtime requirement beyond target selection: the JS engine requests regex match indices (`hasIndices: true`) and consumes `match.indices`. The converter also configures recursion and other emulation rules. Setting target to ES2018 does not prove that a nominal ES2018 runtime supports the whole package. Test the actual embedded engine, bundled source, language set, and themes. [Regex conversion](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/engine-javascript/src/engine-compile.ts), [scanner](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/engine-javascript/src/scanner.ts)

Shiki v4's declared Node baseline is >=20. That is a supported Node version policy, not evidence that a bundled token-only build cannot run in another engine. [v4 migration guide](https://shiki.style/blog/v4), [engine manifest](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/engine-javascript/package.json)

**Local evidence supplied by the companion investigation:** the same Shiki bundle passes the selected TypeScript/TSX/Markdown cases in Bun but throws `SyntaxError: too many captures` for those cases in rquickjs 0.12.2; Rust/JSON pass. Both ES2018 and automatic targets fail, despite that runtime supporting `d` and `v`. This demonstrates why regex flags alone are insufficient. See the [Bun result](probe/bun-result.json), [QuickJS result](probe/quickjs-result.json), [automatic-target result](probe/quickjs-auto-result.json), and [runtime probe](probe/quickjs/src/main.rs). The companion investigation attributes the failure to QuickJS's 255-capture limit and reports `WebAssembly` absent. Native scanner integration, another tokenizer runtime, or native TextMate therefore needs evaluation before recommending broad Shiki use in the current host.

A custom engine is a supported seam: it supplies `createScanner()` and `createString()` compatible with TextMate's scanner/string interfaces. A native Oniguruma bridge could preserve JS TextMate/theme logic while replacing WASM or JS regex conversion. This is an engineering option, not an existing validated GPUI adapter. It requires exact capture ordering, unmatched captures, search options, UTF-16/UTF-8 conversion, ownership, and error handling, with potentially frequent cross-runtime calls. [Engine types](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/packages/types/src/engines.ts)

## Rust alternatives

| Option | Verified capabilities | Material limits for this exploration |
| --- | --- | --- |
| Keep upstream Shiki and adapt tokens | Public custom-renderer output; JS or Oniguruma engine | Embedded-runtime compatibility and UI scheduling remain local work |
| Giallo 0.5.2 | Native Rust TextMate engine; Shiki-curated assets; styled token output; single/dual themes | EUPL-1.2; public API examined highlights complete input; no public resumable tokenizer state |
| Syntect 5.3.0 | Rust parser/highlighter layers; reusable scopes and styles | Uses Sublime grammar model, so not a direct Shiki implementation |
| Syntect + syntect-tmlanguage 0.1.0 | Converts TextMate grammar data through Sublime YAML | Explicitly unsupported `begin`/`while`, `\\G`, selector injections; unsuitable for broad Shiki parity |
| Rewrite Shiki/TextMate in Rust | Could choose native structures and retain asset ecosystem | Must implement grammar semantics, scope/theme matching, regex behavior, and maintain parity; no effort estimate established |

Giallo's manifest uses `onig-regset` and declares EUPL-1.2. Its renderer-neutral `HighlightedCode` returns lines of owned `HighlightedText` and associated style variants. It normalizes CRLF and lone CR to LF internally, so original-input offsets need separate accounting. Tokenizer modules are private in the public crate surface inspected. Default highlighting does not silently fall back to plain text. These facts follow from [Giallo manifest](https://github.com/getzola/giallo/blob/148fec043603627a226115880efd79a73bb454cd/Cargo.toml), [registry](https://github.com/getzola/giallo/blob/148fec043603627a226115880efd79a73bb454cd/src/registry.rs), [token output](https://github.com/getzola/giallo/blob/148fec043603627a226115880efd79a73bb454cd/src/highlight.rs), and [public exports](https://github.com/getzola/giallo/blob/148fec043603627a226115880efd79a73bb454cd/src/lib.rs). Its claim of matching VS Code is upstream positioning; this research did not independently establish full parity.

Syntect separates parsing and themed highlighting. The converter's own documented limitations specifically affect mainstream Markdown, shell, and HTML paths. This makes conversion materially different from running Shiki's tokenizer. [Syntect 5.3 documentation](https://docs.rs/syntect/5.3.0/syntect/), [syntect-tmlanguage 0.1.0 documentation](https://docs.rs/syntect-tmlanguage/0.1.0/syntect_tmlanguage/)

## Licensing and assets

Shiki implementation code is MIT. Preserve the relevant copyright/license notices when distributing substantial portions. [Pinned Shiki license](https://github.com/shikijs/shiki/blob/2aec8566cf186a92f4e094e6a92c335de3181ecd/LICENSE)

The grammar/theme collection keeps asset-specific licenses. Its general README describes permissive assets, but the actual inspected NOTICE inventory includes other labels: `aurora-x.json` is marked GPL-3.0; GitHub themes and Nord are marked MIT. Grammar entries include GPL-3.0, MPL-2.0, and `NOASSERTION`, among others. `NOASSERTION` is an inventory gap, not a license grant or proof of incompatibility. Select and review the exact shipped files and their embedded dependencies, rather than inferring their terms from the package's MIT label. [Theme NOTICE](https://github.com/shikijs/textmate-grammars-themes/blob/fde14af8c3612d6b966f2d3b7a86d3a79bb396e9/packages/tm-themes/NOTICE), [Grammar NOTICE](https://github.com/shikijs/textmate-grammars-themes/blob/fde14af8c3612d6b966f2d3b7a86d3a79bb396e9/packages/tm-grammars/NOTICE)

Giallo's EUPL-1.2 is a separate adoption consideration; do not assume it has the same distribution terms as Shiki's MIT code. This note records the declared license, without concluding how a particular product's distribution must be licensed. [Pinned Giallo license](https://github.com/getzola/giallo/blob/148fec043603627a226115880efd79a73bb454cd/LICENSE)

## Recommended evaluation order and remaining uncertainty

1. Validate the smallest upstream Shiki token build in the real application runtime, using the actual required languages and themes. Keep strict regex handling.
2. Adapt token output to native text spans with explicit Unicode and line-ending accounting. Preserve styling that the target renderer can actually represent.
3. Move work to a persistent service with revision-aware result publication. Validate multiline continuation and stale-result rejection before pursuing editor incrementality.
4. Measure cold initialization, warm tokenization, transfer/conversion, memory, and rendered scrolling separately. No upstream performance claim establishes GPUI frame latency.
5. Consider a native Oniguruma scanner or native TextMate engine only if measured runtime, parity, deployment, or licensing constraints justify the additional implementation.

Still unverified here: full bundled-language compatibility in the embedded runtime, semantic equality of alternate engines, production cancellation behavior, incremental convergence with all required grammars, end-to-end GPUI style fidelity, and asset license inventory for a final pinned distribution. This note supplies source-level constraints; it does not claim a completed Rust port or production component.
