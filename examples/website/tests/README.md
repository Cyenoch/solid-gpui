# Component preview verification

Run `bun run --cwd examples/website test` for catalog coverage, TSX contracts,
SDK type checking, Markdown parsing, startup, and rendering without DOM globals.
These checks do not establish visual or interaction correctness.

Start the website development server and open
`/solid-gpui/tests/preview-audit.html#Tooltip`. The development-only harness mounts
the actual catalog example in the production `ComponentPreview` shell using the
GPUI WASM host. Main IDs are component names; variant IDs are catalog example IDs,
such as `Button--primary`. The title identifies the mounted example. An `ERROR:`
title indicates a transport failure.

Review all catalog compositions at a desktop width, then every main preview and
layout-sensitive variant at 390 px. Check empty content, clipping, contrast,
wrapping, code alignment, and horizontal overflow. Exercise each interaction
family: controlled state, text entry, focus cycling, nested scrolling, resizing,
menus, dialogs, hover surfaces, and clipboard actions. Navigate away with an open
overlay and confirm it disappears. Verify the complete documentation shell too;
this harness deliberately does not include its navigation or article layout.

After changing native Rust, rebuild the WASM host and reload the browser document.
A hash navigation alone keeps the previous WASM instance. Wait for the selected
preview to render before recording evidence; a blank frame during a Vite restart
is not a successful check. Desktop-only system integrations require a separate
desktop run and must not be counted as browser-verified behavior.

The September 2026 audit inventory and findings are recorded in
`.scratch/website-component-audit/` at the repository root.
