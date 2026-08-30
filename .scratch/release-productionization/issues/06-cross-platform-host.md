# Cross-platform host needs GPUI backend and target validation

Status: needs-triage
Type: task

The process-runtime boundary is substantially more portable than the native host,
but the current release candidate is explicitly a macOS ARM host. `ProcessAdapter`
can spawn an external renderer and exchange framed bytes without crossing into GPUI;
the executable that owns `ReactRoot` still unconditionally starts a GPUI application,
so a cross-platform host needs a real GPUI platform/backend build and target-specific
validation.

Evidence: `Cargo.toml:16-21`, `Cargo.lock:1941-2203`,
`crates/react-gpui/src/transport.rs:1-8,43-65,309-499`,
`crates/react-gpui-host/src/main.rs:1-13,109-157`,
`references/zed/crates/gpui_platform/src/gpui_platform.rs:56-97`,
and the platform manifests/sources cited below.

## Comments

### Current boundary

- The documented candidate is the process-runtime host for macOS ARM; it does not
  bundle Bun or a renderer command (`README.md:133-137`). The ordinary Rust/host
  release jobs run on `macos-15` (`.github/workflows/ci.yml:18-22,68-71`), and the
  embedded-Bun job is separately macOS ARM on `macos-26`
  (`.github/workflows/embedded-bun.yml:29-33`). There is no Linux or Windows host
  build, renderer smoke, or release artifact in the checked-in workflows.
- The embedded runtime is a separate, hard macOS boundary: its build script rejects
  every non-macOS target because it currently requires macOS JavaScriptCore
  (`crates/react-gpui-bun/build.rs:146-149`) and then invokes `xcrun`, the macOS SDK,
  and macOS-specific linker flags (`crates/react-gpui-bun/build.rs:239-267`). A first
  cross-platform host should therefore mean process mode with an externally supplied
  renderer; embedded Bun requires a separately designed per-target port.

### What `ProcessAdapter` can reach independently of GPUI

- `RuntimeAdapter` is a byte-stream seam with no GPUI methods, and the transport
  implementation uses only `std::process`, pipes, synchronization, and worker
  threads (`crates/react-gpui/src/transport.rs:1-8,43-53`). `ProcessAdapter::spawn`
  wires child stdin/stdout and inherited stderr (`crates/react-gpui/src/transport.rs:409-447`), while
  `recv_commit`, `send_event`, and `shutdown` only frame bytes and stop/wait for the
  child (`crates/react-gpui/src/transport.rs:470-499`). This path can run a platform-native command on
  Linux or Windows as well as macOS; it does not require Metal, Cocoa, a display, or
  a GPU merely to start the renderer process.
- Child termination is also written portably: `Child::try_wait`, `kill`, and `wait`
  are used for shutdown (`crates/react-gpui/src/transport.rs:309-365`), and only the optional exit-signal
  field is Unix-specific (`crates/react-gpui/src/transport.rs:391-405`). The fatal transport path explicitly
  says it never touches GPUI (`crates/react-gpui/src/transport.rs:55-65`).
- That is runtime reachability, not an independently buildable package today.
  `react-gpui` has an unconditional `gpui` dependency (`crates/react-gpui/Cargo.toml:8-13`),
  exports both `ReactRoot` and `ProcessAdapter` (`crates/react-gpui/src/lib.rs:1-24`),
  and the host uses `ReactRoot` plus `gpui_platform::application()` before opening a
  window (`crates/react-gpui-host/src/main.rs:109-157`). Extracting protocol/transport
  into a GPUI-free crate would be required if “ProcessAdapter independently” means a
  separately compilable transport artifact; it is not required for the process mode
  once the GPUI host itself has been built for a target.

### Locked GPUI revision and feature graph

- The workspace requests `gpui = 0.2.2` and patches crates.io to the Zed Git
  revision `6805d952f9f3d702f760aa11b1547df8a625fa16`
  (`Cargo.toml:14-21`). The host pins `gpui_platform` to that same revision
  (`crates/react-gpui-host/Cargo.toml:12-15`), and every GPUI platform package in
  the current lock is sourced from that revision (`Cargo.lock:1941-2203`). This is
  a source snapshot, not a promise that the current crates.io GPUI release or a
  newer Zed checkout has the same feature graph. Any GPUI upgrade must update both
  pins and the lock together, then re-check each target.
