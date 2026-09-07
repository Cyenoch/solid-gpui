# Embedded Gallery native acceptance

2026-09-06, macOS, the development build with `gpui-component` + `embedded-bun`.
The run used that investigation's Vite production bundle and rebuilt Bun dynamic
library. A temporary .app enabled native automation discovery; it was not a
release package.

- Host SHA-256: `94a16cd217ad97d6233d021f0e05132631ab2bbf8286bd468945d7de7c4ef582`
- Gallery bundle SHA-256: `4171c4f759fb6886d04a7b937d1b2f3656e606e6c2d1745efb957ce8f20eb54c`
- Bun library SHA-256: `9e9831857148c6b013ef46a0534ba00ed49653dfbbd7b3e4cd1666d0dae303c0`
- Entrypoint: `examples/gallery-vite/dist/main.js`, `--runtime embedded`, actual PID 84149.
- Initial logical window: 800×633, with further interaction after native zoom. CUA returned a 1304×768 screenshot; its pixels do not establish logical window size or device scale.

Observed interactions:

1. The native Build workspace button advanced progress from 0 to 20. Rust BuildBadge displayed `Rust component · 1 builds`.
2. Analyze in Rust returned `Untitled workspace`, `untitled-workspace`, and 20% progress.
3. Calling again with empty native input displayed Rust's `Workspace name must not be blank` error.
4. Calling with `  Rust   Studio  ` returned the normalized name `Rust Studio` and slug `rust-studio`; the error disappeared.
5. Clicking the native close button initiated background cleanup. The log recorded `runtime terminated status=Shutdown`, the application exited, and the launcher returned 0.

[Success screenshot](gallery-embedded.jpg), [startup and normal-exit log](gallery-embedded.log),
and [protocol recording](gallery-embedded.tap). These establish interaction,
application round trips, and lifecycle behavior, not scrolling performance or
display frame-rate acceptance.

The run also identified injected Clippy attributes being included in the macro's
contract digest. The fix captures developer declarations before adding compiler
support attributes. Regeneration through canonical native-codegen produced
matching component digests for the SDK and the host actually linked by Gallery.
