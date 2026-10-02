# Recorded paint and live frames

`RecordedPaint` and `LiveFrame` are generated NativeView components exported by
`@solid-gpui/core/components`. The component host and website WASM host register
them through `solid_gpui::components::native_module()`. Custom hosts must register
that module and export matching bindings. These capabilities use the existing
Extension and native-call contracts.

Both components declare behavioral version `1.0.0`. Generated bindings include
the canonical contract identity and the selected host's build identity, and wrap
props and ref-command arguments in the strict native request envelope. Direct
Rust fixtures use `encode_native_request(module.build_digest(), &dto)`; missing
envelopes or stale builds are rejected before admission. See
[native contract identity](rust-bridge.md) for host registration and regeneration.

## Recorded drawing

Solid constructs a `PaintRecording` reactively. Native code validates the complete
recording before publishing the tree, prepares path geometry when the recording
changes, and replays it during GPUI paint. No JavaScript function runs during
native paint. The website's **RecordedPaint** example is an interactive workflow
diagram with node movement, zoom, and native viewport feedback.

```tsx
import { RecordedPaint } from "@solid-gpui/core/components";

<RecordedPaint
  viewportHeight={180}
  recording={{
    width: 320,
    height: 180,
    commands: [
      { kind: "quad", bounds: { x: 20, y: 30, width: 100, height: 50 }, color: { kind: "accent" } },
      { kind: "text", origin: { x: 30, y: 45 }, text: "Input", fontSize: 14, color: { kind: "foreground" } },
      {
        kind: "path",
        points: [
          { x: 120, y: 55 },
          { x: 240, y: 90 },
        ],
        closed: false,
        stroke: { kind: "border" },
        strokeWidth: 3,
        fill: null,
      },
    ],
  }}
/>;
```

The native viewport fills its parent's width and uses `viewportHeight` (default
240 logical pixels). Wrap it in a styled `View` for borders, backgrounds, input,
or constrained width. `onViewport` reports actual logical width/height and the
window's device scale after layout, only when those values change. It is optional
layout feedback, never a paint callback.

| Contract    | Behavior                                                                                                                                                                                                                                       |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Coordinates | Logical drawing units; finite, absolute value at most 16,384. Recording dimensions and viewport height are 1..16,384.                                                                                                                          |
| Resize      | Default `fit: "contain"` centers and shrinks the recording to fit, without enlarging it. `fit: "none"` centers at 1:1 logical size.                                                                                                            |
| Transform   | Uniform positive `scale` (0.01..16) and `translateX`/`translateY`. Scale applies to geometry, strokes, and text. Translation uses drawing units multiplied by the fit scale. Rotation, skew, and nonuniform text transforms are unsupported.   |
| DPI         | GPUI applies device scaling once during scene construction; the recording stays in logical coordinates. No pixel readback or JS redraw is required on display changes.                                                                         |
| Clip        | Always clipped to the native viewport and ancestor content mask. Optional `clip` is a transformed axis-aligned rectangle in drawing coordinates.                                                                                               |
| Theme       | `foreground`, `background`, `accent`, and `border` resolve against the current native theme on every replay. `solid` uses validated `#RRGGBB`/`#RRGGBBAA`.                                                                                     |
| Paths       | Straight polylines with independent butt-ended stroke segments; a closed, strictly convex polygon can have a fill. Both winding directions work. Concave/self-intersecting fills and Bézier curves are rejected or absent from the contract.   |
| Text        | Native single-line shaping with inherited native font family, a top-left origin, and fontSize 1..256. Explicit transform scale × fontSize must remain at most 256. No wrapping, control characters, browser text measurement, or text editing. |

Each recording admits at most 1,024 commands, 4,096 aggregate path points,
32,768 generated triangle vertices, 2,048 UTF-8 bytes per text command, and
16,384 aggregate text bytes. Existing native-value and commit byte budgets also
apply. Path admission and preparation are linear in input points; native replay
is bounded by commands, prepared vertices, and text bytes. Retained geometry
depends only on recording content. Bounds, clipping, device scale, font style,
and theme are read during native replay, so resize/theme/DPI changes cannot reuse
stale scene geometry. Removing the component releases its recording.

## CPU frame ownership

`LiveFrame` owns its resource on the mounted native component instance. Its
generated ref is the capability: commands carry surface, epoch, node, module,
catalog, and build identity. No globally transferable image ID or native address is
exposed. A new mount or epoch creates a fresh owner; closing/removing/replacing
the component releases its visible and staging frames and drops GPUI image-atlas
resources when their final native scene owner is released.

