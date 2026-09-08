# Native async composition and accessibility

## SwiftUI and AppKit view hosting

Native Modules currently render GPUI elements and retained GPUI views. They do
not expose embeddable AppKit views or SwiftUI hosting in either direction.
Multiple Surfaces currently use separate GPUI windows; they do not establish
support for multiple GPUI views inside one SwiftUI/AppKit window.

[SystemPopover](system-popover.md) now provides a separate owned native window
with shared Solid context. SwiftUI/AppKit embedding remains a separate proposed
stage. The [native presentation research](../.scratch/native-presentation/spec.md)
records the view-lifetime, input, accessibility, and layout changes it requires.

## Solid async control flow

Import native `Suspense`, `ErrorBoundary`, and `lazy` from
`@solid-gpui/core/runtime`. They use Solid's implementations with native child
types. Resources and transitions use the same Solid owner and reactive graph.

```tsx
import { Text } from "@solid-gpui/core";
import { createResource, ErrorBoundary, Suspense } from "@solid-gpui/core/runtime";
import { useNative } from "./native";

function Greeting() {
  const native = useNative();
  const [greeting] = createResource(() => native.greet({ name: "Ada" }));
  return <Text>{greeting()}</Text>;
}

export function Page() {
  return (
    <ErrorBoundary fallback={(_error, reset) => <Text onPress={reset}>Retry</Text>}>
      <Suspense fallback={<Text>Loading</Text>}><Greeting /></Suspense>
    </ErrorBoundary>
  );
}
```

Create resources inside the boundary's child component so it owns their errors
and cleanup. A transition retains resolved content while its resource is pending.
Detached Suspense content stays in the Solid host graph until attachment; an empty
native commit does not dispose it. Transport failure still rolls back a failed
commit. Root unmount rejects pending native requests and disposes owners.

