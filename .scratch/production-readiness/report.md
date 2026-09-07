# Production readiness audit — 2026-09-07

Baseline: `9e54d9203b3a1cc5bea6be04d81700f8cb6506c6`.
Local environment: macOS 26.6.2, ARM64, Bun 1.4.2, Rust 1.98.1.
The linked native renderer is `gpui-pre 0.3.3` from Cargo.lock.

## Changes

| Area | Finding and resulting behavior |
| --- | --- |
| English presentation | Maintained documentation already used English, but the screenshot depicted the obsolete React implementation. README now shows the real Solid Gallery, has direct development/distribution links, and records screenshot refresh steps. Unicode and IME fixtures remain multilingual intentionally. |
| Host Tree structure | Every insertion rewrote all sibling indexes. NodeGraph now owns insertion/removal, updates only the shifted suffix, and provides constant-time sibling lookup. Unattached subtrees retain exclusive ownership of their children during reparenting. Rollback restores sibling order in place without a large argument spread. |
| Module depth | Removed host-config orchestration of attachment, deletion, movement, and reindexing, plus unused HostTree/RootOwner forwarding methods. The NodeGraph transaction journal now hides these structural invariants behind insertion/removal. |
| Keyboard shortcuts | Per-surface bindings were compiled together into GPUI's application-wide keymap. The registry retains compiled bindings and selects those belonging to the active window, including when no control has focus. Parse failure is atomic; closed surfaces and retired epochs release bindings. |
| Native popup menus | Delayed OS selections could outlive the enabled item snapshot. Routing now requires a live native entity, matching property generation, enabled menu/item/ancestors, and a subscribed live event route. Removed the extra Rc around events. |
| Distribution | A separate Gallery distribution binary embeds its QuickJS bundle and uses the shared native application lifecycle. Native packaging produces macOS app ZIPs, Linux portable archives with desktop entries, and Windows portable ZIPs. Extraction, checksums, the actual routed Host Tree with both native module identities, and runtime shutdown are checked before publishing a candidate artifact. |
| Production bundle | Application builds can explicitly omit source maps. Gallery packaging selects this mode, reducing its embedded source from about 7.0 MiB to 1.4 MiB while preserving development source maps by default. |
| Release presentation | Actual packaged UI inspection exposed an FPS overlay covering search and singular build-count wording. Release builds now disable the development monitor by default; explicit environment overrides remain available, and one-build labels use singular English. |
| Startup commands | A real release trace exposed `Snapshot → SetTitle → Patch` being consumed before the next paint, with the deferred title incorrectly rejected as stale. Commands now execute on their window in message order with their native observers prepared. Strict revision admission remains; later commits cannot overtake an admitted command. |
| First frame | With the monitor disabled, the native packaged window could remain blank until resized. The first accepted Snapshot now explicitly refreshes its window, covering bootstrap before GPUI has tracked the root entity's window dependency. Subsequent commits retain ordinary entity invalidation. |
| Window lifetime | An observation queued for a future frame retained the root after closure. Its callback now uses a weak entity, so dropping a surface releases the root and cancels pending native futures without waiting for another frame. The existing native lifecycle regression caught this during full-workspace validation. |

See [distribution](../../docs/distribution.md) and
[keyboard/menu contracts](../../docs/keyboard-and-menus.md) for maintained usage
and platform-specific behavior.

## Reviewed contracts retained

These existing designs already address the relevant ownership and work bounds;
their complexity serves concrete requirements.

- `protocol/frame.ts`: borrow complete input frames; buffer only an incomplete
  frame and validate its length before increasing retained allocation.
- `host/commit_pump.rs`: the foreground pump yields after 16 messages or 1 MiB,
  preserving message order and atomic application of a legal large payload.
- `native/executor.rs`: module composition shares 128 admission permits;
  cancellable async tasks release admission, and running blocking work retains
  its permit until completion. The foreground never waits for worker shutdown.
- `renderer/paint/virtual_list.rs`: native ListState belongs to the Host Node;
  viewport bounds drive the Solid range, overscan is added once, and unchanged
  edge scrolling propagates to outer panes.
- `tree.rs`: candidate Snapshots publish after validation and Patch journals
  restore the previous tree on failure. Generated protocol/catalog identities
  remain the authoritative cross-language contract.

## Measurement contract

`renderer-structure-bench.ts` measures allocation, sibling insertion, Snapshot
construction/encoding, and a correctness check for wide View trees. It uses one
warmup and five recorded samples at each size. It does not open a native window,
perform layout, or measure GPU/display cadence. The counterexample is a smaller
1,000-node tree alongside 5,000 and 10,000 nodes; every run checks child count and
indexes. Ordinary tests assert bounded work rather than elapsed time.