- The checked-in `gpui_platform` facade selects macOS, Windows, or Linux/FreeBSD at
  compile time (`references/zed/crates/gpui_platform/src/gpui_platform.rs:56-81`),
  but its desktop backend features are explicit: `wayland` and `x11` forward to
  `gpui_linux` (`references/zed/crates/gpui_platform/Cargo.toml:14-35`). The host's
  direct `gpui-platform` dependency currently requests no such features
  (`crates/react-gpui-host/Cargo.toml:12-15`).
- This is visible in the lock graph. The root `gpui_linux` package currently lists
  no `gpui_wgpu`, Wayland, X11, or `zed-xim` dependencies
  (`Cargo.lock:2034-2060`), whereas the checked-in Zed workspace lock lists those
  dependencies when the Linux backend feature graph is active
  (`references/zed/Cargo.lock:7604-7647`). A Linux target build must explicitly
  choose the backend features and prove whether the root lock can be reused or
  needs a target-feature lock refresh; `--locked` on the macOS-resolved graph is not
  evidence of a Linux renderer build.

### Linux reachability and real requirements

- The Linux implementation has real Wayland, X11, and headless paths. It chooses
  headless when requested, otherwise guesses from `WAYLAND_DISPLAY`/`DISPLAY`
  (`references/zed/crates/gpui_linux/src/linux.rs:29-59`,
  `references/zed/crates/gpui/src/platform.rs:93-123`). With the current no-feature
  host dependency, the Linux path can compile to the headless client but does not
  provide a desktop compositor backend; a visible host needs `gpui-platform`
  `wayland`, `x11`, or an intentionally selected single backend.
- Wayland is not just a Rust compile target: the client connects from the compositor
  environment and binds `wl_compositor`, `wl_shm`, `xdg_wm_base`, seats, outputs, and
  optional protocol globals (`references/zed/crates/gpui_linux/src/linux/wayland/client.rs:749-785`).
  X11 connects through XCB and requires the XKB, RandR, Render, and XInput
  extensions (`references/zed/crates/gpui_linux/src/linux/x11/client.rs:351-355`).
  The manifest exposes the corresponding protocol/input pieces, including
  `wayland-backend` with `client_system`/`dlopen`, `x11rb` with DRI3/RandR/XInput,
  `xkbcommon`, `x11-clipboard`, `xim`, and `filedescriptor`
  (`references/zed/crates/gpui_linux/Cargo.toml:17-45,82-129`). A Linux smoke job
  therefore needs an actual X11 or Wayland session and the deployed compositor,
  XKB/input, clipboard, and protocol packages—not only `cargo check`.
- Both Linux window implementations instantiate `gpui_wgpu::WgpuRenderer` from
  raw Wayland/XCB handles (`references/zed/crates/gpui_linux/src/linux/wayland/window.rs:544-578`
  and `references/zed/crates/gpui_linux/src/linux/x11/window.rs:746-767`). The pinned
  renderer creates a native wgpu instance with Vulkan and GL backends
  (`references/zed/crates/gpui_wgpu/src/wgpu_context.rs:289-297`), selects an adapter
  compatible with the real surface (`references/zed/crates/gpui_wgpu/src/wgpu_context.rs:103-112`), and configures/presents
  the surface (`references/zed/crates/gpui_wgpu/src/wgpu_renderer.rs:249-314,1265-1402`).
  The target consequently needs a working Vulkan loader/ICD or OpenGL stack and a
  driver that can present to the selected compositor; this is a rendering/backend
  requirement, not a ProcessAdapter requirement.
