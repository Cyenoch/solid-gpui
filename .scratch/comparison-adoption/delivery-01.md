# Ticket 01 delivery: native acceptance and Unicode input

Branch: `adopt/01-native-acceptance`. Baseline `e879b6625d2590fc5bf1d74c6003e5919b21e92d`
was verified as HEAD and an ancestor before changes. This delivery modifies only
the testing seam, opt-in native runner/paint observation hook, and related guides.
No canonical renderer wire/schema/style fields changed; ticket 04 owns those.

Implementation commit: `e3620cdf`. Merged integration tip `91a63761` into this
branch as `579b7e89`; resolved only the changelog conflict by preserving both
ticket entries. Post-merge core typechecking passed; combined focused
core/native suites passed 53 tests/1400 assertions, including GPU capture, and
website content/Markdown/catalog suites passed 5 tests/3137 assertions. No native
source changed in the integration merge, so the previously built binary matches
the merged native source. Ticket 09 received the committed interface through its
explicitly authorized session. The integration branch was not mutated by this owner.

## Exact public interface for ticket 09

Import `NativeAcceptance` from `@solid-gpui/core/testing` (existing package export).

```ts
const host = await NativeAcceptance.launch({
  command: [acceptanceExecutable], mode: "deterministic", // or "gpu" on macOS
  cwd, timeoutMs: 30_000,
});
const app = mountReferenceStudio(host.transport, 1);
await host.flush();
const target = await host.locate({ label: "studio.history" });
await host.wheel(target, { x: 0, y: -120 });
const snapshot = await host.snapshot(1);
const offset = snapshot.nodes.find(node => node.id === target.id)?.scrollOffset;
const input = await host.locate({ label: "studio.draft" });
await host.click(input);
await host.key("secondary-a");
await host.type("新值🙂");
const nativeValue = (await host.locate({ label: "studio.draft" })).input?.value;
app.dispose();
const cleanup = await host.close(); // { surfaces: 0, windows: 0, popups: 0 }
```

Labels above illustrate the shape; use ticket 09's actual manifest labels.

- `launch({ command: readonly [string, ...string[]], mode: "deterministic" | "gpu", cwd?, timeoutMs? })` appends `--native-acceptance <mode>`. The executable initially owns Surface 1. Always mount with explicit Surface ID 1, since JS auto-allocation advances across tests.
- `capabilities`: `{ version: 1, mode, platform, screenshots, clock }`.
- `transport`: normal production protocol producer/event routing boundary; use any real app mount, not MemoryTransport or TestHost geometry.
- `flush()`: commits→native draw→events→Solid→feedback commits→native paint, bounded to 32 feedback turns. Await application timers separately. Native timers can be advanced with `advanceClock(milliseconds)` (0–60,000 per action).
- `snapshot(surfaceId = 1)`: `{ surfaceId, epoch, revision, nodes }` from native paint; each node has `{ surfaceId, epoch, revision, id, parentId, kind, listenerId, label, text, bounds: {x,y,width,height}, input, scrollOffset, selectedText }`. Bounds are content-mask-clipped logical native layout boxes. Only elements that actually reach paint with nonempty visible bounds are returned. No fake layout arithmetic or screenshot OCR.
- `locate({ id?, label?, text? }, surfaceId = 1)`: exact criteria, exactly one painted match; labels use existing accessibilityLabel. Reacquire targets after any application commit; captured targets reject stale revision/epoch/listener/node identities. IDs are stable while mounted.
- `click(target)`: native pointer move/down/up at painted center, through GPUI hitboxes. Clicks on non-listening child Text can reach the real ancestor Pressable, proving no direct callback injection.
- `type(text, surfaceId = 1)`: native Unicode keystrokes into native focus; 512 scalar values per action. `key(keystroke, surfaceId = 1)` dispatches one native GPUI keystroke (`secondary-a`, `secondary-c`, etc.).
- `drag(target, destination: {x,y}, options?: { from?: {x,y} })`: explicit start must be within painted target bounds, otherwise center; 8 real pointer moves then mouse-up. Use `from` near the start of a paragraph for selection.
- `wheel(target, {x,y})`: real pixel wheel delta. Negative y moves content upwards. `scrollOffset` is positive native VirtualList scrollbar offset, null for other kinds. Ordinary overflow scrolling is validated by painted content displacement.
- `input`: null except core TextInput; `{ value, selectionStart, selectionEnd, reversed, editSeq, focused, markedStart, markedEnd }` reflects host-owned editing state. Selection offsets are UTF-16. `text` contains display text/placeholder, while `input.value` is the actual value.
- `selectedText`: native existing per-node selectable Text selection, null otherwise. Renderer-wide selection belongs to ticket 06's public native module; use its public methods/events plus native drag/key and clipboard assertions. Do not mirror that state in JS automation.
- `clipboardText()`: GPUI's owned test clipboard in both modes, isolated from user clipboard; returns string|null. Native `secondary-c` has actual input/selection handlers.
- `screenshot(surfaceId = 1)`: `{ width, height, png: Uint8Array }` from Metal native rendered scene in physical pixels. Deterministic mode errors. Bound: 4 million physical pixels and 2 MiB PNG bytes.
- `close(): Promise<NativeCleanup>`: idempotent including concurrent calls, paints empty frames, removes all app windows, drops registry surfaces/popups/root entities, closes owned child process; returns zero cleanup counts and requires exit code 0. `root.unmount()` alone only detaches JS; always close host in finally. For an in-session removal assertion commit conditional content removal; for remount use epoch 2.

