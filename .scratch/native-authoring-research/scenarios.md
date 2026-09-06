# Retained native components: implementation scenarios

Date: 2026-09-06. Scope: read-only source investigation; this file is the only authored artifact. The APIs below are proposed, not existing exports. Other workspace changes were left intact.

## Baseline and decisive evidence

Actual Cargo dependency: `gpui-pre 0.3.3`; `gpui-component` revision `928c3eb776a3d733d9b771f7dea27a6a79242ced` (`Cargo.toml:32-34`, `Cargo.lock:2277`). The native sources cited below are from that Cargo checkout, not an unpinned upstream branch.

Absolute dependency prefix:

`/Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/`

| Evidence | Source below that prefix |
| --- | --- |
| `Input::new` takes `&Entity<InputState>` | `crates/component/src/input/input.rs:167` |
| `InputState::new` takes `Window` and `Context<Self>` | `crates/base/src/input/base/state.rs:4978` |
| `set_value` resets selection, LSP, scroll and clears undo; it suppresses ordinary change emission | `crates/base/src/input/base/state.rs:834-855` |
| `replace_all` preserves undo history, but still resets selection and scroll | `crates/base/src/input/base/state.rs:865` |
| Input events are Change, PressEnter, Focus, Blur | `crates/base/src/input/base/state.rs:113` |
| Public trait implementation exposes marked range without accessing private fields | `crates/base/src/input/base/state.rs:2692,2717` |
| Committed input clears marked range and emits Change | `crates/base/src/input/base/state.rs:2829-2873` |
| Preedit updates marked range and notifies; it does not emit Change | `crates/base/src/input/base/state.rs:2880-2981` |
| Composition can end through `unmark_text` without Change | `crates/base/src/input/base/state.rs:2726` |

GPUI source prefix:

`/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/`

`src/app/context.rs:355` exposes `subscribe_in`: typed subscriptions receive the subscriber, emitter, event, `Window`, and `Context`. The subscription internally uses weak entities. Retain the returned Subscription in the component; do not detach it into an unrelated lifetime.

Current repository boundaries:

- `crates/solid-gpui/src/renderer/extensions.rs:148`: adapters only validate/render; context at line 226 has no Window/App and no retained instance.
- `crates/solid-gpui/src/renderer/extensions.rs:341`: children are a fresh element iterator created from the committed tree.
- `crates/solid-gpui/src/renderer.rs:495-613`: Snapshot/Patch use a candidate store, validate all extensions, then publish and reconcile native maps.
- `crates/solid-gpui-host/src/lib.rs:556-574`: foreground commits currently call `root.update`, without entering the surface Window.
- `crates/solid-gpui/src/renderer.rs:1121`: render currently processes queued commands before constructing the element tree.
- `crates/solid-gpui/src/renderer/input.rs:654`: existing built-in input accepts controlled writes only when `edit_seq <= ack_edit_seq` and no marked text exists. This is a useful semantic baseline, not automatically inherited by gpui-component Input.

## Minimum retained runtime

Keep typed contract validation pure. Give the renderer a per-surface instance table whose records contain the exact component identity, a retained native Entity through an erased adapter, a revocable event binding, and a native incarnation token. The public authoring macro should generate this erasure; application authors should write a normal GPUI struct and Render implementation.

Proposed internal seam:

```rust
// Names and signatures are illustrative, not existing APIs.
trait ComponentDescriptor {
    fn prepare(&self, wire: &ExtensionProperties, children: &ChildContract)
        -> Result<PreparedProps, ExtensionError>;
    fn mount(&self, props: PreparedProps, events: NativeEvents,
        window: &mut Window, cx: &mut App) -> Box<dyn NativeInstance>;
}

trait NativeInstance {
    fn update(&mut self, props: PreparedProps, window: &mut Window, cx: &mut App);
    fn element(&self) -> AnyElement;
    fn invoke(&mut self, command: PreparedCommand,
        window: &mut Window, cx: &mut App) -> Result<Vec<u8>, String>;
}
```

`PreparedProps` is an erased, already decoded concrete props value, not a second serialized format. Descriptor/instance pairing guarantees the downcast. Optional lifecycle methods and empty event/command sets are generated; simple render functions compile into the same runtime mechanism without requiring handwritten empty methods.

Concrete integration order:

1. Split the current `apply_decoded_message` commit path into pure candidate-tree/typed-property preparation and publication. Reuse existing tree validation; do not mutate an InputState while another node can still fail validation.
2. Enter the actual surface Window in `NativeStateRegistry::apply_to_surface` before applying a prepared commit. The surface already owns its window handle. Perform root/entity updates within that foreground Window callback. Do not invent a background entity mutation path.
3. Publish the validated store; revoke instances removed, replaced, or from another epoch; mount new instances and apply typed props to surviving ones synchronously on the foreground. Constructors/updates in this phase are infallible under their validated contract. Async/file/network initialization belongs in explicit loading/error component state. Arbitrary Rust panic or I/O is not transactionally rolled back.
4. Activate/rebind instance event routes after the published revision is known. Events raised by native reconciliation must be stamped consistently with that committed state; buffer until activation if necessary.
5. Render extensions by returning their retained entity element, under the existing extension style/measurement boundary. Render constructs visual elements, not a new input model, subscription or task.
6. A props update and a move preserve the entity. Component contract replacement, deletion, new epoch, or surface close revokes routes and drops owned subscriptions/tasks/entities. Incarnation tokens protect delayed async completion even if a node number reappears.

There is no need to add an extension-specific HashMap for Input, another for Editor and another for each provider. The single native instance table owns those providers' ordinary GPUI entities. Native list state and editor document state remain inside their own entities.

## Input adapter that can actually be implemented

The retained wrapper owns `Entity<InputState>`, a Subscription, its latest prepared props, a native edit sequence, and an optional pending controlled replacement. Mount creates InputState once with `cx.new(|cx| InputState::new(window, cx).default_value(...))`; render uses `Input::new(&self.input)`.

For Change, subscribe once using `subscribe_in`; read the native current value, increment edit sequence, emit typed `{ value, editSeq }`. Do not reconstruct the native model in response to this event. Listeners are rebound by the instance event route, not by permanently capturing the first listener ID.

Two input contracts must not be conflated:

- Native-owned input: `defaultValue` initializes once; Change reports values; explicit `replaceAll`/`reset` command performs intentional external edits. This can be implemented safely with the upstream public API today.
- Controlled input: generated bindings carry an edit acknowledgement alongside `value`. Hide transport metadata from ordinary application props if desired, but the generator/runtime must preserve it. Supporting only `{ value: string }` cannot reliably distinguish stale echoes from intentional replacement during ongoing typing.

Controlled update algorithm:

1. If requested value equals native value, do nothing to InputState. This is essential even when acknowledgement advances: `set_value` is not an idempotent setter for editing state.
2. If acknowledgement is older than current native edit sequence, do not apply the replacement. A newer edit must survive a delayed callback response.
3. Query composition using `EntityInputHandler::marked_text_range` inside the input entity update with Window/Context. Private `ime_marked_range` access is unnecessary.
4. When marked text exists, retain the latest eligible replacement for explicit re-evaluation; do not call `set_value` or `replace_all` during composition.
5. Before applying a deferred replacement, re-check acknowledgement against the current edit sequence and compare values again. A later committed edit may have invalidated the request.
6. For an eligible differing external value outside composition, choose documented semantics: reset uses `set_value`; undoable replacement uses `replace_all`. Neither automatically preserves cursor or scroll. If the public contract promises selection preservation, implement it explicitly with the upstream selection API and test it; do not claim `replace_all` already provides that guarantee.

Important challenge: upstream preedit, cancellation, and `unmark_text` do not all issue Change. Retrying deferred replacement only in the Change subscription is incomplete. Observe native entity notifications or reconcile the controlled value in the wrapper's native render/update path, with composition checks, and verify cancellation. `unmark_text` itself does not notify; prove the actual OS/GPUI path triggers the wrapper's re-evaluation or provide an explicit observation/interception seam. A first implementation can deliberately expose native-owned Input plus explicit replacement commands while this controlled contract is completed; it must not advertise complete controlled IME support without that test.

Also, InputEvent::Change contains no value snapshot. GPUI event delivery may be deferred, so reading current state in a subscription can observe later state when multiple mutations occur in a single update. Verify the expected event granularity with the real upstream API; do not claim lossless per-edit deltas from a notification-only event. Full editor delta/version transport needs its own document-model integration.

## Instance commands: the existing wire is sufficient

Evidence:

- `crates/solid-gpui/src/protocol.rs:403`: CommandMeta already carries surface, epoch, after_revision, request_id, and node_id.
- `crates/solid-gpui/src/protocol.rs:499`: InvokeNative carries module_id, module_digest, function_id, args bytes.
- `packages/solid-gpui/src/protocol/protocol.bop:113`: Command already includes nodeId; line 409 has InvokeNativeCommand; CommandValue includes Bytes.
- `crates/solid-gpui/src/renderer/commands.rs:655-682`: InvokeNative currently classified as a root operation; non-root node_id is rejected.
- `packages/solid-gpui/src/renderer/root-container.ts:349`: invokeNative currently always uses submitSurfaceCommandValue, whose nodeId is 1.
- `crates/solid-gpui/src/renderer/native_calls.rs:10`: root module implementation is resolved and moved to background execution. It must not receive GPUI entities.

Implement node-target InvokeNative in the existing envelope:

1. JS generated ref binds the originating root and mounted host node. Its invoke method validates liveness, flushes relevant pending host mutations through the same ordering mechanism as existing node commands, and submits InvokeNative with that nodeId.
2. In `process_commands`, classify InvokeNative as root-only only when node_id is 1. Dispatch non-root InvokeNative to the retained instance in the foreground with Window and Context.
3. Verify that target node exists and is Extension; module ID/digest identify that component's declared method contract, not an arbitrary global native module. Decode/validate function ID and argument bytes before invoking the component. A provider can export per-component method descriptors under the same generated registry; entry identity comes from the addressed node.
4. Use the existing bytes CommandValue and request ID acknowledgement. Keep root module calls on the existing background path. Node calls have typed foreground semantics; expensive work explicitly spawns background work and later updates through weak entity/incarnation checks.
5. Bound argument/result/error sizes and pending calls. A rejected command must not silently succeed or fall through to a root module.
6. JS ref disposal/HMR rejects pending node calls and prevents new submissions. Async native completion must capture the native incarnation and original epoch; a later same-epoch ordinary props revision does not by itself invalidate an already admitted call.

No new Bebop message or command-kind is required. This is still a semantic API change: update runtime admission, generated clients, contract metadata/digests, and tests together.

Identity caveat: the canonical JS producer allocates node IDs monotonically (`packages/solid-gpui/src/renderer/nodes.ts:478,532,574`). Native instance tokens remain necessary for delayed native work and type replacement. If future callers can rebind a disposed JS ref to an arbitrarily reused node ID, node_id plus current after_revision alone is insufficient; bind a creation-generation token in generated args or enforce the producer's no-reuse rule. Do not solve this by exposing Entity IDs to JS.

Ordering caveat: `renderer/commands.rs:647` currently requires after_revision equal the store revision when commands are drained during render. Several commits arriving before one draw can make an earlier queued command stale. Retain and test the existing explicit rejection semantics, or move admission/execution to the ordered foreground commit handoff where its stated after_revision is current. Do not loosen equality blindly and accidentally target a changed instance. Processing methods inline after the corresponding commit is the cleaner long-term route because UI method ordering should not depend on whether the OS drew a frame between two messages.

## Children, editor and list constraints

Initial retained Input has no children; this isolates lifecycle without pretending that every gpui-component widget is covered.

For child-bearing native components, the framework must own stable child/slot handles that render fresh elements from the published host tree. Do not persist `AnyElement`, borrow an iterator across renders, or invoke JS render closures synchronously from native layout. Named slots require explicit contract/tree representation; ordered children already exist. Child dirtiness must notify the retained parent/slot view even when its props have not changed. Avoid recursively borrowing SolidRoot during its own Render: use a separately owned committed document handle or an equivalent slot-view boundary.

Editor owns rope, selection, IME, undo, LSP tasks and caches in Rust. The JS-facing document/event/command contract transmits bounded values/deltas, not those resources. Large list owns ListState and scrolling in Rust; use the existing visible-range JS materialization path or a Rust-owned data provider. GPUI render_item cannot synchronously call across the byte/process boundary to JS.

## Minimal tests that can disprove the design

1. **Retained identity and cleanup:** mount a real Input; patch value-independent props, replace listener, move node, update sibling, redraw twice; assert the same InputState entity and one subscription. Delete/new epoch drops the instance and owned task/subscription exactly once. No event from a captured retired emitter is delivered into the replacement.
2. **Editing semantics:** native edit creates undo history/selection; same-value JS acknowledgement leaves both unchanged. Deliver edit 2 before acknowledgement of edit 1; the older value must not overwrite edit 2. Confirm an explicit reset or replacement has its documented undo behavior.
3. **IME deferred replacement:** start marked text using EntityInputHandler, deliver a differing acknowledged value, then separately test commit and cancellation/unmark without Change. The candidate is not applied during marked text; on completion it is either applied or discarded by a freshly checked sequence. This test challenges the notification gap above rather than merely checking a boolean helper.
4. **Atomic validation:** prepare one valid retained component update plus another invalid component/slot in a single batch. Assert unchanged published revision, unchanged native value/entity, zero extra mount/update/event effects. Do not claim rollback of arbitrary asynchronous effects.
5. **Node method roundtrip:** generate a typed Input ref; focus the actual native input and obtain a bytes result. Wrong node/contract/function/args, disposed ref, epoch replacement and delayed completion after delete are rejected; an ordinary props update after admitted async work does not reject its legitimate result. Include command followed by another patch before draw to pin ordered execution/rejection semantics.

Follow these with one real-window focus/IME check. Deterministic TestAppContext tests prove state transitions; they do not prove OS IME behavior or smoothness. For slots/list delivery add a concrete child-only update test and visible-content/list work-boundary tests, rather than a large suite that mirrors generated code.