`createResource` does not automatically cancel a superseded fetcher. Pass a
request-scoped signal when stale work should stop, and abort it on owner cleanup.
See [request cancellation](rust-bridge.md#request-cancellation-and-deadlines).

The production Bun and real QuickJS fixture in `fixtures/quickjs-async.tsx`
qualifies resource loading, nested Suspense, lazy content, transitions, error retry,
and disposal through binary native commits and replies.

## Accessibility semantics

Primitive native elements support button, text, textbox, checkbox, heading, link,
status, alert, group, list, listitem, and dialog roles. A generic role does not
create an AccessKit node. Choose a semantic role for an accessible container.

```tsx
<View accessibilityRole="status" accessibilityLive="polite"
      accessibilityValue={status()}>
  <Text>{status()}</Text>
</View>
```

`accessibilityLive` accepts `off`, `polite`, or `assertive`. Non-off live regions
require a semantic role and `accessibilityValue`: macOS announcements use the
value, not just the label. The renderer updates disabled and live states on the
same AccessKit node, preserving its identity, actions, and children.

`accessibilityDisabled` describes state to assistive technology. It does not
suppress application callbacks. Use the control's `disabled` property for actual
interaction policy; primitive TextInput also projects that state to AccessKit.
Native editor semantics remain owned by their native controls.

Protocol and native-node tests establish state projection. Screen-reader speech,
focus traversal, and target-platform accessibility acceptance still require an
actual assistive-technology session.

## Motion and text editing

The generated native client exposes `getMotionPreference()` and
`setMotionPreference("system" | "reduced" | "full")`. The returned state includes
`mode`, effective `reduced`, and a source tagged `starting`, `available`, or
`unavailable`. Gallery's Transitions page exposes all three modes.

Native hosts start in System mode with decorative motion reduced until the first
system value resolves. NSWorkspace notifications on macOS, UISettings events on
Windows 10 2004+, and the standardized desktop Settings Portal on Linux keep the
application preference current. Each subscription belongs to the application,
survives zero windows/HMR, and releases on shutdown. Explicit Reduced/Full modes
retain their effect while the latest system value continues to be tracked.

Choosing System without an available source returns an explicit error and keeps
the previous mode. Source failure while following System retains the last
effective value and reports `unavailable`; it never invents an OS preference.
A desktop without a supported portal can use either explicit mode. The common
GPUI flag refreshes every window and is consumed by native and renderer motion.
These native host subscriptions do not install a browser `matchMedia` adapter.

Primitive TextInput left/right, Shift-arrow, backspace, and forward delete step
across extended grapheme clusters, including combining accents and ZWJ emoji.
The external selection and IME contract remains UTF-16; explicit selection ranges
are not silently widened to graphemes. Existing undo and marked-text rules apply.
This does not establish full bidirectional visual caret/selection support or
change the separate gpui-component editor implementation.


## Application lifetime and activation

`mountApplication` owns the connection independently of a window. Its
`lastWindowClose` policy defaults to `"quit"`; `"keep-alive"` retains the Solid
application owner, transport and captured state while `application.root` is
`undefined`. `application.quit()` requests native process exit and disposes the
connection. `dispose()` releases the application and its connection.

```tsx
import { mountApplication, Text } from "@solid-gpui/core";
import { StdioTransport } from "@solid-gpui/core/stdio";

const application = mountApplication({
  transport: () => new StdioTransport(),
  lastWindowClose: "keep-alive",
  setup() {
    return {
      render: () => <Text>Document workspace</Text>,
      onMount(root) { /* Configure each newly opened window here. */ },
      onActivate({ reason, urls, root }) {
        // Dispatch validated application URLs to the current document model.
        console.error(`Activation: ${reason}; ${urls.length} URLs`);
      },
    };
  },
});
```

Activation reasons are `launch`, `reopen`, and `open-urls`. The host installs GPUI
OS callbacks before starting the application. It buffers up to 32 activations
until the renderer is ready, with at most 64 URLs of 4096 bytes per activation.
After the last window closes, activation allocates a fresh Surface ID and calls
`onMount` before `onActivate`. Retired IDs are never reused. Application messages
use an application epoch and acknowledged sequence independent of Surface
revisions. Unacknowledged activations replay after HMR; acknowledged ones do not.
Handlers acknowledge synchronous acceptance, so applications own any subsequent
asynchronous document loading and its error reporting. Handler failure terminates
the connection explicitly.

The linked GPUI macOS provider dispatches reopen and open-URL callbacks. Windows
and Linux OS delivery and cross-process single-instance forwarding are not
implemented by this adapter. Registering a product's URL/document associations
also belongs to its installer/bundle configuration. Do not treat an in-process
activation test as desktop registration or second-instance acceptance.

## App-owned progress services

Gallery's Native & Platform page uses the generated `WorkspaceScan` native view.
A real filesystem worker owns its cancellation token; the view owns its observer.
Changing `requestId` restarts a path, clearing `path` cancels, and unmount cancels
and drops the observer. Replies carry the request ID so old progress cannot
replace a newer request's UI.

The example permits two live workers, counts at most 100,000 entries, queues at
most 4096 directories, never follows symlinks, and coalesces progress into one
latest-value slot at 20 Hz. Filesystem calls run off the GPUI foreground thread.
Cancellation is cooperative between filesystem operations: a blocked OS read
keeps its admission permit until it actually exits. Errors appear in the progress
result. This is an application service example, not a framework filesystem API.

## Native grid

Primitive styles support `gridColumns`, `gridRows`, `gridColumnSpan`, and
`gridRowSpan`. Each accepts an integer from 1 through 64. Specifying tracks selects
GPUI's native grid; `gap` and alignment apply without switching it to flex.
Tracks are equal fractions with a zero minimum. Grid track declarations cannot
combine with `flexDirection`; an item may have a span and its own flex layout.
The Gallery layout page demonstrates reactive two/three-column grids.

Generated Input/Textarea/Editor navigation and deletion also use extended
Unicode graphemes. Their Rope implementation requests only the chunks needed by
segmentation, including cross-chunk context, without copying the full document.
This protects logical editing; complete visual bidi geometry remains separate.

## Images

`Image` accepts local filesystem paths, percent-encoded `file:` URLs, HTTP(S)
URLs, and `data:image/...` URLs. The native host installs its HTTP client before
opening windows; browser hosts use their platform fetch client. Inline data is
decoded through GPUI's asynchronous asset cache, including SVG and animated
formats supported by GPUI. Each source and fallback is bounded to 1 MiB of UTF-8
text, with the normal total-frame budget still enforced.

`fallbackSource` is used when the primary fails; it is not shown merely because a
request is pending. It keeps the image viewport, object fit, and corner radius.
Use an inline fallback for a packaged application that should work offline.
Relative filesystem paths are resolved against the host working directory.

Pass the URL directly; no JavaScript fetch or temporary local file is needed:

```tsx
import { Image } from "@solid-gpui/core";

<Image
  source="https://images.unsplash.com/photo-1470770841072-f978cf4d019e?w=640"
  fallbackSource="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='320' height='180'%3E%3Crect width='320' height='180' fill='%2394a3b8'/%3E%3C/svg%3E"
  objectFit="cover"
  style={{ width: 320, height: 180, borderRadius: 12 }}
/>
```

GPUI owns asynchronous download, decoding, and caching by source. Updating
`source` selects the new resource. Browser requests must satisfy the image
server's CORS policy. Custom Rust applications that create their own GPUI
`Application` must configure its HTTP client with `with_http_client`.

Native HTTP requests have a 10-second connection timeout and a 15-second idle
read timeout. These are idle/connection bounds, not a total image-download
duration limit.
