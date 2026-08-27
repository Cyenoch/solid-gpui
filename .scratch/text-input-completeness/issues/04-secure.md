# TextInput secure/password semantics

Status: upstream-gap

## Evidence

- `crates/react-gpui/src/protocol.rs:337-350` defines the complete native
  `TextInputProperties` tuple and has no secure/password field.
- The consumer README explicitly records `secureTextEntry` and `keyboardType`
  as unsupported because the desktop GPUI surface has no password-obscuring
  primitive (`packages/react-gpui/README.md:623-625`).
- The pinned GPUI `InputHandler` contract exposes ordinary text and marked text
  but no secure-display or password mode (`references/zed/crates/gpui/src/platform.rs:1669-1797`).

## Decision

Retain the existing upstream-gap documentation and do not add a renderer-level
masking fallback. A real secure primitive would require an upstream GPUI and
protocol design rather than hiding text only in this paint path.
