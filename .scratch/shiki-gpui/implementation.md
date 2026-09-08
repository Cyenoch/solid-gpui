# Shiki integration delivery

Date: 2026-09-07. This supersedes the implementation status in the earlier
[research report](report.md); its upstream/runtime findings remain research evidence.

## Delivered

- `packages/solid-gpui-shiki` publishes `@solid-gpui/shiki`: asynchronous native
  `CodeBlock`, precomputed `HighlightedCode`, and a runtime-independent service
  contract. `@solid-gpui/shiki/bun` supplies one persistent Bun/Oniguruma Worker.
- Requests preserve Unicode and line endings, coalesce compatible runs, enforce
  source/output/queue limits, reject unsupported styles, support cancellation,
  reject stale completions and dispose explicitly. Deadlines expire the service.
- Workspace build/typecheck/test, package packing, export inventory, release
  version/peer synchronization, candidate artifacts and notices include the package.
- The documentation website uses the same package. Vite computes every static
  code source using Bun and sends only serialized results to the browser. The
  previous lexical regex highlighter was removed. Markdown retains fence language.
- [User guide](../../docs/shiki.md) and
  [package README](../../packages/solid-gpui-shiki/README.md) describe usage and limits.

## Verification

- Eight package tests passed: actual Oniguruma worker, emoji/comment regression,
  Unicode/CRLF/empty source, loaded languages/themes, invalid requests, active and
  queued cancellation, disposal, deadline expiry, stale component results,
  typography-only updates and native error-boundary delivery.
- Four website tests passed, including all code-source generation and component
  catalog type-checking. Website TypeScript check and production Vite build passed.
- Core/router/Shiki tarballs were installed in a fresh temporary consumer. Shiki
  ran directly and from a bundled consumer, produced a native highlighted Patch,
  resolved its installed Worker asset and passed consumer TypeScript checking.
- Release rollback/version synchronization and task-contract checks passed.
- Native preview ran with the existing macOS host and Bun 1.4.2. TypeScript and
  Rust highlighting rendered in the window; the user inspected and accepted the
  effect. This was visual/functional verification, not a native performance benchmark.
- Chrome displayed the built Shiki guide and Button component page. The guide
  was checked at the normal wide viewport and at 420 × 850, then the override was
  reset. Highlighted code and wrapping remained visible; no browser errors were
  recorded. Built browser assets contain the GPUI host WASM, not an additional
  Oniguruma WASM or a Bun worker/tokenizer implementation.

## Scope

Live tokenization is supported in Bun child-process applications. QuickJS,
embedded-Bun distribution and standalone executable assets are not qualified.
Read-only code blocks are bounded, not virtualized editors. No Rust rendering or
core wire schema changes were needed. Pre-existing workspace changes were retained.

The website build used the existing generated GPUI WebAssembly host; this task did
not rebuild or benchmark that host. Vite still reports its existing WASM-glue eval
and large-chunk warnings, plus extensionless config imports. No npm publication,
Git push or public-site deployment was performed.