- Linux also pulls platform services beyond rendering: `ashpd` file chooser/settings/
  notification/trash features and `oo7` native-crypto keyring support are declared
  in `references/zed/crates/gpui_linux/Cargo.toml:53-87`, and the source listens to the system D-Bus
  `org.freedesktop.login1` sleep signal (`references/zed/crates/gpui_linux/src/linux/platform.rs:205-227`).
  `xdg-desktop-portal`/D-Bus availability, font discovery, clipboard, and system
  notification behavior belong in the target acceptance matrix. Linux text uses
  `CosmicTextSystem` and the optional pinned `zed-font-kit` path
  (`references/zed/crates/gpui_linux/src/linux/platform.rs:149-152`;
  `references/zed/crates/gpui_linux/Cargo.toml:53-63`;
  `references/zed/crates/gpui_wgpu/Cargo.toml:35-39`).

### Windows reachability and real requirements

- The pinned revision has a native Windows backend, selected by the facade
  (`references/zed/crates/gpui_platform/src/gpui_platform.rs:63-69`). It is not the Linux wgpu path:
  `gpui_windows` is `cfg(target_os = "windows")` and declares AccessKit Windows,
  DirectWrite-related support, `windows`, `windows-core`, `windows-numerics`, and
  `windows-registry` (`references/zed/crates/gpui_windows/src/gpui_windows.rs:1-40`;
  `references/zed/crates/gpui_windows/Cargo.toml:22-50`).
- `WindowsPlatform::new` initializes OLE, creates `DirectXDevices`, and constructs
  DirectWrite text before the message loop (`references/zed/crates/gpui_windows/src/platform.rs:109-131`).
  `DirectXDevices::new` creates a DXGI factory and enumerates a D3D11 adapter
  (`references/zed/crates/gpui_windows/src/directx_devices.rs:46-72,92-137`), then requires D3D feature
  level 10.1+ and StructuredBuffer support (`references/zed/crates/gpui_windows/src/directx_devices.rs:143-192`). The
  renderer creates DirectX resources/pipelines and optional DirectComposition
  (`references/zed/crates/gpui_windows/src/directx_renderer.rs:124-197`), draws every GPUI primitive, and
  presents a DXGI swap chain (`references/zed/crates/gpui_windows/src/directx_renderer.rs:330-392,1201-1255`). The checked-in
  Windows workspace feature list names the required Win32 families—Direct3D11,
  DXGI, DirectComposition, DirectWrite, HLSL, GDI, COM/OLE, Shell, HiDPI, input,
  and window messaging (`references/zed/Cargo.toml:901-961`).
- Release Windows builds compile the HLSL shader set with `fxc.exe`
  (`references/zed/crates/gpui_windows/build.rs:3-8,21-29`); the build script searches
  `GPUI_FXC_PATH`, `where.exe`, or the installed Windows 10 SDK and fails if no
  compiler is found (`references/zed/crates/gpui_windows/build.rs:71-138`). Debug builds instead use
  `D3DCompileFromFile` (`references/zed/crates/gpui_windows/src/directx_renderer.rs:1705-1746`). A Windows
  artifact gate must therefore include a Windows SDK/toolchain, D3D11-capable
  hardware/driver, DirectComposition (or the explicitly disabled fallback), native
  text/input, accessibility, and a visible-window smoke test.

### macOS-only paths that define the current candidate

- The macOS facade compiles `MacPlatform` only on macOS
  (`references/zed/crates/gpui_macos/src/gpui_macos.rs:1-35`), while the Apple crate
  gates Cocoa, Core Foundation/Video, Metal, and Objective-C dependencies to
  `target_os = "macos"` (`references/zed/crates/gpui_apple/Cargo.toml:19-40`; `references/zed/crates/gpui_macos/Cargo.toml:24-63`).
  The source links the Carbon keyboard framework and Security framework directly
  (`references/zed/crates/gpui_macos/src/platform.rs:1515-1563`).
- `gpui_apple` creates a `CAMetalLayer`, selects the system Metal device, loads the
  generated Metal shader library, and presents command buffers
  (`references/zed/crates/gpui_apple/src/metal_renderer.rs:151-224,327-340,447-488`).
  Its build script invokes `xcrun -sdk macosx metal` and `metallib` against the macOS
  SDK (`references/zed/crates/gpui_apple/build.rs:123-173`). These are correctly excluded from Linux and
  Windows by `cfg`; they are evidence of the current macOS ARM packaging boundary,
  not a cross-platform fallback.

