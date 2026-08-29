# Standalone archive distribution verification

Date: 2026-08-30
Verdict: PASS for the standalone process-host pair (release archive host + packed `@react-gpui/core` tarball + external app).

## Artifacts

- Host archive: `dist/react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz`
- Host archive SHA-256: `e08298d124d813de8b25713460a4c538c0281c06c191bbec7f2a8f3e09b4ae7b`
- Packed package: `/tmp/dist-verify/package/react-gpui-core-0.2.0.tgz`
- Packed package SHA-256: `49434a6085e96d80ebe511237109978298ec2a1c94bb004b83206a65437de1c8`
- Extracted host binary SHA-256: `e89bb382118529c4c04af993403090d8c91ce4473b06c0f3073c66e1f200bfd9`

The archive was rebuilt by `make host-candidate-smoke` before extraction. Its own gate passed, including archive reproducibility, extracted checksum verification, snapshot commit, timeout status 124, protocol-v3 startup diagnostic, help, and version checks.

## Method and commands

All external-app host runs below used working directory `/tmp/dist-verify/app`. No Cargo command or repository path was present in the host launch command.

```sh
rm -rf /tmp/dist-verify && mkdir -p /tmp/dist-verify/host /tmp/dist-verify/app /tmp/dist-verify/package

# From the repository root:
make host-candidate-smoke
shasum -a 256 dist/react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz

# From packages/react-gpui:
bun install --frozen-lockfile
bun run build
bun pm pack --destination /tmp/dist-verify/package --quiet
shasum -a 256 /tmp/dist-verify/package/react-gpui-core-0.2.0.tgz

# From /tmp/dist-verify/app:
bun init -y
bun add react "file:/tmp/dist-verify/package/react-gpui-core-0.2.0.tgz"

# Archive extraction:
tar -xzf /path/to/vue-gpui/dist/react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz -C /tmp/dist-verify/host

# Check the extracted binary's supported options:
/tmp/dist-verify/host/react-gpui-host-0.2.0-aarch64-apple-darwin/react-gpui-host --help

# External app startup/run smoke (bounded at 6 seconds, then SIGTERM):
REACT_GPUI_LOG=info /tmp/dist-verify/host/react-gpui-host-0.2.0-aarch64-apple-darwin/react-gpui-host \
  --runtime process -- bun run app.tsx
```

The external app was a real app importing `@react-gpui/core` from the packed tarball and rendering a `TextInput`, a `Pressable` button, and a 20-row `VirtualList`.

## Evidence

The bounded process run remained alive for 6 seconds and was terminated with SIGTERM (no startup or renderer error). Stderr contained:

```text
react-gpui-host: starting mode=Process protocol=v3 entry=bun pid=58321
```

The extracted archive binary reported:

```text
react-gpui-host 0.2.0 protocol=v3
```

and its `--help` output included `--runtime process`, the renderer separator `--`, and `--smoke-press`.

`--smoke-press` is an embedded-runtime smoke option, not a process-mode custom-entry interaction option. Running it against the external entry therefore correctly reported:

```text
react-gpui-host: embedded runtime is not compiled; use `--features embedded-bun`
```

To prove that the external process run emitted a real renderer commit while keeping the tap environment host-local, the following additional bounded run was made:

```sh
REACT_GPUI_LOG=info REACT_GPUI_TAP=/tmp/dist-verify/host-inbound.tap \
  /tmp/dist-verify/host/react-gpui-host-0.2.0-aarch64-apple-darwin/react-gpui-host \
  --runtime process -- env -u REACT_GPUI_TAP bun run app.tsx
```

It timed out cleanly after 6 seconds and recorded an inbound `snapshot` protocol record (`bytes=1376`, `seq=1`) from the external package app. This verifies startup plus initial renderer/host protocol exchange externally. The archive has no embedded runtime, and process mode exposes no command-line press injector, so a native button press could not be injected headlessly; no `--smoke-press` interaction claim is made for this archive.

## Documentation

Updated `docs/getting-started.md` so the pre-publish host step documents both equivalent choices:

1. checkout Cargo invocation using `--manifest-path`; or
2. extracted release archive binary invocation from the external application directory.

## Cleanup

The temporary verification tree under `/tmp/dist-verify` was removed after recording these notes.
