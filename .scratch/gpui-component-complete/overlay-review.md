# Root overlay ownership review

2026-09-06. Read-only review of the native vendor implementation. No JS adapter review, implementation edits, repository test runs, or native runtime execution. HEAD: `1bf74901244d8af95d7d547de5a0fb5ce362b2fc`; vendor files are untracked working-tree content. Only this report was written by the reviewer. Findings were delivered to the parent while implementation continued; the fixes described below were subsequently reread. Source hashes at the end pin the reviewed state.

## Current conclusion

All three confirmed defects found in this review now have corresponding code fixes: render identity was tied to Dialog position instead of lifetime, Sheet insertion under surviving Dialogs discarded pending focus restoration, and a token could close against another Window. The old Sheet prepaint size write is also guarded, and Dialog snapshots are filtered before assigning surviving positions and the visible dimming overlay. Static review supports these changes. The four native acceptance cases below remain required evidence; no runtime pass is claimed.

## 1. Lifetime identity and surviving native state

**Original issue, P1:** Closing lower Dialog A moved surviving B from stack index 1 to 0. Host, focus-trap, and popup ids included the index, so B could reuse A's child state or remount as its old path disappeared. Replacing a Dialog at the same index without an empty frame also reused the old path. Sibling Dialogs shared the outer `fade-in` identity. Replacing Sheet reused `sheet-host`, `sheet`, and `slide`. Correct token addressing did not prevent these transfers.

This follows from actual GPUI behavior: [explicit ids build the GlobalElementId stack](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/element.rs:300); [element state is retrieved by GlobalElementId and state type](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/window.rs:3976); the animation element retains its start time through the same state system. [TextSelectionScopeMarker forwards the child id](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/base/src/text_selection.rs:1722), so selection scope was not an identity boundary.

**Current code:** [OverlayElement](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root/overlay.rs:122) adds an `("overlay", lifetime id)` boundary and forwards the child's own layout, prepaint, and paint. Root uses it for the entire Dialog, including the outer animation, while Sheet's containing Div receives the lifetime id ([Dialog wrapper](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root.rs:292), [Sheet wrapper](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root.rs:226)). The Dialog popup, base host, and focus trap now use constant ids within that lifetime boundary ([popup](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/dialog/dialog.rs:571), [base host](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/base/src/dialog.rs:496), [focus trap](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/base/src/dialog.rs:504)). Therefore moving B in the stack does not change those state paths; index remains available for z-order, topmost state, and visual offset.

Both builder invocations also run in the lifetime namespace, covering eager `window.use_keyed_state` calls before the returned elements render ([Sheet builder](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root.rs:215), [Dialog builder](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root.rs:259)). This places the boundary in the overlay layer instead of adding wrappers to individual controls. Runtime verification must still cover both eager builder state and actual rendered child state.

## 2. Pending focus when opening Sheet under Dialogs

**Original issue, P2:** Focus an input in A, open B, Confirm B, then insert or replace Sheet before the animation delay expires. Confirm requested deferred close, Root removed B, and restoration to A was pending. The old Sheet path called `take_previous_focus` unconditionally, cancelling the task and taking pending A. Since A remained active, Sheet intentionally skipped taking focus itself. Neither branch then restored A; focus could remain in removed B. B's surface `on_close` or its deferred owner callback can trigger this sequence.

**Current code:** [Sheet consumes pending focus only if no Dialog survives](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root/overlay.rs:272). With A alive, the pending task remains intact while the first Dialog's eventual predecessor is changed to the new Sheet. [The restore helper](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root/overlay.rs:184) replaces older tasks, and opening a new Dialog still deliberately consumes pending focus so its old timer cannot steal focus from the newly opened top layer. These branches are statically coherent; delayed native focus and Tab assertions remain necessary.

## 3. Token and Window affinity

**Original issue, P2:** `OverlayToken::close` updated the token's Root using any caller-supplied Window. Token A could remove A's layer while restoring a handle, clearing selection, and notifying the owner in Window B. [GPUI focus installs the supplied handle id without checking its originating window](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/window.rs:2209).

**Current code:** [Token close](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root/overlay.rs:152) checks the Root's stored `window_id` against the provided Window and returns false before mutation on mismatch. `refresh` and `set_sheet_placement` take no Window and update only the token's Root, so this issue did not apply to them. A two-window test should assert both windows remain unchanged on rejection, then confirm the token still closes normally with its own Window.

## 4. Retired render snapshots

The original Sheet prepaint unconditionally wrote a captured size into `root.sheet_size`. If its builder or later same-frame rendering replaced/closed that lifetime, the obsolete callback could overwrite the new Sheet's size or repopulate a cleared value. Notifications consume this size to offset themselves. This requires render-time mutation; ordinary event callbacks are not assumed to interleave inside a frame.

The current Sheet checks the cloned lifetime after the builder and publishes its prepaint size only while the id still matches the current Sheet ([post-builder lifetime check](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root.rs:216), [prepaint ownership check](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root.rs:230)). Dialog snapshots are retained by live lifetime before surviving indexes and dimming placement are computed ([survivor filtering](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root.rs:273)). This avoids leaving the only visible overlay on a retired snapshot or leaving B at A-and-B's former top index after A disappears. An adversarial native builder case should verify these boundaries.

