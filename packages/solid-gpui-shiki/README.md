# @solid-gpui/shiki

Shiki highlighting for selectable native Solid GPUI code blocks. The Bun backend
uses one persistent Worker and Shiki 4.4.2's Oniguruma engine. GPUI shapes and
paints the text; no HTML, DOM or WebView is involved.

```sh
bun add @solid-gpui/shiki @solid-gpui/core solid-js
```

```tsx
import { CodeBlock } from "@solid-gpui/shiki";
import { createBunHighlighter } from "@solid-gpui/shiki/bun";
import { onCleanup } from "@solid-gpui/core/runtime";

// Initialize once before mounting the application, then share the service.
const highlighter = await createBunHighlighter({
  languages: ["typescript", "tsx", "rust"],
  themes: ["github-dark", "github-light"],
});

function App() {
  onCleanup(() => highlighter.dispose());
  return (
    <CodeBlock
      highlighter={highlighter}
      code={'const message = "Hello, GPUI — 你好😀";\n'}
      language="typescript"
      theme="github-dark"
      style={{ fontSize: 14, lineHeight: 22 }}
    />
  );
}
```

Compile JSX with the Solid GPUI universal transform and run Bun with
`--conditions=browser`, as for other Solid GPUI components. Wrap the component
in `ErrorBoundary` from `@solid-gpui/core/runtime` to present highlighting errors.
If initialization fails before mounting, handle that rejected promise in the
application entrypoint. An application that never mounts must dispose its service.

## Component

`CodeBlock` accepts `highlighter`, `code`, `language`, `theme`, optional `style`,
`selectable` (default `true`), `accessibilityLabel`, and `onLayout`.

Code, language, theme and highlighter changes request a new result. Pending work
shows the current source without syntax styles, using the surrounding/caller's
text colors. Only a result for the current request is displayed. Errors reach the
Solid error boundary; they do not silently select another grammar or engine.
The resolved theme controls the completed block's foreground and background;
`style` controls layout and typography. The default font family is `monospace`;
choose an installed code font using `style.fontFamily` if needed.

Runs share one native paragraph, so selection and copying cross token and line
boundaries. Original Unicode and line endings are preserved. Tokenization runs
only when the highlighting inputs change, never during native scrolling or
resizing. Adjacent identical styles are coalesced. This is a bounded code block,
not a virtualized file editor: source is limited to 64 KiB of UTF-8 and output to
4,096 styled runs. The component does not add line numbers, folding, editing,
semantic tokens or DOM transformer support.

Colors, bold, italic, underline and strikethrough map to existing native inline
styles. A token combining underline and strikethrough, or requesting an inline
background, rejects because the current core inline style contract cannot express
it. Theme colors must resolve to hexadecimal values; CSS-variable themes are not
supported. Source containing unpaired surrogates rejects instead of being changed
by UTF-8 encoding.

## Service and lifetime

For static documentation, compute results in Bun during the build and render
`<HighlightedCode highlighted={result} />`. It accepts the same appearance props
as `CodeBlock` and does no asynchronous work or runtime loading. The documentation
website uses this path for every code example; browsers receive serialized results
and the renderer, without Shiki's tokenizer or Oniguruma WASM.

`createBunHighlighter({ languages, themes, timeoutMs? })` resolves after loading
the selected bundled assets and their grammar dependencies. Load 1–32 languages
and 1–8 themes. Unknown/unloaded language or theme requests reject. Shiki's loaded
language aliases and plain-text language are accepted by the backend.

```ts
const result = await highlighter.highlight(
  { code, language: "typescript", theme: "github-dark" },
  { signal: abortController.signal },
);
// result: original request + foreground, background, runs
// run: { text, color, fontStyle } (resolved Shiki font flag bits)
highlighter.dispose();
```

The backend executes one request at a time and allows 32 queued requests. Cancelling
a queued request removes it; cancelling an active request rejects that consumer
immediately while the worker finishes its bounded slot. This does not terminate a
shared service. The initialization/active-work deadline defaults to 30 seconds;
`timeoutMs` accepts integers from 1 to 300,000. Expiry terminates the worker and
rejects all requests. Create a new service explicitly after a terminal failure.
Shiki's soft line timeout is disabled so incomplete tokens are not published as
successful results. Worker termination supplies the hard deadline.

`dispose()` is idempotent, terminates the worker and rejects pending/future work.
A `CodeBlock` cancels its own request on change/unmount, but never disposes the
shared backend. The application owns backend disposal, including hot reload and
transport shutdown. No process-global highlighter or unbounded result cache exists.

`CodeHighlighter` and `CodeBlock` have no Bun imports. A future QuickJS backend can
implement the same asynchronous interface; **QuickJS is not supported today**.
The current backend is qualified for Bun child-process applications. Embedded Bun
and standalone compiled executables need separate runtime/asset qualification.

## Packaging and example

The runtime resolves `@solid-gpui/shiki/worker` from the installed package. Deploy
the package's `dist` directory and its Shiki dependencies with the application;
copying only an application bundle is insufficient. Keep `@solid-gpui/shiki` as
an installed application dependency even if the main application is bundled.
The worker subpath is an internal asset entrypoint, not a component to import.
Build tools supplying a separate Worker artifact can pass its URL as `workerURL`
to `createBunHighlighter`. The repository website points this at the package's
Worker source so its source-based Vite build needs no native package compilation.

From this repository:

```sh
bun run task package-build
cargo run -p solid-gpui --bin solid-gpui-host -- \
  bun --conditions=browser packages/solid-gpui-shiki/examples/code-block.ts
bun run task shiki-package-pack /tmp/solid-gpui-shiki.tgz
```

The example uses ordinary functions and needs no JSX compilation. It exercises
TypeScript/Rust, dark/light themes and native selection. Tests cover the actual
worker, Unicode/CRLF, the emoji/comment regression, invalid requests, cancellation,
disposal, and stale component results.

This package's code is MIT. Shiki and its bundled languages/themes retain their
own upstream notices; include those notices when distributing the selected assets.
