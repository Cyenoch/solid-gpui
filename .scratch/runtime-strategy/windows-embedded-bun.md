# Embedded Bun on Windows — portable single-executable implementation research

Date: 2026-09-16. Status: source investigation and implementation proposal only. No Windows application or embedded runtime was built or executed.

## Delivery requirement and decision

The user requires one portable Windows executable that runs without installation. That requirement supersedes the earlier DLL-and-sidecar recommendation.

**Recommended deliverable:** one `application.exe` containing the GPUI host, Bun/JSC runtime, bundled application JavaScript, and required application resources. No adjacent `bun_embed.dll`, `bun.exe`, Node executable, JavaScript tree, or `node_modules` directory is required to launch it. No installer, administrator rights or prerequisite VC++ redistributable installation should be needed on the declared Windows baseline. Windows system libraries and the supported graphics driver remain normal platform dependencies.

Use true static embedding as the design target. A self-extracting EXE that writes a Bun DLL or application JS to a temporary directory is not the default interpretation of this requirement and is not the recommended implementation. Normal application data/cache writes are a separate policy; the single-file requirement is not a read-only-application requirement.

The static architecture should preserve our existing persistent Bun owner thread and byte-transport contract. The difficult new work is (1) joining Bun and the Rust host into a coherent final native link, (2) evaluating the application from EXE-owned bytes, and (3) proving there are no non-system runtime files left outside the EXE.

## Evidence and pinned inputs

| Input | Inspected value | Source |
| --- | --- | --- |
| Bun revision | `34cbb9a40b4bd1bd767d134a7065e66c2432a676` | [Local build script][H4] |
| Bun compiler | `nightly-2026-07-20`; local query reports `rustc 1.99.0-nightly`, LLVM `22.1.8` | [Local build script][H4]; `rustup run nightly-2026-07-20 rustc --version --verbose` |
| Host compiler today | `1.98.1` | [Host toolchain][S1] |
| Bun native LLVM tooling | Pinned source accepts LLVM 21.1.x; cross-language LTO can need rustc's newer bundled linker | [Upstream tools][U1] |
| WebKit/JSC revision | `0f966e81b78c84bb23213e391bc679c4ef83e56b` | [Upstream WebKit dependency recipe][U2] |
| Windows cross-build inputs | SDK `10.0.26100`, CRT `14.44.17.14`, xwin `0.9.0`, UCRT servicing overlay `10.0.26100.8249` | [Upstream sysroot recipe][U3] |

### Corrections to the earlier assessment

1. **macOS embedding does not depend on Apple's system JavaScriptCore.** Bun's graph links its own static WTF/JavaScriptCore/bmalloc archives. macOS uses system ICU; Windows artifacts include static ICU. The error text in our macOS-only gate does not establish a JSC platform dependency. [U2][H4]
2. **Windows JSC already exists at our exact pin.** HTTP HEAD requests to the pinned `oven-sh/WebKit` release returned 200 for all three assets below. Availability does not prove that our embedding patch, static integration, or whole application works on Windows.
3. **The dedicated owner thread is reusable.** Upstream first-call JSC initialization and per-thread Windows libuv loops support retaining that architecture; no evidence requires replacing it with a CFRunLoop-equivalent design. See the runtime section.
4. **A portable EXE is not the same as Bun's `build --compile` output.** The latter is a standalone Bun application executable, not a library or an automatic linker for our Rust/GPUI host. [B1]

| Pinned WebKit asset | HEAD status | Content length (bytes) |
| --- | --- | --- |
| `bun-webkit-windows-amd64-debug.tar.gz` | 200 | 451341893 |
| `bun-webkit-windows-amd64-lto.tar.gz` | 200 | 637137613 |
| `bun-webkit-windows-arm64-debug.tar.gz` | 200 | 476431386 |

Release tag: `autobuild-0f966e81b78c84bb23213e391bc679c4ef83e56b`. No large asset was downloaded. The x64 graph targets a Nehalem CPU floor, not an AVX-only build; ARM64 has its own profile and currently lacks the x64 LTO WebKit variant. Qualify x64 first, then ARM64 separately. The inspected upstream release CI cross-compiles Windows from Linux, but its build driver also has a native Windows VS-shell path. [U2][U4][U5]

## Why static linking is the correct packaging direction

| Design | Meets the chosen single-EXE contract? | Relevant cost |
| --- | --- | --- |
| Statically linked Bun/JSC plus in-memory JS/assets | Yes, after dependency closure is proven | Shared Rust/native link, static CRT, memory-loaded application |
| EXE plus `bun_embed.dll` and JS resources | No | Multiple required runtime files |
| EXE that unpacks a DLL/runtime/JS before starting | Not the chosen design | Temporary native files, loader/security/cleanup complexity |
| External Bun process | Not the chosen embedded design | Requires a runtime executable or extraction |
| QuickJS package | A different runtime, not a substitute | Does not preserve Bun services |

