# Syntax highlighting with Shiki

`@solid-gpui/shiki` renders selectable code with GPUI's native text system.
The current live highlighting backend runs Shiki and Oniguruma in a persistent
Bun Worker. QuickJS is not supported yet.

## Install and initialize

```sh
bun add @solid-gpui/shiki @solid-gpui/core solid-js
```

Create one service per application or explicitly owned feature, before mounting:

```ts
import { createBunHighlighter } from "@solid-gpui/shiki/bun";

const highlighter = await createBunHighlighter({
  languages: ["typescript", "tsx", "rust"],
  themes: ["github-dark", "github-light"],
});
```

Use the same Solid GPUI JSX transform and Bun `--conditions=browser` setting as
the rest of your application. Handle initialization failure in the application
entrypoint; wrap rendered components in a Solid GPUI `ErrorBoundary` to display
highlighting errors.

## Render code

```tsx
import { CodeBlock } from "@solid-gpui/shiki";
import { onCleanup } from "@solid-gpui/core/runtime";

function App() {
  onCleanup(() => highlighter.dispose());
  return (
    <CodeBlock
      highlighter={highlighter}
      code={'const greeting = "Hello, GPUI!";\n'}
      language="typescript"
      theme="github-dark"
      style={{ fontSize: 14, lineHeight: 22 }}
    />
  );
}
```

Code, language and theme props are reactive. The component cancels superseded
requests and ignores late results. While a result is pending, it shows the current
source with the surrounding text style. The completed theme supplies foreground
and background; layout and typography come from `style`.

The component is selectable by default. All runs form one native paragraph, so
selection and copying cross token boundaries. Original Unicode and line endings
are retained. This is a read-only code block, limited to 64 KiB of UTF-8 source
and 4,096 coalesced runs. It is not a virtualized editor.

## Build-time highlighting

Known snippets can be highlighted during a Bun build and serialized as data:

```ts
const result = await highlighter.highlight({
  code: "const answer = 42;",
  language: "typescript",
  theme: "github-dark",
});
highlighter.dispose();
```

```tsx
import { HighlightedCode } from "@solid-gpui/shiki";

<HighlightedCode highlighted={result} />;
```

`HighlightedCode` only renders the supplied result. It has no Bun dependency and
does not start a worker. This project's documentation website uses that path:
Vite generates results through the Bun backend, and the browser's GPUI WebAssembly
host renders native text runs. The tokenizer and its Oniguruma WASM stay out of
the browser bundle.

## Ownership and distribution

Applications own service disposal, including unmount, hot reload and shutdown.
Components cancel their own requests without destroying a shared service. The
backend runs one active request with up to 32 queued requests. A hard deadline
terminates stuck work and rejects the service's requests; there is no silent
engine replacement or automatic restart.

Ship the installed package's Worker asset and Shiki dependencies alongside a Bun
application bundle. A single copied application JavaScript file is insufficient.
Embedded Bun and standalone executable packaging still need separate qualification.

See the [package README](../packages/solid-gpui-shiki/README.md) for API details,
supported styles, limits, Worker URL overrides, cancellation and the runnable
native example.