Stock Rust command:

```sh
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR=/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-gpui-adoption-target
cargo build -p solid-gpui --features native-acceptance --bin solid-gpui-acceptance --offline
```

Stock runner is NoExtensions. For ticket 09's generated components/media modules,
add a dedicated application acceptance executable that calls
`solid_gpui::host::acceptance::run(the_same_production_HostProfile)` with Cargo
feature `solid-gpui/native-acceptance`. Keep the regular host startup unchanged.
The profile's production init/registry/window root and command routing are reused.
No canonical wire generation is required for this ticket.

## Implementation and ownership

- Correct TestHost default offsets to JS UTF-16 `text.length`; emit native-order change then selection. Expose committed `inputState` to verify ackEditSeq/selection without private decoder imports. Key test uses combined CJK+astral text and controlled echo.
- Dedicated Cargo `native-acceptance` feature/bin and `host::acceptance::run(profile)` reuse production NativeStateRegistry/SolidRoot admission, extension catalog, layout, paint and event paths. Native points pass to Window::dispatch_event; text/keys use native dispatch_keystroke.
- Paint observer wraps actual renderer elements, preserves their native layout IDs, records clipped bounds only during paint, and has no production cost when the opt-in Cargo feature is absent. No fake styles, DOM runtime, pointer serialization, or N-API pump.
- Separate bounded, versioned length-prefixed JSON automation packets carry canonical framed commit/event bytes. Only an explicitly launched process enables it; no listener is automatically enabled from piped stdin. Bounded queues: 256 commits/4 MiB, 4096 events/4 MiB, 16 MiB packet, 32 feedback turns. One owned native foreground context per process; process input is lock-step, so autonomous timers require advanceClock/actions.
- Epoch/revision/listener target checks preserve callback generation safety. Owned empty-frame teardown uses the byte/native host seams.

## Checks and evidence

Executed on macOS Apple Silicon with linked vendored `gpui-pre 0.3.7`, dev/test profiles:

- Core TypeScript check: `bun node_modules/typescript/bin/tsc -p packages/solid-gpui/tsconfig.json` passed.
- Core focused semantic suite: `bun --conditions=browser --conditions=solid-gpui-source test packages/solid-gpui/tests/testing.test.ts packages/solid-gpui/tests/renderer.test.ts`: 49 pass, 0 fail (1369 assertions).
- Public native suite with actual built executable and GPU opt-in: 4 pass, 0 fail (31 assertions). Covers update→paint→hit-test→callback→paint; stable node identity; CJK+astral native edit/caret and clipboard copy; drag selection/copy and follow-up action; real overflow content displacement and VirtualList offset; stale revision/deleted content/remounted epoch; no direct callback routing from non-listening child locator; opt-in refusal; deterministic screenshot unsupported; Metal changed-pixel capture; cleanup count and zero exit.
- `cargo test -p solid-gpui --features native-acceptance --test host_command_roundtrip --offline`: 18 pass, 0 fail.
- `cargo check -p solid-gpui --lib --offline`: passed with automation feature absent.
- Website content/highlight + Markdown + catalog/translation focused tests: 5 pass, 0 fail (3136 assertions).
- Website typecheck attempted: blocked only by absent derived `examples/website/src/wasm/solid_gpui_web.js` in this isolated checkout. No fake declaration added. Integrator runs normal build:host/build:frontend after combined bindings/WASM generation. Route generation introduced no tracked changes.

The initial Unicode test was red before the helper fix; the native test exposed
JS unmount versus native process lifetime and was corrected to assert committed
removal plus explicit cleanup. No performance timing was run during shared builds.

Deterministic native mode uses GPUI TestAppContext native layout/scene/input with
test-platform text/OS services. GPU mode uses macOS VisualTestAppContext and
real offscreen Metal rendering. Linux/Windows GPU explicitly reject because this
linked GPUI visual context is macOS-only; those deterministic targets are not
locally qualified. Physical foreground mouse/trackpad/IME, display presentation,
and performance claims are not established by these tests. Custom production
module profiles must be qualified by their consuming applications.

## Documentation synchronized

Authoritative English + explicit Chinese `docs/native-acceptance` guides, Vite
testing guide/translation Unicode/API fixes, docs index, SDK README, changelog,
website reference navigation and Chinese label. Website imports those guides
directly. No generated renderer/native contract declarations changed.

## Integration commands and coordination

Owner commits before merging `integrate/comparison-adoption` into this branch;
the integrator merges this branch, never the reverse direction by this owner.
Preserve ticket 09's core VirtualList AccessibilityProps forwarding fix. Shared
renderer touch points are module/optional observation field/init/render clearing
and one wrapper at render_node. Ticket 05 may need to reconcile observations with
retained render regions: acceptance draws must report current geometry/text,
including native-only edits and cached regions. Ticket 06 owns renderer-wide
selection observations; existing per-node selected_text observation can be adapted
when that native model changes.

```sh
git merge --no-ff adopt/01-native-acceptance
# Rebuild the acceptance binary from integrated sources, then:
SOLID_GPUI_ACCEPTANCE_BINARY="$CARGO_TARGET_DIR/debug/solid-gpui-acceptance" \
SOLID_GPUI_ACCEPTANCE_GPU=1 \
bun --conditions=browser --conditions=solid-gpui-source test packages/solid-gpui/tests/native-acceptance.test.ts
```

All work remains local. No upstream posting, publication, deployment, or physical
user clipboard/app modifications were performed.