### Assessment and gap list

| Target | ProcessAdapter | Native GPUI host | Remaining gap |
| --- | --- | --- | --- |
| macOS ARM | Reachable and covered in process mode | Current candidate path: Cocoa/AppKit + Metal, with an external Bun/renderer command | Keep this as the explicit boundary; no claim that the candidate is cross-platform or that it bundles Bun |
| Linux | The std process/pipe path is reachable | Source support exists for headless, Wayland, and X11, but the host currently enables no `gpui-platform` Linux backend features and the root lock has no active Linux renderer deps | Select Wayland/X11 features, refresh/verify the locked graph, install compositor/input/portal/font/GPU prerequisites, and run display-backed WGPU smoke tests on supported distros |
| Windows | The std process/pipe path is reachable; signal reporting simply returns `None` on non-Unix | Source support exists through Win32 + Direct3D11/DirectComposition | Build on Windows with SDK/FXC, validate D3D11/DirectWrite/input/accessibility and swap-chain presentation, then produce a target artifact and smoke evidence |
| Embedded Bun on Linux/Windows | Not applicable to ProcessAdapter | Blocked by the current build script's explicit macOS JavaScriptCore check | Treat as a separate port; do not imply process-mode support means embedded-Bun support |

The concrete productionization task is to decide and document a process-mode target
matrix, make the Linux backend feature/lock choice explicit, add Windows and Linux
build plus renderer smoke evidence, and keep the artifact/runtime prerequisites
visible. Until those gates exist, “cross-platform host” should not be inferred from
`ProcessAdapter` portability or from the existence of Linux/Windows source modules
in this one locked GPUI revision.

### Process-mode target matrix implementation

- The host now enables both `gpui_platform` Linux backend features,
  `wayland` and `x11`, while leaving the existing macOS dependency path
  unchanged. The pinned `gpui_linux` manifest forwards both features; its
  `wayland-backend` dependency enables `client_system` and `dlopen`. The
  former selects the system client API and the latter loads Wayland libraries
  dynamically, avoiding link-time Wayland library requirements.
  `font-kit` is kept intentionally for the pinned Zed font-kit path and
  consistent cross-platform glyph metrics rather than a fallback text system.
- The target-aware lock refresh adds only the Linux backend graph (including
  `gpui_wgpu`, Wayland/X11 protocol crates, `xkbcommon`, and `zed-xim`). It
  leaves existing package versions and non-Linux dependency edges unchanged.
- `.github/workflows/cross-platform.yml` adds locked Ubuntu format/check/
  Clippy/protocol-test coverage and a locked Windows workspace check plus
  process-host build. Embedded Bun is explicitly excluded on both runners
  because its build script requires macOS JavaScriptCore; the Windows build
  comment records the hosted Windows SDK/FXC requirement.
- The local Linux target probe installed `x86_64-unknown-linux-gnu`. The first
  `cargo check -p react-gpui-host --target x86_64-unknown-linux-gnu` stopped
  in `psm` because macOS has no `x86_64-linux-gnu-gcc`; retrying with
  `CC_x86_64_unknown_linux_gnu=clang` reached
  `yeslogic-fontconfig-sys`, which requires a configured Linux pkg-config
  sysroot. This is a macOS cross-check limitation, not runner validation.
- The analogous Windows probe installed `x86_64-pc-windows-msvc`, but
  `cargo check -p react-gpui-host --target x86_64-pc-windows-msvc --locked`
  stopped in `psm` (`lib.exe` missing) and `stacker` (`windows.h` missing).
  A macOS host cannot provide the MSVC/Windows SDK, so Windows remains
  runner-only validation.


The remaining work is display-backed Linux and Windows renderer smoke on real
CI runners (including compositor, GPU, portal, font, input, and accessibility
prerequisites) and a separately designed non-macOS embedded-Bun port.
