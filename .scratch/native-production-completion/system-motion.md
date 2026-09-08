# Following system reduced motion

Date: 2026-09-07. Read-only source/API investigation. Existing [theme commands](../../crates/solid-gpui/src/components/theme.rs:57) expose explicit application-wide get/set bool. No production code/dependencies changed; none of these watchers was executed in this investigation.

## Decision

Implement an application-owned native preference watcher with target-specific modules and a small shared mode/state reducer. No GPUI fork is necessary: every current target has a query plus an actual notification API. The common effect is existing `App::set_reduce_motion(bool)`, which refreshes windows only when the effective value changes. Native renderer/widget animation paths already honor it.

Use installed versions as explicit target dependencies where needed; transitive availability does not permit importing them without declaring dependencies. Avoid shelling out to defaults/gsettings, polling, desktop detection chains or pretending an unavailable source means the OS selected false.

## Exact target seams

### macOS

Query `NSWorkspace::sharedWorkspace()` then `accessibilityDisplayShouldReduceMotion() -> bool`, provided by installed objc2-app-kit 0.2.2 with NSWorkspace/NSAccessibility features: [query](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-app-kit-0.2.2/src/generated/NSAccessibility.rs:127).

Subscribe to `NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification` ([notification constant](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-app-kit-0.2.2/src/generated/NSAccessibility.rs:147)) on **the workspace's notification center**, not the process default center. [NSWorkspace::notificationCenter](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-app-kit-0.2.2/src/generated/NSWorkspace.rs:44) returns a retained center.

The installed [NSNotificationCenter block API](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-foundation-0.2.2/src/generated/NSNotification.rs:145) is:

```rust
unsafe fn addObserverForName_object_queue_usingBlock(
    &self,
    name: Option<&NSNotificationName>,
    obj: Option<&AnyObject>,
    queue: Option<&NSOperationQueue>,
    block: &block2::Block<dyn Fn(NonNull<NSNotification>)>,
) -> Retained<NSObject>;
```

Use the main operation queue, keep the center/token/block in an owned macOS subscription, and re-read the property on notification. On drop, call [removeObserver](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-foundation-0.2.2/src/generated/NSNotification.rs:131) on that same center. The notification is for several accessibility display preferences, so coalesce unchanged motion values. GPUI already uses the workspace center for system-wake observation ([native precedent](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-macos-0.3.3/src/platform.rs:1323)).

Keep registration/query/removal on the main thread. The callback should publish a value/invalidation through a channel rather than borrow GPUI App synchronously; native notifications can be reentrant. Required target deps/features: the already-installed matching objc2, objc2-app-kit, objc2-foundation, block2 versions; NSWorkspace, NSAccessibility, NSNotification, NSOperation, NSString and block2 feature gates. Confirm exact feature unification in Cargo rather than importing a different objc2 generation.

### Windows

`UISettings::new()?`, `AnimationsEnabled() -> windows::core::Result<bool>` ([query](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/windows-0.62.2/src/Windows/UI/ViewManagement/mod.rs:1907)); effective reduced motion is the inverse. Subscribe with:

```rust
UISettings::AnimationsEnabledChanged(
    TypedEventHandler<UISettings, UISettingsAnimationsEnabledChangedEventArgs>
) -> windows::core::Result<i64>;
UISettings::RemoveAnimationsEnabledChanged(token: i64) -> windows::core::Result<()>;
```