### The `/GA` discovery matters differently for a static EXE

Bun's Windows flags add `/GA`, explicitly assuming TLS variables live in the executable. LLVM 21.1.8 maps it to `-ftls-model=local-exec`; Microsoft warns that it can generate incorrect code in a DLL. Consequently, the earlier idea of simply relinking `bun-debug.exe` objects as a DLL was not sound. A DLL would need its own DLL-safe object/PCH build. [U4][U6][M5]

For static linking, Bun's code and TLS are in the main application image, so this particular executable-versus-DLL mismatch is avoided. That supports the user's single-EXE choice, but does not prove Rust runtime/allocator compatibility or overall link success. Reuse upstream executable compile policies only where their assumptions still hold; the application owns its main entrypoint and manifest, not Bun's CLI.

## Static native link: build one application, not two Rust runtimes

The current implementation builds `bun_bin` as a complete Rust `staticlib` under Bun's separately pinned nightly, then links that plus Bun's C/C++ graph into a dylib. The normal GPUI application is another Rust executable built with the repository's stable compiler. Replacing `dylib=bun_embed` with `static=bun_embed` is therefore not a complete design. [H4][S1]

The Rust Reference states that a `staticlib` includes upstream Rust dependencies, including std; multiple Rust staticlibs are likely to conflict. It recommends a single combined Rust link/`staticlib` when multiple Rust subsystems are involved. `rlib` retains compiler metadata and allows rustc to reconcile the dependency graph. This is a source-backed design constraint, not a reproduced linker error in this repository. [R2]

**Preferred direction:** compile Bun's embedded Rust entry layer and the GPUI application as Rust libraries in one coordinated dependency graph using one exact compiler, one panic policy, and one selected global allocator. Let the final executable's Rust link (or one combined Rust staticlib passed to the native linker) include the C/C++/JSC/ICU archives. Retain the C-shaped byte interface as a useful seam without pretending it isolates two Rust runtimes in a single image.

Do not use `/FORCE:MULTIPLE`, delete arbitrary std objects from an archive, or depend on accidental link order to hide duplicate runtime definitions. A direct two-artifact static link can be a diagnostic experiment, but is not the production architecture until its symbol/runtime ownership is demonstrated.

The following sections incorporate the exact pinned-source link and source-loading investigation.

### A dedicated static-image Cargo lane

Use a deliberate packaging root containing the application and Bun's Rust crates in one dependency graph, rather than changing the entire repository's default toolchain as a side effect. A generated or maintained workspace-excluded application-image crate is one concrete implementation: depend on the application entry library and a patched `bun_bin` reusable `rlib` form, enable `solid-gpui-embed`, and produce the final executable with Bun's exact nightly. The whole participating graph, including GPUI, must compile with that compiler; this has not been demonstrated. [T1][T2]

Bun's `bun_bin` currently declares its global mimalloc allocator. Keep exactly one allocator selection, either owned by that integration crate or deliberately retained in the reused Bun crate; do not add a second `#[global_allocator]` to the application. Preserve Bun's native dependency allocator setup (`use_mimalloc_in_dependencies`) and required C exports. The existing patch excludes CLI `main` for embedding; retain that exclusion and the application's own startup. Allocator, panic and sanitizer ownership must be explicit, not an accidental result of linker archive extraction. [T2][L5]

Prepare the pinned checkout, overlays, required submodules/path dependencies (including `vendor/lolhtml`), generated configuration, and Windows shim inputs before the packaging Cargo invocation. A `build.rs` cannot introduce a path dependency after Cargo has already resolved its manifests. Nested-workspace root patches, profiles and `.cargo/config.toml` settings do not automatically become the new application's policy. Carry the required dependency patches, feature unification, lockfile, `--cfg`, `-Z`, codegen paths and environment deliberately. [T1][T3][T4]

Extend the pinned Bun native graph with an embedding product that emits explicit C/C++ objects, archives and link requirements **without** linking a separate `bun_rust.lib` into the final application or requiring the normal Bun CLI build first. Feed those native inputs into the one final rustc link. Keep code generation in upstream dependency order; the build product must describe whole-archive needs, exports, system libraries, delay-load flags and linker/profile requirements, not merely a filename list. [T3][T5]

CLI response-file scraping is not an adequate build API: Windows output is `bun-debug.exe` with `bun-debug.exe.rsp`, and the response file contains inputs, not every flag from the link command. Preserve the host's GUI entrypoint/resources rather than copying CLI-only flags/resources indiscriminately. Cache by source/patch, target, compiler, native toolchain, codegen and profile inputs. [T5][T6]

Finally, map the release profile through the entire graph. Today's local build hardcodes `debug-no-asan`, `bun-debug` and Rust `dev`; outer `cargo --release` does not turn embedded Bun into a release artifact. Begin with a non-sanitized diagnostic build to settle link/runtime behavior, then qualify a coherent release product and its actual dependencies. LLVM/LTO interoperability is a separate check; do not mix bitcode producers and linkers just because both are called LLVM. [H4][U1]

