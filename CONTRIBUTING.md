# Contributing

React GPUI is a Rust and TypeScript monorepo. Keep changes focused on the
protocol, renderer, host, or development package that owns the behavior.

## Toolchain

Use the pinned toolchains from `.bun-version` and `rust-toolchain.toml`:

```sh
bun --version
rustc --version
```

Install each package with its frozen lockfile before running package commands:

```sh
cd packages/react-gpui && bun install --frozen-lockfile
cd ../react-gpui-dev && bun install --frozen-lockfile
```

## Verification

Run the ordinary gate from the repository root:

```sh
make ci
```

Changes that touch the embedded runtime also require:

```sh
make embedded-bun
```

For a protocol change, regenerate and review the checked-in golden vectors and
API snapshots, then run the relevant focused tests before the complete gate:

```sh
make protocol-golden-generate
make api-surface-generate
```

Do not regenerate fixtures to hide an unexpected change. Review the resulting
wire bytes or export names and kinds as part of the change.

## Documentation and examples

Keep application-facing behavior in `docs/getting-started.md` and field-level
wire details in `docs/protocol.md`. Update the package README when a public API
or supported example changes. Examples under `packages/react-gpui/examples/`
are source-tree smoke entries and are not included in published package files.

## Change hygiene

Use the existing lowercase commit prefixes (`protocol:`, `renderer:`,
`tests:`, `docs:`, `chore:`, and similar). Keep generated build output out of
commits, and run `git diff --check` before handing off a change. Do not push,
rebase, or rewrite shared history as part of a contribution.