The key bounded-work regression builds 512 siblings: the baseline writes 131,328
indexes; the candidate permits at most 512. Existing reparent/removal and event
tests remain, with one regression for detached reparenting and atomic rollback.

Final measurements ran serially after release compilation finished, with native
UI automation paused. Both versions used the same installed dependencies and
benchmark source. Baseline renderer files came from the commit above in an
isolated source copy; candidate files came from the working tree.

| View children | Baseline median (ms) | Candidate median (ms) |
| ---: | ---: | ---: |
| 1,000 | 4.282 | 0.956 |
| 5,000 | 62.473 | 4.413 |
| 10,000 | 259.453 | 7.112 |

Raw samples: [baseline](renderer-baseline.jsonl),
[candidate](renderer-candidate.jsonl). The measured
[renderer diff](renderer-change.patch) has SHA-256
`d8d1dcc19f30913797a31fd4846b6fd4b876da3a699d8fec51585c835b153c08`.
Reproduce the candidate with:

```sh
bun --conditions=browser .scratch/production-readiness/renderer-structure-bench.ts
```

For a baseline comparison, run the same benchmark against the baseline renderer
source with the same dependencies. Stop builds and other benchmark processes
before recording samples. Keep the child-count/index assertions enabled.

## Validation

- `bun run task package-ci`: passed, including 81 existing/key regression tests,
  all TypeScript project checks, formatting, regenerated native catalog identity,
  and packed package consumer smoke tests. See [log](package-ci.log).
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --locked --features solid-gpui/quickjs`: 288 passed,
  zero failed, one existing ignored case. Includes the embedded-source QuickJS
  press/Patch roundtrip after deleting authored source. See [log](rust-tests.log).
- `cargo clippy --workspace --all-targets --locked --features solid-gpui/quickjs -- -D warnings`:
  passed. See [log](clippy.log). Cargo reports the existing upstream `block 0.1.6`
  future-incompatibility notice separately.
- `bun audit`: no vulnerabilities found across 147 packages.
- All 18 host command integration tests and the native popup regression passed.
  The startup command test first failed with the actual stale title result;
  see [red log](command-order-red.log) and [green log](command-order-green.log).
- macOS Gallery input and three Build interactions were observed in the real
  native window used for the English README screenshot.
- Earlier extracted standalone macOS candidates completed native editing, Build,
  and Rust workspace-analysis round trips. The lifecycle-corrected final
  candidate received the startup qualification below; its interaction smoke
  was not repeated after the last rebuild.

## Native startup regression evidence

Two fresh launches of the extracted release app remained blank for more than
10 seconds with the performance monitor disabled; resizing recovered the
actual Gallery tree. The process sample showed idle event/commit loops rather
than a busy loop. The same binary with the monitor enabled displayed its UI.
The monitor-disabled A/B trial that later received external resize events is
not used as evidence of recovery without a resize.

The lifecycle-corrected final archive then passed three untouched cold starts.
Each initial accessibility tree showed the full Overview/workspace controls at
800×633 with the English window title and no performance overlay; the saved
records are `package-final-trial1.ax.txt`, `package-final-trial2.ax.txt`, and
`package-final-trial3.ax.txt` (the matching screenshots are beside them).
Later screenshots in trial 2 were changed by an external desktop interaction
and are explicitly excluded from the startup verdict.

The real startup trace contained a loading Snapshot, a title command, and the
actual Gallery Patch before the next paint. The title reply was unsuccessful
because rendering deferred the command beyond its admitted revision. The
host integration regression reproduces this exact batch and also checks
freshly committed focus state and rejection of a newly submitted stale command.
Current-frame focus traversal remains owned by GPUI; the renderer does not
invent a separate tab-stop algorithm.

GPUI's public test context draws a new window synchronously and does not expose
the native frame-request boundary. The black-window regression therefore uses
actual packaged cold launches and accessibility/screenshot inspection; there is
no unit test that claims to simulate this race.

## Release qualification

The available native desktop is macOS ARM64. Windows/Linux packaging is wired to
native CI runners; each target still needs an observed successful job and desktop
qualification. Developer ID/notarization, Windows publisher signing, clean-machine
runtime dependencies, keyboard layouts/IME, accessibility, and OS menu placement
remain release-specific checks. Credentials and public releases are not created
by this audit. The platform guide distinguishes portable packages from installers.
