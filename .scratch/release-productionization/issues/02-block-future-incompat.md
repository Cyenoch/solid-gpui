# block 0.1.6 future-incompatibility warning

Status: ready-for-human
Type: task

Cargo's future-incompatibility report identifies `block v0.1.6`'s
`_NSConcreteStackBlock: Class` declaration as a static of an uninhabited type.
Rust issue #74840 says this is being phased out and may become a hard error.

The dependency chain is the Apple GPUI platform path:
`react-gpui-host -> gpui_platform -> gpui_macos/gpui_apple -> cocoa`,
`cocoa-foundation`, `core-video`, and `metal` -> `block 0.1.6`. The same
`core-video` path is also reachable through the workspace `gpui` dependency.

`cargo info block` and docs.rs show that crates.io's latest release is still
`block 0.1.6`; the published release line is 0.1.x and there is no `block = "1"`
release to force through a direct workspace dependency. The upstream
`SSheldon/rust-block` repository still declares version 0.1.6 and the
uninhabited static.

Status is `ready-for-human` because the available code fix is only an unsigned
third-party fork (`Dicklesworthstone/rust-block` revision
`b39ae859d1ee8e8cb5eef6a516471f1578d26b96`). It is not an official crates.io
release or upstream `SSheldon/rust-block` release, so adding a `[patch]` would
replace a transitive dependency with an unreviewed supply-chain source. Do not
introduce that patch automatically; resolution should come from a GPUI upstream
update or an official/forked `block` release approved by a maintainer.

## Comments

- `cargo report future-incompatibilities --id 3` reports the uninhabited static
  at `block-0.1.6/src/lib.rs:64` and links Rust issue #74840.
- `cargo tree -i block@0.1.6` shows the Apple GPUI/cocoa/core-video/metal
  chain above.
- `cargo search block --limit 5` and `cargo info block` report crates.io
  `block = 0.1.6` as the latest published version.
- No root `Cargo.toml` or `Cargo.lock` dependency override was applied.
