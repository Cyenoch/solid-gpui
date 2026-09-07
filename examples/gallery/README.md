# Native Gallery

Run `bun run dev` in this directory or `bun run gallery` at the workspace root.
The command builds the workspace packages and starts the application's Rust host
with the Bun renderer. Use `bun run gallery:vite` from the root for native hot reload.

`src/application.tsx` mounts the shared application in `src/gallery`; the Vite
Gallery imports that application through this workspace package. Solid owns the
route, theme, search, and demo state. GPUI owns native windows, editing state,
layout, and painting. The example's Rust module supplies the workspace analysis
command and build badge. See [native module authoring](../../docs/rust-bridge.md)
and [desktop distribution](../../docs/distribution.md).

## Language and input coverage

Navigation, control labels, documentation, and source comments are maintained in
English. The Native Controls page deliberately includes `Chinese (中文)` and
`Edit me · 中文` to demonstrate Unicode rendering and editing. Preserve those
samples and the IME, selection, and protocol Unicode fixtures when updating copy.
Optional translations belong in explicitly named files such as `README.zh-CN.md`.

## Screenshot

![The English Solid GPUI workbench on macOS, showing native workspace controls and three completed builds.](../../docs/images/gallery.png)

[`docs/images/gallery.png`](../../docs/images/gallery.png) was captured from the
native Gallery on macOS on September 7, 2026. It replaces the former React GPUI
image. The screenshot shows the Overview page in dark mode, a workspace named
`Desktop workspace`, and three presses of **Build workspace**. The live preview
and Rust build badge both reflect those interactions.

To refresh it:

1. Run `SOLID_GPUI_PERF_MONITOR=0 bun run gallery` from the repository root.
2. Open **Overview & Features**, select dark mode, and resize the window wide
   enough for the playground's two columns.
3. Enter `Desktop workspace` and press **Build workspace** three times. Verify
   the preview and the Rust badge both show three builds, then scroll to the top.
4. Capture only the actual native window and save it to `docs/images/gallery.png`.
   Keep labels readable, exclude desktop content, and update this capture record.

The current capture uses the macOS native-window capture API through the UI
automation tool. A temporary app bundle supplied the existing development host's
renderer command so the tool could address the window. The returned image was
converted to PNG without editing its contents. This image documents macOS native
rendering; Linux and Windows require their own visual verification.
