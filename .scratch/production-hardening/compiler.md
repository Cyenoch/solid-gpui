# JSX compiler choice for Bun and QuickJS

Date: 2026-09-07

## Decision

Use the official Oxc-based `@solidjs/compiler` pinned to `2.0.0-rc.6`, with `generate: "universal"`, `moduleName: "@solid-gpui/core/runtime"`, and `builtIns: []`. Keep the existing Solid 1.9.15 runtime. Lower the resulting TypeScript with `oxc-transform` 0.148.0, preserving runtime imports and JavaScript class fields. Compose both source maps with `@jridgewell/remapping` 2.3.5. The Bun preload and Vite plugin share `packages/solid-gpui/src/vite/transform.ts`.

This removes the Babel application compiler, its presets and their ambient declarations. It introduces no handwritten JSX compiler, React compatibility layer or fallback compiler. Repository formatting is a separate Oxfmt responsibility, described below.

## What Oxc supports

The general Oxc JSX transformer exposes React-style automatic/classic runtimes and JSX preservation. A custom `importSource` alone does not provide Solid's reactive getters, effects or universal renderer operations. [Oxc JSX documentation](https://oxc.rs/docs/guide/usage/transformer/jsx)

The community `solid-jsx-oxc` implementation currently documents universal mode as an alias for DOM mode. That is insufficient for GPUI. [Project README](https://github.com/frank-iii/solid-jsx-oxc)

The official Solid compiler is different: it implements universal JSX and accepts a custom renderer module. Its README labels it Solid 2.0 release-candidate software and recommends exact version pins. The supported integration is the JavaScript `transform` interface; its Rust API remains unstable. [Official compiler README](https://github.com/solidjs/solid/blob/next/packages/compiler/README.md)

The npm registry currently assigns `latest` to `2.0.0-rc.2` and `next` to `2.0.0-rc.6`. The installed release is explicitly pinned to rc.6; it ships native optional platform packages and a WASI package. No moving dist-tag is used in the manifests. [Official npm metadata](https://registry.npmjs.org/@solidjs%2fcompiler)

## Compiler and runtime boundary

The compiler executes during development or bundling under Bun. It is not shipped as an application runtime dependency inside the generated JavaScript bundle and does not need to execute within QuickJS.

The compiled application still needs Solid's reactive runtime, this project's universal runtime and the selected transport. Selecting Oxc does not implement QuickJS transport, event-loop scheduling or host APIs; those are separate runtime integration responsibilities.

The official compiler defaults to auto-importing Solid 2 control-flow components. Setting `builtIns: []` leaves the application's explicit Solid 1 imports in place. The local A/B comparison confirms compatibility for the renderer's exercised helper contract, rather than claiming general Solid 1 compatibility for every feature in the Solid 2 compiler. The compiler pin must remain covered by behavioral checks when updated. [Compiler options](https://github.com/solidjs/solid/blob/next/packages/compiler/README.md#options)

## TypeScript and source maps

The official JSX transform preserves TypeScript syntax, as verified with a missing type-only import, a `declare` field and an ordinary optional field. Its output cannot be handed directly to a JavaScript-only loader.

Oxc's TypeScript pass removes explicit type imports with `onlyRemoveTypeImports: true`. Its default class-field behavior preserves JavaScript fields and erases `declare` fields. These choices retain the previous effective Babel 8 behavior and avoid silently removing an imported module's side effects. [Oxc TypeScript documentation](https://oxc.rs/docs/guide/usage/transformer/typescript)

The final source map composes JavaScript-to-generated-TypeScript and generated-TypeScript-to-authored-TSX mappings, with original source content retained. Maps are passed to Vite and embedded into the Bun preload's output. [Remapping API](https://github.com/jridgewell/sourcemaps/tree/main/packages/remapping)

## Local validation

The throwaway comparison installed the candidate compiler in a temporary directory, leaving workspace dependencies unchanged until it passed. Its baseline used `@babel/core` and `@babel/preset-typescript` 7.29.7 with `babel-preset-solid` 1.9.15 and explicit `onlyRemoveTypeImports` / `allowDeclareFields` options.

An actual `createRoot` + `MemoryTransport` application was compiled independently by both pipelines. The comparison exercised:

- Reactive component properties, text and styles; both getter-backed and function-result spreads.
- Explicitly imported `Show`, `For`, `Switch` and `Match`, plus conditional JSX.
- Three stable row identities, reorder/removal, and repeated signal updates.
- Solid owner presence and disposal callbacks.
- Erased type-only imports/declared fields and retained ordinary class fields.

All six decoded protocol-frame groups were deeply equal. Both runs created three owned row components, cleaned up rows in the order `2, 3, 1`, cleaned up the application once and retained only the ordinary class field. This compares application behavior, not textual compiler output.

All 26 current application TSX files compiled successfully. The combined targeted run passed: 5 tests, 82 assertions across hot reload, native export and Gallery. This includes every routed Gallery page, actual Rust native-binding generation, live Vite dependency replacement/error recovery/state restoration/binary stdio, and TypeScript import/field semantics in both the direct Bun preload and Vite paths. Tools TypeScript checking, frozen lockfile installation and `git diff --check` passed. Full package CI and consumer bundling run after the parallel runtime integration finishes.

The added source-map regression traces a generated `new Error` expression through both compiler passes to its exact authored TSX line and column and confirms the original source content is retained.

## Measured compile cost

On this development machine, Bun 1.4.2 compiled all 26 application TSX files (221,003 source bytes). Both pipelines generated source maps; the Oxc measurement included TypeScript lowering and final map composition. Files were preloaded before timing. After three warmup iterations, seven interleaved in-process samples produced:

| Pipeline                                   | Median per corpus |   Sample range |
| ------------------------------------------ | ----------------: | -------------: |
| Babel 7 + Solid preset 1.9.15 + TypeScript |          82.37 ms | 79.52–89.50 ms |
| Shared official Solid/Oxc pipeline         |          14.85 ms | 14.24–19.09 ms |

The observed compiler-stage reduction was about 5.5×. This is a local warm transform measurement; it does not measure cold startup, native binary loading, bundling, QuickJS execution or native UI frame performance. The compiler remains a release candidate even though this bounded compatibility probe and the existing application corpus pass.

## Formatter migration

The subsequent September 7 formatting migration pins `oxfmt` to `0.66.0` and replaces direct Prettier dependencies and command invocations. The root `.oxfmtrc.json` preserves the existing 120-column width, double quotes, semicolons, and trailing commas. Import and package-field sorting are disabled, keeping this migration about formatting. Existing `format` and `package-format` tasks remain checks over their established file scope, with the Oxfmt configuration included.

Oxfmt's CLI writes by default; verification uses `--check`. The local pre-commit skill follows the official lint-staged command, `oxfmt --no-error-on-unmatched-pattern`, and shares the repository configuration. [Oxfmt CLI](https://oxc.rs/docs/guide/usage/formatter/cli), [migration guide](https://oxc.rs/docs/guide/usage/formatter/migrate-from-prettier)

TypeScript and JSON formatting use Oxfmt's native implementation. The unused `scripts/format-generated.ts` helper was removed. Oxfmt itself bundles Prettier support for other formats, including Markdown; removing the project's direct Prettier tooling does not remove every upstream Prettier implementation. This change does not alter the JSX compiler, Solid runtime, or the earlier compiler benchmark. [Oxfmt language support](https://oxc.rs/docs/guide/usage/formatter/language-support)

Migration verification is recorded separately in [report.md](report.md#formatter-migration-follow-up).