```tsx
import { LiveFrame, type LiveFrameRef } from "@solid-gpui/core/components";

let frame: LiveFrameRef | undefined;
<LiveFrame ref={(value) => (frame = value)} viewportHeight={180} fit="contain" />;

// Run in an application event/effect. Await each call before producing another frame.
await frame?.replaceFrame({
  sequence: 1,
  width: 2,
  height: 1,
  rgba: [255, 0, 0, 255, 0, 255, 0, 255],
});
await frame?.clear();
```

Pixels are tightly packed, top-to-bottom, straight-alpha sRGB RGBA8, with exactly
`width × height × 4` validated bytes. The native adapter converts RGBA to the
linked GPUI BGRA representation once when presenting. Generated JSON contracts
use `number[]` with native `u8` validation; no image encoding or data URL occurs.
`replaceFrame` accepts up to 240 KiB; this fits the existing 1 MiB native JSON
limit even when every byte uses three decimal digits.

Larger frames use bounded chunks:

```ts
await frame.beginFrame({ sequence: 2, width: 1280, height: 720 });
for (let offset = 0; offset < rgba.length; offset += 240 * 1024) {
  await frame.writeFrameChunk({ sequence: 2, offset, rgba: rgba.slice(offset, offset + 240 * 1024) });
}
await frame.presentFrame(2);
```

| Command           | Behavior                                                                                                                                                            |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `replaceFrame`    | Validates all bytes and reserves capacity before replacing visible content. Rejects stale/duplicate sequence values. Cancels an existing staging upload on success. |
| `beginFrame`      | Reserves one complete staging frame. Rejects another active upload; cancel it explicitly first.                                                                     |
| `writeFrameChunk` | Requires the active sequence, contiguous offset, and 1..240 KiB within the reserved frame. Invalid chunks leave both staging and visible pixels unchanged.          |
| `presentFrame`    | Requires the complete active sequence; atomically replaces visible content. Partial uploads never paint.                                                            |
| `cancelFrame`     | Releases only the staging upload.                                                                                                                                   |
| `clear`           | Releases visible and staging pixels. Retains the last accepted sequence to reject delayed producer frames.                                                          |
| `dispose`         | Permanently closes this mounted owner, releases pixels, and rejects subsequent uploads. Idempotent. Remount to create another owner.                                |
| `getState`        | Returns disposed status, last accepted sequence, current dimensions, and reserved/retained CPU bytes for this owner.                                                |

Frame dimensions are 1..4,096, with at most 16 MiB per decoded frame. There is one
visible frame and one staging frame per component, and a 64 MiB host-wide budget
covering staging reservations and all live pixel owners, including scene references
awaiting release. Budget exhaustion rejects acquisition without dropping the
visible frame. Replacement may temporarily need capacity for both old and new
pixels. GPUI schedules final entity/atlas release at the end of its update cycle;
`clear`/`dispose` do not promise synchronous GPU-driver memory reclamation.

No native frame queue exists. Producers must await acknowledgement and keep
their own pending work bounded. The website **LiveFrame** consumer demonstrates
one upload at a time, pause/clear, error reporting, and timer cleanup on unmount.
`fit` supports contain, cover, or fill; clipping follows the viewport and ancestors.
CPU frames preserve their pixel colors across theme changes.

Beginning an upload reserves its sequence even if it is canceled. Every new
`beginFrame` or `replaceFrame` must exceed all previously admitted sequences on
that owner; `FrameState.sequence` describes the last presented frame. This prevents
delayed chunks from entering a restarted upload. Clear retains this high-water mark.

## Platforms and limits

The CPU resource and GPUI image path are shared by macOS, Linux, Windows, and the
WASM host. Local native rendering qualification is on macOS; the other native
platforms require their own pixel/window acceptance. Browser examples compile
against the same generated contracts. CPU upload/presentation is bounded but
performs copying and RGBA conversion; this is not a high-resolution video decoder
or a zero-copy claim. Applications can feed decoded frames from their own sources.

No IOSurface, shared GPU handle, camera capture, codec, audio, hardware decoding,
or platform acceleration import is exposed. Adding any such import requires an
explicit platform contract, ownership validation, and native acceptance. Encoding
pointer bytes does not create a portable media handle.

Generate and verify bindings after native changes:

```sh
bun scripts/native-codegen.ts
bun scripts/native-codegen.ts --check
bun --conditions=browser test examples/website/tests
```

Run native resource/scene checks with
`cargo test -p solid-gpui --features gpui-component,test-support --lib paint_media`.
On a desktop session, `cargo test -p solid-gpui --features gpui-component,test-support --test host_paint_media_platform`
checks actual native pixels for quads, paths, text, frames, and clear/dispose.
