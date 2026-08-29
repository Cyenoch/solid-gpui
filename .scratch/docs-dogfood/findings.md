# Onboarding dogfood findings

## Initial path

I followed `README.md` and then `docs/getting-started.md` from an external scratch directory (`/tmp/dogfood-app`). The first documented command was `bun init -y`, which completed with Bun 1.4.0. The next documented command was `bun add react @react-gpui/core`.

## Findings

### F-001 — core package cannot be installed from the documented registry command

- **What I did:** Created `/tmp/dogfood-app` with `bun init -y`, then ran `bun add react @react-gpui/core` exactly as `docs/getting-started.md` instructs.
- **What the docs said:** A consumer package needs React 19 and the core package; install with `bun add react @react-gpui/core`.
- **What happened:** Bun 1.4.0 resolved the public `react` package metadata but failed with `GET https://registry.npmjs.org/@react-gpui%2fcore - 404`. No application or host command could be reached from a new-user install because the package is not published at that registry location.
- **Severity:** blocker
- **Troubleshooting cross-reference:** Pending review of `docs/troubleshooting.md`; the onboarding docs do not currently explain the pre-publish/local-path requirement.

### F-002 — the host step assumes a repository checkout and Cargo context

- **What I did:** After building the package locally and installing it into the scratch app, ran the documented process-host command with the scratch entry from the repository checkout. The command reached the host and emitted its startup diagnostic, then stayed alive until the bounded smoke timeout.
- **What the docs said:** During local development, build and run the host from the repository with `cargo run -p react-gpui-host -- --runtime process bun run path/to/counter.tsx`.
- **What happened:** The command works only when the user has a checkout of this repository and invokes it from that checkout (or supplies an equivalent Cargo manifest context). A new consumer with only the app directory and a release host binary has no documented way to obtain the pre-publish host/package pair. The guide should state the checkout/build prerequisite and distinguish it from the future published/release flow.
- **Severity:** major

### F-003 — “install the pinned toolchain” lists versions but no installation or verification step

- **What I did:** Followed the guide on a machine where Bun 1.4.0 and Cargo were already available. `bun init -y` completed, and the host build later succeeded.
- **What the docs said:** “Install the pinned toolchain” followed only by the Bun and Rust version numbers.
- **What happened:** The guide assumes the reader already knows how to install or select those exact versions. On a clean machine, the first actionable command is unavailable, and no `bun --version`/`rustc --version` check is provided. The guide should call out these prerequisites and provide a verification command without prescribing a version manager.
- **Severity:** major

### F-004 — package README JSX example uses an undefined transport

- **What I did:** Compared the package README’s first runnable JSX example with the complete getting-started example.
- **What the docs said:** The package README imports `createRoot`, then calls `createRoot(transport, { surfaceId: 1, epoch: 1 })`.
- **What happened:** `transport` is never declared or imported, so copying the package README example into a new app fails before rendering. The onboarding example needs to construct `StdioTransport` and include the process termination handler (or explicitly define a transport supplied by the caller).
- **Severity:** major

### F-005 — linked examples are not directly runnable from a consumer app

- **What I did:** Followed the small-app steps to use the linked `todo.tsx`, `focus-flow.tsx`, and `dropdown.tsx` examples as composition references. A consumer app that copies one of these files encounters repository-relative imports such as `../src/index` (and example-only theme imports), which do not resolve from the app directory even after installing `@react-gpui/core`.
- **What the docs said:** “The examples are complete runnable entries; use them as the source of truth for the composition rather than copying a large tutorial listing.”
- **What happened:** They are runnable from this repository, but not verbatim in an external consumer app. The guide needs to identify repository-only imports and tell consumers to replace them with `@react-gpui/core` and copy/adapt any example-local helpers.
- **Severity:** major

## Fixed-path verification

From a new clean directory (`/tmp/dogfood-fixed`), I re-walked the corrected
steps: `bun init -y`; package build and tarball creation from the checkout;
`bun add react file:/tmp/react-gpui-package/react-gpui-core-0.2.0.tgz`; then the
external two-view app with TextInput, a state-driven Pressable style, and a
50-row VirtualList. The host command using `--manifest-path` and the explicit
renderer separator reached `react-gpui-host` and emitted:

```text
react-gpui-host: starting mode=Process protocol=v3 entry=bun pid=56666
```

The bounded smoke run ended with timeout status `124`, as expected for a
long-running GUI process in this headless shell. No install, TypeScript, or
host-startup error remained on the corrected path.
