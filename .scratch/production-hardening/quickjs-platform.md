# QuickJS platform qualification

The embedded engine runs standard bundled ECMAScript, not Bun or a browser. The QuickJS application build explicitly evaluates `src/vite/quickjs-platform.ts` before application dependencies; Bun builds retain Bun's platform. No feature-detection fallback or network implementation is installed.

## Required router contract

Actual `@solid-gpui/router` bundles failed immediately in a bare VM because TanStack Router uses `URLSearchParams` during location parsing and `AbortController` for route matches. Its redirect factory constructs `Headers` and a null-body `Response`; its error path calls `isRedirect`, which performs an unguarded `instanceof Response`. Merely making the counter work did not qualify routing. These observations follow the installed [TanStack router source](https://github.com/TanStack/router/tree/main/packages/router-core/src), particularly `qss.ts`, `router.ts`, `load-matches.ts`, and `redirect.ts`.

The supported embedded platform is deliberately bounded:

- A worker-style `self === globalThis` alias, without a DOM or `window`. TanStack's production client entry uses `self` when constructing the router.
- URL and URLSearchParams from `core-js-pure` 3.50.0; Unicode query values, repeated parameters, and relative URLs are qualified.
- Event and EventTarget from `event-target-shim` 6.0.2, including listener identity, `once`, and abort event attributes.
- AbortController and AbortSignal adapted from pinned Vercel Edge Runtime source. Abort reasons, idempotent abort, `throwIfAborted`, static `abort`, and static `timeout` are supported. `AbortSignal.any` is not supplied. Timeouts accept integers from 0 through 2147483647 milliseconds, matching the embedded timer range.
- Headers adapted from pinned `fetch-headers` source, including validation, normalization, iteration, and `getSetCookie`.
- An explicit bodyless `Response` contract for native router redirects: construction, `status`, `statusText`, `headers`, `ok`, `body === null`, and `bodyUsed === false`. A non-null body throws a descriptive TypeError; invalid status or reason phrase is rejected. Fetch body readers, streams, static Response factories, and network fetch are not advertised.

This is a lightweight UI platform, not an implementation of the full Fetch API. Host services remain explicit native commands.

## Dependency and source choices

[`core-js-pure`](https://github.com/zloirock/core-js/tree/v3.50.0/packages/core-js-pure) provides maintained, side-effect-isolated URL implementations. The actual URL pair bundled to about 49 KB minified in a throwaway experiment. Latest `whatwg-url` 17.1.0 bundled to about 459 KB and initialized legacy TextDecoder encodings unsupported by this UTF-8-only engine; it was rejected for this platform.

[`event-target-shim`](https://github.com/mysticatea/event-target-shim/tree/v6.0.2) supplies the event algorithms used by upstream Vercel too. Its package omits an ESM types export, so the platform includes narrowly scoped declarations for the standard event interfaces and its event-attribute helpers. `core-js-pure` does not publish TypeScript declarations; the imported constructors have scoped standard interface declarations as well.

The Abort source is [Vercel commit 440c123a37284d6a852ce453af810ad484ecfc01](https://github.com/vercel/edge-runtime/blob/440c123a37284d6a852ce453af810ad484ecfc01/packages/primitives/src/primitives/abort-controller.js), the source of `@edge-runtime/primitives` 6.0.0. The package exposes this implementation as a source string, while its aggregate entry imports Node dependencies. Vendoring the source avoids eval and Node adapters. Local changes are explicit imports/types, the shim's event-attribute helpers (correctly clearing `onabort`), core-js DOMException, and timer-bound validation. Full MIT text and provenance are retained in `quickjs-abort.ts` and emitted application bundles.

The Headers source is [`fetch-headers` 3.0.1 commit 66d63ac7a67d3b9c863cd80b872f2ea999cbc7a7](https://github.com/jimmywarting/fetch-headers/blob/66d63ac7a67d3b9c863cd80b872f2ea999cbc7a7/headers.js). The source is self-contained and uses null-prototype storage. Local fixes remove stateful RegExp validation, preserve invalid interior newlines for rejection, validate original ASCII names before lowercasing, and validate `set` before mutating the old value. The unused Node inspection hook and internal bag export are removed. Full MIT text remains in `quickjs-headers.js` and emitted application bundles. Original npm tarball integrity: `sha512-Kq+NyED/wLgT29St7aW47gAWg8EmmE5QmhwQ5RmPRULYLqpglA7Kc/ZnbqXu2vhH6mw1koikew2g94WiHLPmpA==`.

The local fixes follow [Fetch header algorithms](https://fetch.spec.whatwg.org/#headers-class) and the [DOM abort algorithms](https://dom.spec.whatwg.org/#aborting-ongoing-activities). A regression covers the Kelvin sign `K`, which Unicode lowercasing turns into ASCII `k` but which must be rejected as an original HTTP header name.

The apparently maintained `headers-polyfill` 5.0.1 was experimentally rejected: it silently dropped whitespace-bearing values, returned inherited properties for names such as `__proto__`, and silently ignored invalid updates. Its [published source](https://github.com/mswjs/headers-polyfill/tree/v5.0.1) validates values before normalization and uses ordinary object storage. The dependency was removed, not patched behind a wrapper.

`whatwg-fetch` could construct the specific redirect response but auto-installs an unavailable XHR fetch and has observed invalid-header and UTF-8-body defects. `@web-std/fetch`, `@remix-run/web-fetch`, and aggregate Edge primitives depend on Node streams, buffer, or undici. `@gjsify/fetch` additionally requires GObject services. None is used.

## Qualification evidence

The complete platform bundles to 74,458 bytes minified, including both retained MIT notices. Normal application builds also contain Solid, protocol code, application code, and inline source maps, so this isolated measurement is not a final application size.

`fixtures/quickjs-router.tsx` is the real runtime fixture. It verifies Unicode search parsing and navigation, relative native URLs, loader search data, superseded loader cancellation, cancellation isolation, internal redirects, rendered loader errors, unsupported body rejection, header normalization and prototype-safe names, repeated invalid-header rejection, and failed-set atomicity. It then visits every Gallery page through its actual router and emits an EmbeddedTransport snapshot only after the assertions pass. The Rust worker executes this bundled fixture in the actual QuickJS VM.

Tools, core, and fixture TypeScript checks pass. All six actual QuickJS tests pass, including the complete router fixture and all 23 Gallery pages. The fixture uses the built package graph consistently; package building is already a prerequisite of the repository's Rust integration tests. Extensionless dist aliases select declarations for TypeScript and JavaScript for Bun without loading duplicate source/dist host registries.

The application build explicitly defines `process.env.NODE_ENV` as `"production"`. A bounded Bun 1.4.2 probe showed that merely adding a `production` export condition still selected Solid's development client, whereas the explicit definition selected `solid.js` and the production universal runtime. This agrees with [Bun's production build configuration](https://bun.com/docs/bundler). The engine's bounded 2 MiB JavaScript stack and 8 MiB worker stack, together with optimized QuickJS C code, pass the nested real Gallery components. Temporary stack-trace instrumentation was removed.