Exact installed [registration/revocation](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/windows-0.62.2/src/Windows/UI/ViewManagement/mod.rs:2040). The event requires **Windows 10 version 2004 / build 19041** per [Microsoft's primary API documentation](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.uisettings.animationsenabledchanged?view=winrt-26100). Do not infer support on older Windows from successful compilation; registration casts to IUISettings6 and can fail. Windows 11 UTM meets this API floor.

Keep UISettings and event token in the application subscription. Event handlers enqueue the newly queried bool; assume callbacks may execute off the GPUI thread. Never capture `Rc<App>` in a WinRT callback. Revoke on teardown and guard a late queued event by watcher generation. Reuse the host's COM/WinRT initialization discipline; GPUI already constructs UISettings for scrollbar preferences ([existing use](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-windows-0.3.3/src/platform.rs:1423)). Declare windows 0.62.2 target features UI_ViewManagement and Foundation explicitly.

There is also `SPI_GETCLIENTAREAANIMATION` plus `WM_SETTINGCHANGE` in existing Win32 bindings and GPUI handles WM_SETTINGCHANGE. That is an alternative for an explicitly chosen older-Windows baseline, but adding a second hidden fallback is unnecessary for the current Windows 11 target. If required later, implement it as the declared platform backend rather than catching every WinRT error and guessing.

### Linux

The **installed** ashpd 0.13.13 already implements the standardized preference. Use `ashpd::desktop::settings::Settings::new().await?`, then [reduced_motion query](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ashpd-0.13.13/src/desktop/settings.rs:313):

```rust
async fn reduced_motion(&self) -> Result<ReducedMotion, Error>;
async fn receive_reduced_motion_changed(
    &self,
) -> Result<impl Stream<Item = ReducedMotion>, Error>;
```

[typed notification stream](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ashpd-0.13.13/src/desktop/settings.rs:348) subscribes to `org.freedesktop.appearance` / `reduced-motion`. `ReducedMotion::ReducedMotion` means true; `NoPreference` means false ([enum and decoding](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ashpd-0.13.13/src/desktop/settings.rs:159)). Unknown integer values decode to no preference, matching the [official Settings portal contract](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html). Missing keys or a missing portal are errors, not the no-preference value.

Prefer the lower-level `receive_setting_changed_with_args::<ReducedMotion>(APPEARANCE_NAMESPACE, REDUCED_MOTION_KEY)` when explicit stream decoding errors are needed: the convenience stream filters conversion errors out with `filter_map(t.ok())`. Subscribe **before** the initial read, and process the initial result plus subsequent notifications in one watcher task. The subscription must own the proxy and stream for its full lifetime.

ashpd's default feature is tokio, and the repository already has a native Tokio runtime. Run the DBus watcher on that actual runtime, then publish to the GPUI foreground; `App::background_spawn` alone does not establish Tokio context. Do not install a global/default Tokio runtime merely to make this one proxy work. A dedicated long-lived host service task on the existing executor is preferable, owned independently of a Surface. Portal stream termination marks the source unavailable and ends the watcher; explicit retry/re-enable can recreate it. No polling or endless retry loop is needed.

No GTK/GSettings/KDE heuristic chain is required: the standardized portal is the chosen backend. A desktop lacking the preference can continue using explicit application motion mode and report unavailable system mode honestly.

### Web/WASM

Use real browser `web_sys::window()` and:

```rust
Window::match_media("(prefers-reduced-motion: reduce)")
    -> Result<Option<MediaQueryList>, JsValue>;
MediaQueryList::matches() -> bool;
EventTarget::add_event_listener_with_callback("change", callback);
EventTarget::remove_event_listener_with_callback("change", same_callback);
```

Installed web-sys is 0.3.98: [query](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/web-sys-0.3.98/src/features/gen_Window.rs:2217), [matches](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/web-sys-0.3.98/src/features/gen_MediaQueryList.rs:33), [event registration](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/web-sys-0.3.98/src/features/gen_EventTarget.rs:32). Retain the MediaQueryList and `Closure<dyn FnMut(MediaQueryListEvent)>` in the guard; on teardown remove the listener and then drop the closure. Do not `forget()` a permanent closure. Use Window, MediaQueryList, MediaQueryListEvent, EventTarget feature gates and existing wasm-bindgen version.

This executes in the browser main-thread host, not QuickJS. A worker without Window returns unavailable; do not add matchMedia to the embedded runtime. Main-thread event handlers still enqueue changes instead of reentrantly mutating App. MatchMedia follows browser-exposed OS preference; it does not require DOM rendering.

## Minimal shared design

Suggested files: `crates/solid-gpui/src/host/motion.rs` plus target modules `motion/macos.rs`, `windows.rs`, `linux.rs`, `web.rs`; generated client contract remains beside `components/theme.rs`. The watcher should be initialized by shared host startup so default host and component host have the same effective policy. Platform watch ownership belongs to the application, not the first window or a native component instance.

Use one authoritative reducer:

```rust
enum MotionMode { System, Reduced, Full }
enum MotionSourceState { Starting, Available(bool), Unavailable(MotionSourceError) }
struct MotionState {
    mode: MotionMode,
    source: MotionSourceState,
    effective: bool,
    generation: u64,
}
```

Suggested generated interface: `getMotionPreference() -> MotionPreferenceState`, `setMotionPreference(mode) -> MotionPreferenceState`, where the returned state includes mode, effective bool and source availability. Refactor the newly added bool API into this contract if adopting it now; the project explicitly does not require a compatibility wrapper. Keep one App effective bool; do not make each window a separate preference owner.

**Startup policy must be explicit.** One defensible default is mode System, source Starting, effective true until initial source resolution, avoiding startup decorative animation before the user's preference is known. Available(false) then enables it. If source is unavailable, keep effective true and expose that state; this is an explicit conservative application policy, not an assertion about the OS. If that startup policy is undesirable, choose explicit Full until the application opts into System; do not silently imply OS following.

**Explicit mode switching:** Reduced/Full applies immediately with `cx.set_reduce_motion`; System requires an available source (or returns a precise `system-motion-unavailable` error and leaves the previous explicit mode intact). The watcher can remain registered while overridden, keeping the latest source value ready; updates must not override explicit mode. A source failure while already in System preserves the last resolved effective value, marks source unavailable and emits/logs one typed diagnostic. An explicit retry can recreate the subscription with a new generation.

Use a bounded latest-value handoff (`watch`/single-slot coalescing) because only the newest preference matters. Keep a GPUI foreground task like the existing [CommitPump consumer](../../crates/solid-gpui/src/host/commit_pump.rs:110), update the reducer there, and call `cx.set_reduce_motion` only after processing the current generation. Never pass App/Window handles through OS callback threads or DBus tasks. Dropping application ownership revokes the native listener/cancels the watcher and the foreground receiver. Zero-window residence and HMR must not duplicate subscriptions.

**Initial-read ordering:** establish the notification listener before querying. Notifications should invalidate/re-read or be sequenced with the initial result so a slow initial read cannot overwrite a newer update. Native synchronous query backends can install/query on one main-thread turn; Linux needs its query and queued events serialized in its task. A post-query invalidation causes another event-triggered read, not timer polling.

## Error and acceptance contract

- Distinguish unavailable API/portal/key, subscription failure, malformed values, and watcher disconnection. Return a small typed code plus bounded diagnostic text; do not panic the UI.
- Don't return a successful System mode if a read succeeded but listener registration failed: that's a snapshot, not following.
- Callback teardown failures on application shutdown should be logged once; dropped generation/receiver still prevents stale state writes.
- Key deterministic test: initial system value, user override, changing system value, returning to System, source loss, and late callback after teardown. Assert effective state, one application subscription and no stale mutation; no per-platform mirrored test boilerplate.
- macOS/Web acceptance can toggle the actual OS/browser preference while a native animation runs and show immediate settlement/resumption policy. Windows/Linux desktop toggles remain deferred to the user's cross-platform acceptance session. Compilation and reducer tests do not substitute for these observations.

No source changes are blocked on missing APIs. The work is targeted dependency declarations, RAII watcher ownership, and one foreground reducer, with honest unavailable behavior on hosts that cannot supply the chosen event source.