## Reuse pinned WebKit release archives on all three platforms

**User requirement incorporated:** Linux, macOS and Windows builds should consume the precompiled static libraries published at [oven-sh/WebKit Releases](https://github.com/oven-sh/WebKit/releases), reducing build time without replacing Bun's patched JSC with an unrelated engine build.

This is already a first-class mode in the pinned Bun dependency recipe, not a new downloader to invent. With `cfg.webkit == "prebuilt"`, `scripts/build/deps/webkit.ts` selects the archive, declares its headers/libraries and returns `build: { kind: "none" }`. Reuse that path for the new embedding product. Do not clone or compile WebKit by default. [U2]

| Target family | Prebuilt native inputs described by the pinned recipe | Selection distinction |
| --- | --- | --- |
| macOS | WTF, JavaScriptCore, bmalloc and matching headers; system ICU | `macos`, `amd64`/`arm64`, build/sanitizer suffix |
| Linux | WTF, JavaScriptCore, bmalloc, static ICU and headers | `linux`, architecture, libc ABI (including separate `-musl`), build/sanitizer suffix |
| Windows | WTF, JavaScriptCore, bmalloc, static ICU (`sicudt`, `sicuin`, `sicuuc`, debug variants) and headers | `windows`, `amd64`/`arm64`, build/sanitizer suffix |

The upstream URL form is `releases/download/autobuild-<WebKit revision>/bun-webkit-<os>-<arch><suffix>.tar.gz`. It is a platform-specific artifact matrix, **not** one archive usable on every OS, nor proof that every theoretical flag combination exists. Use Bun's pinned WebKit revision, not `latest`; do not silently downgrade an unavailable variant. Windows assets at the exact pin were checked as listed above; this pass did not independently enumerate every Linux/macOS release asset. [U2]

Requirements for the integration:

1. Reuse upstream URL selection, matching generated/public headers and extraction logic. Key any shared CI cache by the complete artifact identity (revision, target OS, architecture, libc ABI, debug/release/LTO and ASAN), not just by host or revision. Record artifact provenance/digest for reproducible inputs.
2. Match ABI-affecting flags exactly. The upstream recipe explicitly warns that ASAN changes `WTF::Vector` layout: mixing ASAN and non-ASAN variants can corrupt memory. Debug and LTO variants are also distinct inputs, not interchangeable speed choices. [U2]
3. Validate native compiler/linker compatibility, CRT and LTO requirements against actual archive contents during implementation. Prebuilt availability removes JSC compilation; it does not prove the final static GPUI link or platform support.
4. If the chosen release variant is unavailable or WebKit itself must be changed, fail with the missing artifact/configuration and use an explicitly selected source-build path when necessary. Do not silently spend a full build compiling a different engine configuration.
5. Keep layers distinct: build-owned prebuilt archives/headers are not end-user sidecars. The application statically consumes their code; Windows still ships only the final EXE. Bun Rust/C++, embedding patches, other required native dependencies and the GPUI app remain build work.

No compile-time speedup was benchmarked, and no large release archive was downloaded during this investigation.


## Runtime lifecycle: reusable architecture and genuine Windows risks

### Preserve the persistent owner thread

The pinned Bun `JSCInitialize` uses `std::call_once` and calls `WTF::initializeMainThread()` inside that one-time body. The first caller therefore owns Bun's JSC/WTF initialization identity. Windows libuv `Loop::get()` lazily initializes a per-thread loop; Bun's Windows uSockets wrapper uses that loop. Bun also creates Worker VMs on native worker threads. These are source-level reasons to retain our persistent runtime-owner thread, not to move Bun onto the GPUI thread or invent a Windows-only thread architecture. The underlying pinned WebKit Windows RunLoop implementation was not independently executed or fully traced in this investigation. [L1][L2][L3]

The existing startup order is already appropriate: `EmbeddedVm::new()` initializes the VM, upstream `ensure_waker()` initializes the owner thread's loop, and only then `run()` publishes `Target { VmHandle, JsPoster }` to other threads. Before target publication, wake notifications only set pending flags. The actual Windows wake path goes through uSockets `us_wakeup_loop` and libuv `uv_async_send`; no CFRunLoop dependency is necessary for that transport. [L2][L3][L4]

Several Windows accommodations are already in the local patch:

- `bun_embedded_initialize_process()` performs Windows WTF-8 environment conversion inside its process-wide `Once`.
- `EventLoop::wakeup()` uses the raw `us_wakeup_loop` API instead of materializing an off-thread mutable Rust reference to `WindowsLoop`.
- `Teardown::Embedded` joins Worker-style paths for Windows libuv request draining, VM-owned open-handle shutdown, uSockets loop release, and `close_thread_loop()`.
- The `process.exit` patch requests VM termination rather than exiting the host; this must still be proven on Windows.

The audit did not establish a Windows runtime compile blocker in these overlays. That is weaker than a successful Windows compile/run, and does not remove the known build-system blockers. [L5]

### The most important runtime gate is restart after teardown

Windows `Loop::close_thread_loop()` calls `uv_loop_close`. On `UV_EBUSY` it walks/closes remaining handles, drives up to 64 nonblocking loop turns, and has a debug assertion that closure succeeded. It then clears its TLS slot. A later `Loop::get()` initializes the thread's loop storage again. Those are explicit upstream behaviors, not a measured failure in this project. [L2]

**Risk [INFERENCE]:** Worker-style shutdown is normally followed by thread exit, while our owner thread survives to host another session. Remaining callbacks, outstanding requests, or cached pointers to a freed uSockets loop must not survive that boundary. `WindowsWaker` contains a `BackRef<WindowsLoop>` and an upstream lifetime comment assuming that loop is never freed; its actual owners and destruction order need qualification under the embedded session model. The comment is not proof of a reachable use-after-free. [L6]

**Decision:** retain the shutdown invariant and prove it under live filesystem/network/Worker/child-process activity. If close fails, fix the outstanding ownership/request cleanup and do not admit another VM onto that storage. Do not suppress the assertion, merely log and continue, or disable assertions to make the test pass. Any explicit error-return path must preserve valid storage and make failed retirement non-reusable until it is actually safe. This is a likely investigation area, not a license to claim a reproduced defect.

The existing Context-identifier and process-main-VM lifetime patches should stay in the same shared lifecycle: each new session must see the previous main context fully retired. No second Windows-specific session manager is needed. [L5]

### Separate CLI initialization from host-owned process state

The current embedded initializer invokes `bun_crash_handler::init()`. Its Windows path installs a vectored exception handler and calls `SetUnhandledExceptionFilter`; the latter is a process-wide integration point. `JSCInitialize` also sets a C++ terminate handler. Bun installs a Rust panic hook as well. In the proposed unified Rust graph this is shared process policy, so the application must deliberately own or coordinate it rather than inherit CLI setup accidentally. [L1][L7]

**Proposal:** separate indispensable VM/JIT support from CLI crash reporting and console/process policy. Let the embedding host own the latter, or explicitly coordinate it. Do not blindly remove every Windows exception handler, and do not add a public configuration ABI unless the actual integration needs one. Inspect crash-reporting behavior before shipping a desktop application; inheriting CLI reporting is not automatically the application's intended policy.

A native crash, allocator OOM, or panic-abort in an in-process runtime can terminate the whole application. Intercepting JavaScript `process.exit` is not memory-safety isolation or native-crash containment. Fixing handler integration does not change that fundamental property.

Upstream Windows stdio initialization already represents unavailable handles as invalid and probes console capability before applying TTY behavior; a GUI-subsystem launch is therefore a qualification case, not a demonstrated unconditional startup crash. However, initialization changes console code pages/modes when a console exists. Because process setup currently runs once, restoring those settings after every VM session without reinitialization would be inconsistent. Prefer a host-owned console policy established once, or symmetric process-lifetime setup/restore, and test both Explorer-style GUI launch and inherited-console launch. [L8]

Windows environment conversion obtains `GetEnvironmentStringsW`, produces Bun's WTF-8 environment representation, and replaces Bun's `bun_core::os` environment pointers. The source does not establish that it replaces the OS-wide environment block. Preserve its before-JSC/before-Worker ordering and qualify non-ASCII values and host/Bun environment behavior; avoid a global-environment-clobber claim without tracing the actual storage. [L9]

Do not add the CLI parent-death watchdog to the embedding initializer. The existing embedded path omits its installation; its VM-level Windows hook is a no-op. An inert CLI environment flag alone is not justification to reject a valid embedded startup. [L6]

### Evidence boundaries

No Windows lifecycle scenario was run. Candidate issues above are deliberately separated from facts in source. Neither stale-loop reachability nor GUI stdio failure is presented as a reproduced bug. In particular, no claim is made that a normal imported application's `import.meta.url` becomes the bootstrap's synthetic `/[eval]` path; wrapper identity and imported-module identity are distinct.

### Runtime source references

[L1]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/bindings/ZigGlobalObject.cpp#L285-L330
[L2]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/libuv_sys/libuv.rs#L403-L515
[L3]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/uws_sys/Loop.rs
[L4]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/event_loop.rs#L952-L1060
[L5]: ../../crates/solid-gpui-bun-sys/bun_embed.patch
[L6]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/io/lib.rs
[L7]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/crash_handler/lib.rs#L1666-L1734
[L8]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/bun_core/output.rs#L520-L760
[L9]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/sys/windows/env.rs

## Embed the application as a module graph, not concatenated eval text

### Reuse Bun's existing in-memory module/resource graph

Our current `wrapper_source()` installs the JavaScript transport bridge and then performs `await import(<disk entry>)`. The wrapper occupies `module_loader.eval_source`; the application remains a separate imported module. The `eval_source` hook only serves `[eval]`/`[stdin]`-shaped module names and forces the TSX loader. Concatenating arbitrary application ESM into that wrapper would change its identity and module semantics, and would not implement an asset filesystem. [H3][G2]

The pinned Bun already has `StandaloneModuleGraph`: an in-memory file graph with Windows virtual paths under `B:/~BUN/`. Its module fetch path returns graph-backed source with the graph key as `source_url`; the resolver supports relative graph imports. Graph-aware paths also exist for `Bun.file`, `node:fs` operations and `Bun.embeddedFiles`. Other filesystem paths still address the real filesystem; this is not a sandbox or automatic remapping of all relative paths. [G1][G2][G4][G5]

**Recommended packaged entry:** serialize the Vite-produced application modules and required assets into that graph, embed it in the application image, and adopt it before VM initialization. Load the real application by its graph entry key. Use the same persistent graph for successive VM sessions; the graph singleton is process-owned and cannot be replaced with a different bundle each session. Keep ordinary filesystem entries for development in the existing runtime design, rather than adding an unrelated loader. [G1][G3][G6]

Preserve the proven transport startup ordering. The smallest candidate change is to keep today's *separate* eval bootstrap and change its dynamic import target to the graph entry. That makes the bridge available before application evaluation without concatenating user code, rewriting its identity or moving the entire bridge to native JSC object construction. This exact combination still needs a live VM check; it is a proposal, not an executed path. With the current bootstrap, `vm.main` remains the eval wrapper: real application module identity is preserved, but application `import.meta.main` semantics must be specified/tested separately rather than assumed. If a dedicated graph bootstrap module is used instead, it must initialize the bridge before dynamically importing the application; static import declarations alone do not establish that order. [H3]

### Build-time serialization and PE transport

Bun's standalone graph serialization is internal, version-pinned machinery. Prefer exposing a graph-only packaging target in the same pinned build tooling. An initial experiment can use a matching `bun build --compile` output as a **serialization intermediate**, extract its graph payload, and discard that intermediate executable. This is not using Bun's executable as our GPUI host. Do not use an arbitrary installed Bun version to generate a payload consumed by the pinned runtime. Inspect any target-runtime download behavior too: a matching frontend command alone does not establish that all intermediate inputs match. [B1][G1]

On Windows, the native graph reader scans the **main executable** for a `.bun` PE section containing a little-endian 64-bit payload length followed by the serialized graph. Static linking matches its `GetModuleHandleA(NULL)` assumption. A native section can avoid an unconditional full-payload startup copy. Prove section alignment, retention under release dead stripping, access permissions, declared length versus section bounds, and graph layout before adopting this transport. The trailer is at the end of the declared payload, not in padding after it. [U9][G1]

An alternative is EXE-owned bytes plus a narrow upstream entrypoint factoring `from_executable`'s payload parser. The pinned parser only creates mutable payload subslices for nonempty bytecode/module-info buffers; upstream emits its PE section with read-only characteristics. A source-only graph therefore has a source-backed zero-copy route; no unconditional writable copy is needed. Heap-owned graph caches remain separately mutable. [G1][G7] Both transports are single-file and no-extraction. Windows-native section integration is the first candidate; other-platform section mechanisms are not part of this Windows port. [G1]

Initially omit precompiled JSC bytecode. Its in-place mutable buffers add a separate version/lifecycle constraint. Even without bytecode, graph `File` objects cache source strings/Blobs across sessions, so graph reuse after teardown and Worker access require explicit qualification. Source evidence for graph loading is not proof that every cache is safe under our repeated-main-VM lifecycle. [G1][G3]

### Packaging closure must be explicit

- Include entry modules, code-split chunks, required worker entrypoints and application assets. A `new Worker` or dynamically computed import is not proof the bundler discovers the target: declared build inputs and final graph contents must agree.
- Preserve native Bun/Node built-ins as runtime imports. Do not reuse the QuickJS build rule that bans those APIs. Preserve the Vite Solid transform; the graph packaging phase consumes its output rather than replacing it with a second JSX pipeline. [S2]
- A graph-backed `Bun.file(<graph path>)` or `node:fs.readFile(<graph path>)` has source support; arbitrary asset URL forms, `?url`, `new URL`, and GPUI-native resource consumers need end-to-end checks. Native GPUI code does not automatically resolve Bun's virtual paths; provide bytes to its existing resource API or resolve packaged resources on the host deliberately.
- Reject unresolved application dependencies at packaging time when detectable. Computed imports require explicit entries or a runtime error naming the missing module; do not promise perfect static discovery. Keep errors visible rather than searching a developer checkout.
- Upstream embedded `.node` and FFI-library loading writes graph bytes into temporary files before loading them. Under the chosen no-runtime-extraction design, these files cannot silently pass package validation. Static integration of a required native dependency is a separate implementation task, not a format flag. [G2]
- Bind the bundle identity to the pinned serializer/runtime format. Validate lengths/offsets and format compatibility; a trailer check alone is not comprehensive parser safety or version validation. Avoid exposing an untrusted arbitrary payload loader when only build-owned bytes are needed.

## No-install deployment: CRT, assets, exports and signing

### Make the Windows CRT policy explicit

The pinned Bun C/C++ graph uses `/MTd` in debug and `/MT` in release. The Windows Rust target defaults are not equivalent to a guarantee of static CRT: the local `--print cfg` query has no `crt-static` feature by default, and adding `-C target-feature=+crt-static` reports that feature. The Rust Reference documents this switch and specifically recommends inspecting the resulting binary rather than inferring final linkage from flags. [U4][R2]

For the no-install release product, explicitly align **target** Rust and native dependency builds with static CRT where required, preserving a consistent release runtime. Do not indiscriminately impose target flags on host proc-macro/build-tool crates. Ensure the build-driver environment cleanup does not discard the product's required flags: current `build.rs::run` deliberately removes inherited Rust flags before invoking Bun. One explicit product configuration should drive both halves. [H4]

Acceptance is the final EXE's imports and actual clean-machine behavior, not the presence of `/MT` in one command. Inspect normal and delay imports with `dumpbin /DEPENDENTS` and/or `llvm-readobj --coff-imports`, and exercise dynamically loaded capabilities. Fail packaging on an unexplained non-system DLL requirement such as a VC runtime DLL not part of the declared OS baseline; do not quietly add a redistributable installer or sidecar DLL. A developer image can mask those dependencies. Microsoft's deployment guidance explicitly warns about this. [M6]

Static runtime security updates require rebuilding and redistributing the EXE. That servicing cost is the deliberate tradeoff for a no-install single-file product, not a reason to disregard the user's requirement. [M6]

### Native addon compatibility is a separate contract

Bun's Windows executable exports a defined `uv_*`, `napi_*`, `node_api_*` surface via `src/symbols.def`. Stock node-gyp addons generally delay-load `node.exe`; their hook looks for `libnode.dll` and otherwise returns the main executable's module handle. With Bun statically linked into our application, the main image can expose the corresponding supported exports directly. Generate them from the pinned list and verify an actual N-API addon; do not maintain a manually copied list or promise V8-ABI addon compatibility. [U7][N1][N2]

However, making symbols resolvable does not make a `.node` file disappear. The normal Windows addon loader uses `LoadLibraryExW` on a filesystem module. Native addons required by the packaged application must either be integrated through a deliberate static registration design, replaced with an appropriate built-in/native implementation chosen for that application, or reported as an unmet strict single-file packaging dependency. Do not silently extract arbitrary `.node` files or implement a custom PE memory loader just to claim one-file support. Build-time native-dependency discovery and real runtime qualification are both required. [U8][M7]

The static EXE also makes `GetModuleHandle(NULL)` refer to the image actually containing Bun, unlike the DLL design. This helps module identity but does not automatically enable Bun's standalone graph initialization or make the GUI executable accept Bun CLI arguments. [U9]

### Application identity and subprocess behavior

`node:child_process.fork()` defaults to `process.execPath` and launches it. In an embedded application that path denotes the application EXE, not a separate Bun CLI. JS Workers, spawning an explicit external command, and reexecuting the runtime are distinct behaviors. [B2]

If the application requires `fork`/`cluster` or `spawn(process.execPath, ...)`, implement and qualify a deliberate headless child-runtime mode in the **same EXE**, including script/module identity and IPC startup, rather than relaunching the GUI or adding a second runtime executable. Do not claim those APIs work merely because the embedded VM starts. The child-runtime mode must not open a GPUI window, and its termination must not accidentally end the parent application.

### Production resources and packaging output

- Vite remains the sole Solid/JSX application compiler. Preserve Bun/Node built-ins as runtime imports; bundle application JS dependencies and account for every emitted asset/module. The current Bun Vite production path bundles packages but does not enforce the QuickJS path's single-output policy. Add a Bun-specific embedded-product policy rather than reusing QuickJS's ban on Bun APIs. [S2]
- Compile application JS, required images/fonts/WASM/data and appropriate notices into the EXE, using explicit in-memory loading semantics. Do not assume any arbitrary `Bun.file(relativePath)` automatically sees PE resources. Runtime-created user files remain normal filesystem data.
- Embed the application icon, Windows GUI subsystem and application manifest in the host EXE. The current website distribution entry/build script demonstrate the existing GUI subsystem, resource embedding and `include_bytes!` pattern, but their runtime is QuickJS. Reuse the pattern, not the runtime or an assertion that Bun already supports it. [S3][S4]
- Emit the executable itself as the runnable release artifact. A published checksum, signature metadata or separately hosted PDB is not a launch dependency. Do not require an adjacent license/JS/resource folder; required license text can be embedded and exposed through an About/licenses surface, subject to actual redistribution obligations.
- Authenticode-sign the final EXE after native linking and application/resource embedding. Verify the signature and compute the published checksum afterward. Rewriting a PE or appending payload after signing invalidates the release evidence.
- The installed application directory may be read-only. Launch must not need to unpack runtime libraries or modify itself. User data and caches belong in explicitly chosen writable locations.

## Implementation stages and acceptance evidence

All rows are proposed work, not completed feature implementation. Start with Windows x64/MSVC and the pinned Bun revision, then qualify ARM64 separately; do not combine this effort with an unneeded Bun upgrade or a new GPUI cross-compilation pipeline.

| Stage | Change | Proof required |
| --- | --- | --- |
| 1. Static-link vertical slice | Prepare the pinned source and codegen; build one coordinated Rust application/runtime graph plus the native Bun/JSC dependencies; exclude Bun CLI main | A headless EXE loads Bun, evaluates a small in-memory ESM module and returns a byte frame, with no Bun DLL or JS file |
| 2. Runtime safety | Reuse the owner thread/queues; qualify Windows initialization, wake and teardown | Existing real-VM lifecycle scenarios pass, including restart after cancellation and host survival after JS `process.exit` |
| 3. Application bundle | Embed the Vite output and required assets with explicit source/module identity | Real Solid UI initial commit, input-to-update response, imports/top-level await, and each used asset/Worker path work without a checkout |
| 4. GPUI desktop | Run the actual GUI-subsystem application | Real Windows window opens, renders and accepts input; closing it shuts down promptly; Explorer-style launch does not require console handles |
| 5. Strict portability | Produce release EXE with intentional static CRT and system dependency closure | Copy only that EXE to a clean standard-user Windows environment; no Bun, Node, Visual Studio, VC redistributable installation, sidecar files or build-cache paths are needed |
| 6. Release | Sign final EXE and qualify each declared OS/CPU/architecture baseline | Published artifact identity, import/dependency evidence, signature checks, licenses, actual desktop acceptance and documented API limits |

### Files and integration points

- `crates/solid-gpui-bun-sys/build.rs`: replace the Windows rejection only with a real implementation; stop constructing a dylib for this Windows product. Separate source/toolchain preparation from final Cargo dependency resolution. Emit/consume an explicit native link manifest rather than reconstructing the CLI response file.
- `crates/solid-gpui-bun-sys/bun_embed.patch`: add a reusable embedded Rust library form and static-product graph integration; maintain allocator/startup/export ownership and CLI-main exclusion.
- `crates/solid-gpui-bun-sys/embedded/{runtime,lifecycle}.rs`, `src/lib.rs`, and `crates/solid-gpui/src/runtime/embedded.rs`: adopt the process graph before VM initialization and allow the host adapter to select a packaged graph entry without filesystem canonicalization. Preserve file-entry development and VM/session/thread/queue ownership. The current pointer-plus-length ABI can convey a UTF-8 graph identity as well as a disk path; do not add a graph-install function or source-buffer ABI unless needed by the chosen implementation. Do not add a second transport.
- Packaging orchestration and Cargo integration: prepare pinned checkout/codegen before Cargo resolves new library dependencies; carry required workspace patches, feature/config/profile settings, lockfile policy and linker inputs explicitly. Merely aligning compiler versions is not enough.
- `packages/solid-gpui-vite/src/index.ts` and the embedded build policy: validate Bun-compatible application/dependency/asset closure without QuickJS shims or a second JSX compiler.
- `scripts/tasks.ts`, `.github/workflows/embedded-bun.yml`: remove macOS task guards only for implemented targets; add real Windows VM and portable-EXE qualification, keeping check-only CI distinct.
- `scripts/host-embedded-candidate-smoke.sh`: replace Unix process-group orchestration with a platform-aware bounded check. The current smoke accepts forced timeout status 124; production acceptance also needs normal shutdown.
- `scripts/website-package.ts` and the eventual application packager: current Windows staging only copies a QuickJS EXE and does not enumerate Windows DLL imports. Add a deliberate Bun single-EXE product and clean-machine check, without changing QuickJS behavior accidentally.
- On implementation, synchronize `docs/runtimes`, `runtime-strategy`, `distribution`, `ci`, their affected website translations and ADR qualification wording. Do not change today's support matrix based on this research alone.

### Minimum meaningful Windows scenarios

1. Run the existing `host_embedded_counter` real-VM lifecycle cases: initial frame/press, startup queueing, backpressure/drain, EOF/beforeExit, Workers and HTTP/timers, busy JS/microtask termination, `process.exit`, rejected module and next-session recovery.
2. Add distinct Windows cases: first cross-thread wake, repeated sessions after libuv close, live FS/network/child activity at cancellation, and no-console GUI launch. Failed loop retirement must block reuse; assertions must not be suppressed to manufacture success.
3. Copy only the final EXE into a new directory containing spaces and CJK, change the working directory, remove Bun/Node/build paths, and exercise all startup/UI resources. Repeat from a read-only application directory under a standard user.
4. Verify actual imported and dynamically loaded modules; no app-private runtime DLL or `.node` may be silently extracted under the chosen contract. System DLL loads and normal application-data writes are not violations.
5. Exercise source-module identity, imports/top-level await, worker bundles/assets, used Bun services and any same-EXE child-runtime mode independently. A frame from a trivial bootstrap cannot stand in for real application content.
6. Repeat release qualification on a clean Windows machine. Hosted runners with development tools are not proof that a VC runtime dependency has been eliminated.

The public Bun documentation lists Windows 10 1809 or later. That is not yet our application's minimum: GPUI, graphics, the selected SDK/runtime imports and the pinned embedded Bun build all contribute to the supported product baseline. [B3]

## Verification and documentation scope

Completed: pinned upstream/local source analysis, Windows WebKit asset HEAD availability, pinned Rust compiler metadata, and Windows `crt-static` target configuration queries. No Windows compile, native link, EXE load, VM lifecycle or GUI run was performed. Therefore this report establishes an implementation direction and explicit experimental gates, not working Windows Embedded Bun support or complete Bun API compatibility.

Public docs/website ownership was checked through `docs/README.md` and `examples/website/README.md`. Their current macOS-only Embedded Bun support statement remains accurate. This private research record is outside the website's `docs/*.md` import glob; no public guide, translation, generated API, navigation or website build product changed. No unrelated website suite was run.

## Additional sources

[H1]: ../../crates/solid-gpui/src/runtime/embedded.rs
[H2]: ../../crates/solid-gpui-bun-sys/src/lib.rs
[H3]: ../../crates/solid-gpui-bun-sys/embedded/runtime.rs
[H4]: ../../crates/solid-gpui-bun-sys/build.rs
[H5]: ../../scripts/tasks.ts
[H6]: ../../examples/website/vite.config.ts
[H7]: ../../scripts/host-embedded-candidate-smoke.sh
[H8]: ../../.github/workflows/embedded-bun.yml
[H9]: ../../scripts/website-package.ts
[H10]: ../../docs/runtime-strategy.md
[H11]: ../../crates/solid-gpui/tests/host_embedded_counter.rs
[M1]: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices
[M2]: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order
[M3]: https://learn.microsoft.com/en-us/cpp/c-runtime-library/potential-errors-passing-crt-objects-across-dll-boundaries?view=msvc-170
[R1]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html#method.as_encoded_bytes
[B1]: https://bun.sh/docs/bundler/executables
[B2]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/js/node/child_process.ts#L763-L810
[B3]: https://bun.sh/docs/installation
[M4]: https://learn.microsoft.com/en-us/cpp/build/run-time-library-behavior?view=msvc-170
[U1]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/scripts/build/tools.ts
[U2]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/scripts/build/deps/webkit.ts
[U3]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/scripts/build/winsysroot.ts
[U4]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/scripts/build/flags.ts
[U5]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/.buildkite/ci.mjs
[U6]: https://github.com/llvm/llvm-project/blob/llvmorg-21.1.8/clang/include/clang/Driver/Options.td#L8856-L8857
[U7]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/symbols.def
[U8]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/bindings/BunProcess.cpp
[U9]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/bindings/c-bindings.cpp
[M5]: https://learn.microsoft.com/en-us/cpp/build/reference/ga-optimize-for-windows-application?view=msvc-170
[M6]: https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method?view=msvc-170
[M7]: https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw
[R2]: https://doc.rust-lang.org/reference/linkage.html
[N1]: https://github.com/nodejs/node-gyp/blob/main/addon.gypi
[N2]: https://github.com/nodejs/node-gyp/blob/main/src/win_delay_load_hook.cc
[S1]: ../../rust-toolchain.toml
[S2]: ../../packages/solid-gpui-vite/src/index.ts
[S3]: ../../examples/website/native/src/distribution.rs
[S4]: ../../examples/website/native/build.rs
[G1]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/standalone_graph/StandaloneModuleGraph.rs
[G2]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/runtime/jsc_hooks.rs
[G3]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VirtualMachine.rs
[G4]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/runtime/node/node_fs.rs
[G5]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/runtime/webcore/Blob.rs
[G6]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/runtime/cli/run_command.rs
[T1]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/bun_bin/Cargo.toml
[T2]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/bun_bin/lib.rs
[T3]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/scripts/build/rust.ts
[T4]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/Cargo.toml
[T5]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/scripts/build/bun.ts
[T6]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/scripts/build/compile.ts
[G7]: https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/exe_format/pe.rs

### Research-note checks

The assembled note was checked for all 16 distinct local reference targets, defined reference-style citation IDs and balanced fenced-code delimiters. No missing local targets or undefined citation IDs were found. These checks validate the research document, not the proposed Windows binary.
