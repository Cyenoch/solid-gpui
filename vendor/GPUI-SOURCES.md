# Vendored GPUI source inventory

Verified against actual crates.io archives on 2026-10-02. These are patched
`gpui-pre` **0.3.7** packages. The package manifests declare Zed revision
`1a28cff4b409169bac058bca40dfbfeb7621d19b`; this is manifest provenance, not a
claim that each archive is a byte-for-byte copy of Zed's original workspace.
`Cargo.toml`, root dependency pins, and `Cargo.lock` agree on 0.3.7.

The core archive was read from Cargo's cached `.crate`; the six platform/client
archives were read from `https://static.crates.io/crates/<crate>/<crate>-0.3.7.crate`.
Archive SHA-256 and a file comparison establish the baselines below independently
of local version strings. Apache-2.0 license files are retained. GPUI Kit has a
separate [commit and adaptation inventory](gpui-kit/SOLID-GPUI.md).

| Directory / published crate | Original archive SHA-256 |
| --- | --- |
| `gpui` / `gpui-pre` | `0e87a42bb37c7cb4e76dd1ac0ce88851e46e976e0373a47ab3e0757abffee54d` |
| `gpui-web` / `gpui-pre-web` | `ab3870cc909471bb830c3d79496770c754401aad6cfa122147ccb13e8908ad08` |
| `gpui-wgpu` / `gpui-pre-wgpu` | `f0b02657b56b09140ce542f5e4434fb96d9fdaba0fd0035ba7dba13c171ddb9b` |
| `gpui-reqwest-client` / `gpui-pre-reqwest-client` | `53fa47d680f9e186b33af323f97345d67b6150c0e4c0bc894406bff822a4d9b4` |
| `gpui-macos` / `gpui-pre-macos` | `5a43af845b260b09393e923c4f1e1a67a10fcfc847b4815192bbfd02ed9fe725` |
| `gpui-windows` / `gpui-pre-windows` | `f05592a6f9e3e6bb7a9f2a0d4779271cf747a948db1c07b02448de777ffff8f3` |
| `gpui-linux` / `gpui-pre-linux` | `0aac7022e347409454777701082201742710052813964a1e25160f7e22c965ad` |

The comparison records the vendor tree at `e879b6625d2590fc5bf1d74c6003e5919b21e92d`
and excludes only the added inventory documents. All seven manifests add local vendor metadata;
Reqwest also updates dependency/features for upstream Reqwest. Published per-crate
`Cargo.lock` files are omitted because the consuming root workspace owns
resolution. Core and Reqwest retain byte-identical `Cargo.toml.orig`; the other
five packages omit it. No archive-supplied source or license file is missing.

| Directory | Changed source paths relative to the 0.3.7 archive | Added source/assets | Behavior inventory |
| --- | --- | --- | --- |
| `gpui` | `src/app/test_context.rs`, `src/elements/animation.rs`, `src/elements/div.rs`, `src/elements/img.rs`, `src/elements/list.rs`, `src/platform/popup.rs`, `src/platform/test/window.rs`, `src/platform.rs`, `src/style.rs`, `src/svg_renderer.rs`, `src/text_system/line_layout.rs`, `src/window.rs` | `src/window/bounds_tests.rs`; licensed IBM Plex Sans and Lilex font fixtures under `test-assets/` | [Core patches](gpui/PATCHES.md) |
| `gpui-web` | `src/window.rs` | None | [Transparency and resize scale](gpui-web/README.solid-gpui.md) |
| `gpui-wgpu` | `src/wgpu_renderer.rs` | None | [Browser premultiplied alpha](gpui-wgpu/README.solid-gpui.md) |
| `gpui-reqwest-client` | `src/reqwest_client.rs` | None | [HTTP pools and Tokio polling](gpui-reqwest-client/PATCHES.md) |
| `gpui-macos` | `src/gpui_macos.rs`, `src/platform.rs`, `src/window.rs` | None | [Popup, movement, blurred backgrounds](gpui-macos/SOLID-GPUI.md) |
| `gpui-windows` | `build.rs`, `src/direct_write.rs`, `src/directx_renderer.rs`, `src/events.rs`, `src/platform.rs`, `src/window.rs` | None | [Popup, embedded shaders, UI font, recovery](gpui-windows/SOLID-GPUI.md) |
| `gpui-linux` | `src/linux/wayland/client.rs`, `src/linux/wayland/window.rs`, `src/linux/x11/client.rs`, `src/linux/x11/window.rs` | None | [Popup and visual-state notifications](gpui-linux/SOLID-GPUI.md) |

These changes remain necessary for the contracts their inventories describe;
changing source references to stock crates would remove required behavior.
Source-build/onboarding improvements belong to the application toolchain rather
than dropping patches without equivalent upstream support.

The isolated [line-clamp cache fix and regression](../.scratch/comparison-adoption/upstream/README.md)
are prepared locally for review. No upstream issue, comment, or pull request has
been sent. Other changes couple multiple native/platform contracts and need a
separate upstream review before they can replace local patches. Refresh this file
and each behavior inventory together when updating the baseline or adding patches.