## Behaviors supported by the current source

| Scenario | Source reasoning |
| --- | --- |
| Close A while newer B survives | [Close lookup](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root/overlay.rs:231) removes A by id and reconnects the next Dialog's previous-focus link. It does not pop B. |
| Close the same token or a clone again | Its id no longer resolves. [Lifetime retirement](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root/overlay.rs:110) also flips the shared open flag only once. |
| Owner callback opens a new modal | `on_closed` is deferred until Root and owner entity updates are released. The callback may reenter Root; no old stack pop runs afterward. |
| Dialog decision callback opens B while closing A | Base invokes the decision first; the component close request captures A's lifetime id, preserving new B ([base close ordering](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/base/src/dialog.rs:507), [owned request](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/dialog/dialog.rs:562)). |
| Sheet surface callback opens a replacement | Base requests close before invoking `on_close`, and the component request captures the old lifetime id ([base ordering](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/base/src/sheet.rs:16), [owned Sheet request](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/sheet.rs:255)). |
| Replace Sheet beneath Dialogs | Old lifetime receives `Replaced`; the new Sheet keeps the old pre-Sheet return target, becomes first Dialog's predecessor, and does not take focus from surviving Dialogs. |
| Close Sheet beneath Dialogs | First Dialog inherits the removed Sheet's predecessor, so later Dialog closure does not restore into that Sheet. |
| Close all current Dialogs, callbacks reopen | [The synchronous close loop](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root/overlay.rs:262) removes the present stack; deferred callbacks may create a new stack afterward. This is coherent callback behavior. |
| Root/window is destroyed | [Token openness](/Users/jgbingzi/workspace/sp/solid-gpui/vendor/gpui-kit/crates/component/src/root/overlay.rs:151) also requires its weak Root to upgrade. No window-teardown `on_closed` guarantee is documented or assumed here. |

Surface `DialogButtonProps.on_close` / `Sheet.on_close` and the owner callback passed to `open_*_owned` are separate signals. Surface hooks are UI dismissal callbacks; the owner callback describes Root lifetime retirement, including programmatic closure and Sheet replacement. They are not duplicate deliveries of one event. A consumer needing all retirement reasons should observe the owner callback. A Dialog decision may veto UI dismissal while caller code independently opens or closes another token; id routing protects the other layer.

## Four meaningful native acceptance cases

1. **Owner and callback reentrancy:** A then B; close A without touching B; repeat via a clone; exercise both a decision callback and deferred owner callback opening a new Dialog. Replace Sheet A with B, then close retired token A. Assert survivor ids, callback reasons/counts, and the focus chain as survivors subsequently close. Include AlertDialog's shared Root path.
2. **Pending focus with Sheet:** A plus B, with a specific A input focused before B. Confirm B, insert/replace Sheet from a close callback, and advance beyond `ANIMATION_DURATION`. Assert focus returns to A's actual input and Tab remains in A; closing A focuses the new Sheet, then closing Sheet restores the original base input. Also open a new Dialog during the pending delay and prove the cancelled timer cannot steal its focus.
3. **Native lifetime state:** A and B contain actual children with the same local keyed-state id and different values. Remove A and redraw; B retains its own state and identity. Close/open an owner without an empty intervening frame; the new owner receives fresh state. Repeat for Sheet replacement. Cover eager builder state and rendered child state, plus a fresh per-owner animation start. Token-only tests cannot establish these properties.
4. **Window and retired-frame containment:** A's token called with Window B returns false and leaves both windows' focus, selection, tokens, and owner counters unchanged. Retire/replace an old Sheet before its prepaint and prove its callback cannot publish the old size. A builder retiring another Dialog must leave the survivor's index, topmost status, and dimming placement correct.

Existing base Sheet tests cover request ordering and focus-trap registration; base Dialog tests cover trigger state and backdrop geometry. They do not exercise this Root lifetime graph or surviving native keyed state. This report claims no test pass and no native presentation acceptance.

## Source snapshot

| File | Lines | SHA-256 |
| --- | ---: | --- |
| `vendor/gpui-kit/crates/component/src/root/overlay.rs` | 307 | `c62ea06b09e02c37c95939de19c671ab7ea02c9abec9cdb80b24364da3aa297f` |
| `vendor/gpui-kit/crates/component/src/root.rs` | 520 | `6679e952fe1f9ee4713b3c0dafa0b5748ea46df24bb16a67fbaada5539378cf5` |
| `vendor/gpui-kit/crates/component/src/dialog/dialog.rs` | 689 | `452f90c50fc4e508bdcc5213ab015ab285daf40d6a70a9f19fbe9bd96356d9a2` |
| `vendor/gpui-kit/crates/component/src/sheet.rs` | 266 | `1bab5a415f938ad35320e8bfa73df6a3bfba85c66443b30fd171dafc44abbf43` |
| `vendor/gpui-kit/crates/base/src/dialog.rs` | 675 | `acb25d42610167aab2f0b76897cfaa1b874ec73031769b1ebc8c71ad892ebe7a` |
| `vendor/gpui-kit/crates/base/src/sheet.rs` | 268 | `c9e820416d280f76ca4fca2e2194bf005b25e99f3fa5c0d6ddcb28aa84030eaa` |

Later edits require rechecking the corresponding conclusion.
