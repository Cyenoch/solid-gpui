# Runtime implementation and validation

Date: 2026-09-07. Scope: runtime documentation, embedded frame transport, and QuickJS application reload. Existing unrelated workspace changes are excluded.

Filesystem paths in this record are normalized for publication. Repository
artifacts use repository-relative paths; `guest-build/` identifies the separately
retained VM build workspace, not a checked-in directory. Temporary directory
identities are omitted. Artifact hashes, measurements and outcomes are unchanged.

## Implemented

- Website reference navigation exposes **Choose a runtime** and **Runtime strategy**, backed by `docs/runtimes.md` and `docs/runtime-strategy.md`.
- External Bun retains binary stdio. Both embedded engines use the explicit `EmbeddedTransport` native frame bridge. Embedded Bun no longer replaces `process.stdin` or `process.stdout` for renderer traffic.
- Transport submission reports accepted-but-pressured frames with `false`; drain resumes bounded router staging and event dispatch. Candidate application publication also respects pressure.
- QuickJS development uses an external Vite watcher and the shared Bun build policy. The native host stages a fresh VM, captures bounded explicit JSON state, validates every open surface and application configuration, then switches generations on the foreground thread.
- Candidate staging uses its hard aggregate capacity without a soft-watermark pause midway through the transaction. Normal pressure applies after activation.
- Invalid builds, invalid candidate trees, and entry module exceptions (including throws after mounting) preserve the active generation. An active VM failure can recover on edit from its most recent capture checkpoint. Native effects belong in `onMount`; external side effects are not transactional.
- `bun run quickjs:dev` launches the counter fixture; application projects use `solid-gpui-quickjs-dev <entry.tsx> <native-host>` through their local package binary. The website Gallery has separate Vite content plugins, so its desktop build is not presented as a plain Bun bundle.

## Automated evidence

- Browser-conditioned renderer/application/transport tests: 40 passed, including candidate pressure ordering and strict state serialization.
- `cargo test -p solid-gpui --features quickjs runtime:: --lib`: 10 passed. Real QuickJS runs cover state handoff, rejected candidates, supersession, recovery, cancellation, platform APIs, and candidate staging above the soft watermark.
- `cargo test -p solid-gpui --features quickjs,test-support generation_preflight --lib`: passed, including multiwindow rejection without partial mutation and zero-window activation.
- `cargo test -p solid-gpui --features embedded-bun --test host_embedded_counter -- --nocapture`: passed with the actual embedded Bun/JSC library, covering frame traffic, pressure, HTTP, workers, timers, and shutdown.
- Core, tooling, and fixture TypeScript checks passed. QuickJS library Clippy passed with warnings denied.
- Website TypeScript and production builds passed; existing Vite configuration and generated WASM warnings remain.
- Development-tool integration against a temporary TCP host verified imported dependency edits, build-error rejection, recovery after fixing the dependency, and cleanup on child exit. Vite browser WebSocket/HMR servers are disabled for this launcher.

## Interactive evidence

The native counter ran in a real macOS window: click to 1, save and retain 1, introduce a syntax error and continue clicking the old generation to 2, then fix the entry and display `Reloaded: 2` in the same window. Closing the window ended the development process successfully. This smoke preceded the final pressure regressions; those were subsequently checked with real-VM automated tests.

The production website runtime page was opened in Chrome and visually checked for the runtime matrix, readable content, and navigation entries.

## Boundaries

No three-engine performance benchmark, component-local Fast Refresh, zero-copy transfer, or new Embedded Bun final packaging/signing pipeline is claimed. Embedded Bun remains the production direction for Bun-dependent applications, with current platform and release qualification explicitly documented. Rust and native contract changes require rebuilding/restarting the host.

## Static Embedded Bun verification (2026-09-16)

The research in `windows-embedded-bun.md` is now followed by an experimental
static application implementation. This section supersedes the earlier
packaging boundary above; it does not declare release support.

- The native build reuses pinned prebuilt WebKit/JSC libraries. Actual debug
  builds completed 1,255 native steps on macOS ARM64 and 1,234 on Windows x64.
- A Bun serializer built from revision `34cbb9a40b4bd1bd767d134a7065e66c2432a676`
  reports `1.4.0-debug+34cbb9a40`. A Windows base executable was built from the
  same revision for cross-target graph serialization.
- GPUI and Bun's `rlib` compiled and linked in one Cargo graph under
  `nightly-2026-07-20`, inheriting Bun's sole global Rust allocator.
- Actual macOS ARM64 and Windows x64 executables each completed two same-process
  VM sessions. Each session began at count 0, applied three press events, checked
  count 3 in the decoded tree, and shut down normally. The reusable probe is
  `fixtures/embedded-static-check.rs`, supplied through the packager's `--main`.
- The Windows executable was copied alone into the Windows 11 ARM64 VM and run
  as its logged-in user at medium integrity (`S-1-16-8192`), without Bun or Node
  installed. Its normal PE imports contain neither Bun/JSC nor VC runtime DLLs;
  Windows graphics, ICU, and other system DLL imports remain.
- A real macOS GPUI window displayed the packaged counter and accepted native
  interaction. Closing its native close control ended the process with code 0.
- Runtime verification found and fixed two actual startup defects: serialization
  must select Solid's `browser` export, and the eval bootstrap must use Bun's
  platform-native, drive-qualified identity on Windows rather than `/[eval]`.
- Native Cargo integration also needed compiler-driver runtime linkage, the
  upstream platform linker rather than the rlib-only lld override, target-specific
  Windows SDK/CRT compiler flags, and an absolute Windows resource-manifest path.
  These corrections were exercised in incremental final links. The full packager
  still needs a clean end-to-end rerun after the latest manifest changes.

The Windows test candidate is frozen at
`%TEMP%\solid-gpui-static-probe\application.exe`: no arguments run the two-session
counter probe, while `--gui` opens the counter window. Windows GUI acceptance is
being performed manually by the user; desktop automation has stopped.

Remaining gates include Vite-built application resource closure, Workers and
dynamic imports, Windows GUI acceptance, read-only and Unicode paths, release,
native ARM64, and final-artifact signing. Linux currently supports native
`--prepare-only`, not ELF application-graph packaging. The macOS debug executable
imports the toolchain's non-system UBSan dylib and is not a portable artifact.
The existing website release pipeline remains QuickJS. Public guides and their
Chinese translations now distinguish the static experiment from release support.

### Windows debug shader portability correction

The user's GUI test of the frozen candidate exited with `-2147483645`
(`0x80000003`). Its underlying error was `Error creating DirectWriteTextSystem`
with `os error 3`, not an identified AVX or stack failure. Debug initialization
canonicalized build-machine HLSL paths before calling `D3DCompileFromFile`;
DirectWrite's emoji rasterizer uses the same shader loader.

`vendor/gpui-windows/src/directx_renderer.rs` now embeds both HLSL modules and
serves `alpha_correction.hlsl` through an in-memory `ID3DInclude` implementation.
Unknown includes fail closed. Release bytecode loading is unchanged. A retained
native regression compiles all nine shader modules for vertex and pixel targets
and checks their DXBC output. The regression passed on the Windows 11 ARM64 VM
under `prlctl exec --current-user`, with no source directory or GUI window.

A separately named `application-shaderfix.exe` also completed both packaged
counter sessions (0 to 3, clean shutdown, same-process restart) in that VM.
It is available under `%TEMP%\solid-gpui-static-probe\`; the original
`application.exe` was not replaced. Both candidates' matching host-side PDBs
were retained. SHA-256:

- Original: `ed12cc0270f8b0a85ad37e8e553e19110c6b33a3a02ec03a8fef76896bbcf75e`.
- Shader fix: `c946b52b592f673d9214290da6eb582b19899b436df72c7082e58a96cdfe1877`.

The user subsequently provided a screenshot of this candidate's real GPUI
window displaying `Count: 2` and its Increment button. This confirms the basic
window/counter path, not broader desktop or GUI shutdown acceptance. The counter
label's hardcoded light foreground has poor contrast on the default white host
background. No desktop/window automation was resumed; PowerShell commands
through Parallels are explicitly authorized.

Further headless checks copied only this EXE into a fresh directory named
`路径 with spaces Δ`, with filename `计数器.exe`. Both the Unicode working
directory and the directory selected by `%SystemRoot%` passed the two-session input/restart probe with
empty `PATH`, cleared Node/Bun option variables, and a private temporary directory.
A create-new probe was denied in `%SystemRoot%`; no ACLs or account permissions
were changed. The EXE directory remained writable and contained only the EXE;
the private temporary directory was empty after both runs. This is not proof
against short-lived extraction and does not qualify a read-only EXE directory.

For the same SHA-256-qualified candidate, `llvm-readobj` found no delay-import
table, a 22,464-byte TLS template and seven callbacks. Actual Bun C++ compile
commands retained `/GA`, `/MTd`, and `/U_DLL`; Rust used `+crt-static`.
`llvm-pdbutil dump -modules` identified only the static debug CRT provider set
`libcmtd.lib`, `libvcruntimed.lib`, and `libucrtd.lib`. This qualifies the current
debug link policy, not release/native-ARM64 or every dynamically loaded OS module.

### Debug builtin source and full graph qualification

The first separate Windows graph candidate failed loading `node:worker_threads`
from a build-machine `build/.../js` path. Disabling C++ builtin hot reload then
exposed `ASSERTION FAILED: characters.size() >= strlen("(function (){})")`.
This was not the preserved shader-fixed counter. VM execution was initially
paused after the dialog; it resumed only after explicit user authorization below.

Offline evidence identified a coupled generator/loader policy bug: debug
`bundle-modules.ts` emitted zero offsets and lengths for every internal module,
while the patched C++ loader now consumed the static blob. The canonical patch
now passes `--embed-modules` for embedded/CI products and retains real source
spans even with debug assertions enabled. Development CLI hot reload remains
unchanged. The fresh `source-17f9d08bf2e17893782c008f` build contains 198 valid
function-wrapped module spans, a 2,835,208-byte builtin payload, and a 30,682-byte
`NodeWorkerThreads` span; no Windows compile command enables
`BUN_DYNAMIC_JS_LOAD_PATH`. Both Windows x64 and macOS ARM64 completed the full
native/serialization/unified-Cargo packaging driver against this clean source.

The macOS graph probe ran under LLDB without opening a GUI. It exposed two
fixture assumptions, subsequently corrected: Worker paths must resolve against
the virtual module directory, and `mountApplication` emits a
`ConfigureApplication` command after its first Snapshot. The probe now consumes
that startup control instead of mistaking it for a counter Patch.

The corrected macOS executable exited 0 after two sessions, each checking the
dynamic-import marker, Worker result 42, embedded text resource, main/Worker
native-load rejection without new `.bun-*` temporary entries, rejected default
fork/cluster interpreter requests, successful system `printf`, three counter
inputs, shutdown, and `CommitPoll::Ended`. All native assertions remain enabled.
Vite emits a separate dynamic chunk, but Bun folds it into the final entry; the
serialized graph has the application, Worker, and resource, not a separate
dynamic-module record. These results do not qualify Windows graph execution,
all transient filesystem activity, read-only installation, release, signing,
or native Windows ARM64. macOS debug still depends on the toolchain UBSan runtime.

After formatting, the complete canonical patch produced a new clean snapshot,
`source-db0a465e7829d7a1155c0398`, and both full packaging builds succeeded again.
At that stage, the macOS artifact repeated the two-session graph probe under LLDB
and exited 0. Intermediate SHA-256 values:

- macOS: `a7bb935c646933d632332868323e4cb1f8d2adda0153b03d50d076ff056dc2cc`.
- Windows: `c84c55c44a0ece1a4fc9888487bab694790055440d4f186eec4655a39fd77c99`.

These intermediate Windows files had not yet been launched at this point.
The original and shader-fixed Windows candidates remained
unchanged. Fixture and tool TypeScript checks passed. Website content/Markdown
checks passed (3 tests, 1,686 assertions); the previously recorded full website
runtime timeout remains unresolved and was not represented as passing. Public
distribution, runtime-strategy, runtime-selection, and troubleshooting guides
and their Chinese translations were synchronized; the website consumes those
guides directly, and its QuickJS release pipeline was unchanged.

### Resumed Windows graph qualification

The user explicitly authorized new, hash-verified graph candidates in independent
directories, running only `--check-graph` with a 60-second outer bound and no
desktop operations. No preserved counter executable was replaced.

The first resumed candidate (`c84c55…77c99`) exposed missing inherited VM
environment: `os.tmpdir()` produced `undefined\temp`. Windows process startup
already captured the OS environment, but `EmbeddedVm::new` did not import it into
the transpiler environment. The overlay now owns the VM before fallible
`load_process()` initialization, independently of `.env` autoload settings.
The Rust probe now returns ordinary errors instead of calling panic handlers.
Candidate `398700…43011` confirmed both changes: temp enumeration and the external
system command ran; a Worker timeout returned exit code 1 without a Rust panic.

Temporary stage markers exposed `ENOENT` for `embedded-static-graph.worker.js`
through a mixed-separator virtual module key before Worker entry execution.
The Worker resolver found the graph record but discarded its canonical key and
returned the original native-separator path. The patch now returns the borrowed
canonical result from `graph.find`, matching the graph trait contract without copying
or filesystem fallback. A clean snapshot, `source-7a27e17e08aa8945611cb6ee`, applied
the complete patch and built both platforms successfully.

Instrumented Windows candidate `bff5be…850b9` passed both sessions with the
original four-second Worker deadline. Construction-to-reply measurements were
3,856 ms and 3,043 ms with x64 emulated on Windows ARM64. After removing all stage
markers, the fixture uses an eight-second step deadline and a 30-second host
commit deadline, retaining the 60-second external process bound. This margin
follows a proven path-resolution fix, not a timeout-only workaround.

The final trace-free probes also require `ERR_DLOPEN_FAILED` for native loaders
and `ERR_INVALID_ARG_VALUE` for default fork/cluster rejection, rather than
accepting any Error. Native policy checks use canonical graph keys so Windows
`process.dlopen` reaches its bundled-file branch rather than merely missing an
OS path. Main and Worker checks passed on both platforms.

Final artifacts:

- Windows: `target/static-graph-qualification.exe`, SHA-256
  `4f16bec6a1f77062fce644065cc04109cbc606788c2375ecdebdffef7ee42c5d`.
- macOS: `target/static-graph-qualification-macos`, SHA-256
  `b063ef5fe0dee4a146e5dce9d74622479d348fd24ddfefdff65c98d99ed3f64d`.
- Windows symbols: `target/static-graph-qualification.pdb`, copied from the
  matching application target; byte comparison passed (1,268,740,096 bytes).

Windows verified the hash and ran the EXE alone as `application-graph.exe`
in a newly created private temporary directory.
With empty `PATH`, private `TEMP`/`TMP`, and Bun/Node configuration variables
cleared, both sessions passed JSX, dynamic import, Worker reply 42, resource
content, native/child policies, three counter inputs, and clean shutdown. It
exited 0; the private temp directory was empty afterward. The final macOS
artifact repeated both sessions under LLDB and also exited 0. Native assertions
remained enabled, and the transfer service was stopped after execution.

This does not qualify a read-only EXE directory, a separate standard-user
account, all transient filesystem activity, full system dependency closure,
broader GUI behavior, release, signing, or native Windows ARM64. The earlier
shader-fixed manual GUI and Unicode/non-writable-CWD evidence remains distinct.
macOS debug still depends on non-system UBSan. Public bilingual guides now
describe the Windows graph result without claiming production portability.
Fixture/tool TypeScript checks and final website content/Markdown checks passed
(3 tests, 1,686 assertions). No SDK contract changed, so no generated API content
required regeneration; the website reads the updated bilingual guides directly.

The final Windows EXE's CodeView GUID and retained PDB GUID both equal
`{EBC02879-126C-CE72-4C4C-44205044422E}`, age 1. Its PDB names `libcmtd.lib`,
`libvcruntimed.lib`, and `libucrtd.lib`; normal PE imports contain no Bun/JSC or
VC redistributable DLL. A further bounded CLI run of the same hash passed both
sessions while collecting 77 module snapshots: 48 distinct modules, comprising
the main EXE and 47 modules under Windows System32/WinSxS. No non-system DLL path
was observed. Inbox `ucrtbase.dll`, `msvcp_win.dll`, and `msvcrt.dll` were present
transitively; static CRT linkage in the application does not mean no OS DLL can
use a dynamic C runtime.

A 64-KiB-buffer filesystem notification observer on that run's private temp tree
reported no create/change/delete/rename events and no observer error. A separate
control using the same observer delivered Created, Changed, and Deleted events.
This strengthens the selected temp-path evidence but is not kernel-wide tracing,
an exhaustive module-load trace, or proof for unexercised GUI/driver paths.

The earlier website timeout was investigated with temporary phase markers.
Server creation took about 440 ms; fixture import included a 20.43-second native
host compilation, while runtime verification took about 617 ms and server close
about 2 ms. That diagnostic run passed in 24.53 seconds. After removing all
markers, the unchanged full website suite passed: 8 tests, 3,917 assertions,
6.26 seconds, with a warm native build. No timeout was increased or website
runtime behavior changed; a cold native build remains relevant to its 30-second
test budget.

Remaining acceptance prerequisites are separate from this authorized CLI run:
manual GUI coverage; an isolated non-administrator account; a read-only EXE
location without changing the user's ACLs; release/signing/native-ARM64 runs.
The Windows guest exposes no `cargo`, `rustc`, or `fxc` command on PATH. Source
inspection shows GPUI's release shader generation is Windows-host-gated and
invokes `fxc.exe`; macOS cross-host release shader preparation was not tested.
No signing identity was provided, and no account, ACL, tool installation, or
desktop changes were made. The staged `.xwin-cache` additions/deletions were
left untouched rather than rewriting the user's index; the root Cargo.lock
delta only records the Bun sys crate's serde/serde_json build dependencies.

### Six-gate implementation and native ARM64 qualification

The follow-on work split independent-account, read-only-installation, dependency,
release, signing and native-ARM64 gates across five agents, then integrated the
source changes centrally. The proposed isolation/signing PowerShell wrappers were
removed: operator provisioning and standard SignTool commands do not need another
maintained framework. No account, ACL, certificate/trust, or desktop change was
made. Independent-user and read-only-installation acceptance remain blocked on
operator-provisioned conditions; the current enabled local account is directly in
Administrators even though its filtered .NET token group list omitted that SID.

The retained dependency gate is `scripts/check-embedded-dependencies.ts`: one LLVM
inspection, architecture validation, reviewed normal/delay imports, before/after
image hashing, and fail-closed diagnostics. Both real Windows debug images pass
with 30 normal imports and no delay imports. Real PE negative fixtures reject a
wrong architecture, delayed `bun_embed.dll`, dynamic `vcruntime140.dll`, and an
undeclared system import; non-PE input fails as unusable. This is not dynamic OS,
API-set, graphics, font, configuration, or resource closure.

The packager now checks native manifest pin/prebuilt/profile provenance, actual
PE/Mach-O architecture, and final linked graph SHA-256 before writing output.
Publication retains the linked executable's permissions. An actual macOS package
overwrote a deliberately non-executable private test output, restored mode `755`,
and directly passed both full graph sessions and exited zero in 0.62 seconds.
That separate output has SHA-256
`1d4c48a991e432bf4b869c119fa4bd23295c8f35938db27c35e570997c2ba441`;
the previously frozen macOS/x64 candidates remain unchanged.

Windows release shader generation now follows target OS/debug assertions rather
than build-script host/profile, tracks the shared HLSL include, and fails when a
real FXC prerequisite is unavailable. Windows-debug and non-Windows target smoke
checks skip shader generation; a Windows-release target fails clearly without FXC
even when the build script itself was compiled with debug assertions. Those checks
did not claim real DXBC compilation. The CI comment no longer conflates a debug
cross-platform job with release FXC coverage.

The pinned ARM64 Bun base initially reached final link but lacked `libcmtd.lib`
and `libcpmtd.lib`. Compiler diagnostics confirmed ARM64 selection and SDK paths.
Microsoft's separate `Microsoft.VC.14.44.17.14.CRT.ARM64.Desktop.debug.base.vsix`
was absent from the tested xwin selection. Its official SHA-256
`317e00c77dc51611fd7bec279b1ef67790bc73df8fe5bbc19432213b84d5e7ee`
was verified, and only missing original ARM64 libraries were copied into the
host-side sysroot. No static archive modification or artificial linker path was
needed. A transient Rust metadata DNS failure was followed by a successful base
link; missing vendor PDB warnings are not runtime DLL dependencies.

The full static packager then produced
`target/static-graph-arm64-qualification.exe`, SHA-256
`c4c577be5155b0f8fa55bb28aec01017720e835aedaef426c0e00523b665d649`.
Its matching retained PDB is `target/static-graph-arm64-qualification.pdb`
(1,279,815,680 bytes), GUID `{9D026098-CF67-2515-4C4C-44205044422E}`, age 1,
matching the EXE CodeView record. Build time was 113.51 seconds.

The first native-process observation harness lost C# string quotes while crossing
the command boundary. It produced no accepted runtime result; inspection afterward
found no surviving owned ARM64 process. The corrected observer transfers its C#
source as data, fails before launch on setup errors, and terminates only its owned
child on harness failure. In a new directory, the verified ARM64 image passed
`--check-graph` under the current user, empty PATH and private TEMP/TMP, with the
60-second outer limit. `IsWow64Process2` reported process `0x0000`, native
`0xAA64`; both sessions, both PASS lines, and exit zero were observed. Temp was
empty afterward. This is native ARM64 execution inside the Windows VM, not ARM64
GUI, physical-device, isolated-account, read-only-installation or release proof.

The bilingual distribution/runtime-strategy/runtimes/troubleshooting guides now
separate these gates and document direct signing/identity/timestamp checks. The
website consumes these guides directly. Adding PowerShell examples exposed a
missing loaded Shiki grammar; the existing language list now includes PowerShell.
The full website suite passed with 8 tests and 3,920 assertions after that fix.
Tool TypeScript and focused formatting checks also passed. No SDK API changed,
so no generated API content was regenerated; the QuickJS release path is unchanged.

### Windows native release environment follow-up

The user reported the Windows development tools were installed. The earlier PATH
inventory was not an installation inventory. Explicit lookup located Visual
Studio Community 2026, SDK 10.0.26100.0 ARM64 FXC and SignTool, Bun 1.4.2, pinned
Rust nightly-2026-07-20, and VS-bundled CMake 4.3.1/Ninja 1.13.2. Initially observed
LLVM was 23.1.1 and rust-src was absent. The user first chose to keep the existing
installation, then explicitly superseded that choice by authorizing correction of
the VM's installed development environment. That authorization does not grant
account/ACL/trust/signing or desktop changes.

The fixed nightly's rust-src component was installed successfully. The official
LLVM 21.1.8 ARM64 portable archive was acquired through Motrix and verified against
the release asset SHA-256
`f214b1226d8de005b5f691dd29d9dfea2b49e22d0de445429916173dbb626f7f`.
An isolated native-build workspace was created (identified here as `guest-build/`).
Source, pinned Git objects, the real pinned ARM64 serializer, release WebKit and
serviced UCRT were transferred with hashes.
The release WebKit SHA-256 is
`d88e7dda7938dd402ee0314d2e653145618dcfb55770345549c6de0a3688496d`.
macOS AppleDouble metadata initially polluted transferred Git pack-index names;
metadata-free source archives corrected the transfer, preserving the original
copies separately. The user index and pinned source commit were not rewritten.

Subsequent explicit selection/version checks found the system LLVM directory also
contained matching clang/clang-cl/LLD 21.1.8. No compiler-discovery source workaround
was needed. The native Windows ARM64 release attempt uses the real SDK FXC,
prebuilt release JSC, existing driver, and a separate output name. Its initial
startup and source-preparation failures, and their verification, are recorded
below. Starting a build is not release acceptance. Signing still requires a
supplied publisher identity, timestamp endpoint and explicit approval.

### Native release startup and source preparation (2026-09-17)

The guest source snapshot initially lacked installed repository dependencies.
`bun install --frozen-lockfile` installed the declared 190 packages, including
`@solidjs/compiler@2.0.0-rc.6`. This corrected auto-resolution of an unrelated
compiler cache version, but exposed the real Windows ARM64 WASM fallback failure:
`wasi.initialize is not a function` under Bun 1.4.2. The packager consumes Vite JS;
the repository's test-only JSX preload must not eagerly require this compiler.
`scripts/solid-jsx.ts` now loads it inside the existing JSX/TSX handler. No WASI
shim, compiler replacement or application-side fallback was added.

The new regression runs a plain Bun command with an unavailable compiler module.
It failed before the loader change and passed afterward on macOS and Windows
ARM64. The real Windows packager `--help` then returned zero. Tools TypeScript
and focused formatting checks passed; the combined regression/website run passed
9 tests with 3,922 assertions. This does not qualify Windows ARM64 Vite/JSX
compilation itself.

The native build uses the installed `VsDevCmd.bat` with
`-arch=arm64 -host_arch=arm64`, rather than fabricating Visual Studio environment
variables or bypassing Restricted PowerShell policy. Existing Git for Windows
supplies Perl 5.42.2; only the build process PATH was adjusted. Log files now use
shared-read handles so build progress is observable while the owner drains both
streams. No maintained launcher or compiler-forwarding layer was introduced.

The first real release graph reached builtin module generation, then failed with
`ENAMETOOLONG`: upstream passed every generated module's absolute path to the Bun
CLI. The canonical patch now passes relative input paths from the explicit
temporary-module working directory, retaining the CLI, root/output paths and
compiler flags. The patch applies cleanly to the exact pinned source. Its SHA-256
is `f6c57ba5ac8a3e3c809c9bfb66175f8329b1d768ca388d7d873f6d1990fef9b2`;
the fresh prepared source key is `source-01f64c16e72f058011c65ac3`. Under the same
long Windows workspace path, release codegen completed 198 internal modules,
13 native modules and 92 internal functions across 17 files. No older frozen
EXE/PDB or prepared-source tree was changed to obtain that result.

Source preparation then stopped on zstd's archived test symlinks. Native
`CreateSymbolicLinkW` probes returned Win32 error 1314 both with flags 0 and with
`SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE` (2). The agent did not skip archive
errors or manufacture `.ref` cache markers. The user explicitly selected enabling
Developer Mode, then required all desktop/security UI actions to remain manual.
The agent stopped UI automation; the user reported enabling the mode. The same
normal-user API probe subsequently returned flags 2 success / error 0, while
flags 0 still failed with 1314. No elevated build was substituted.

The ordinary-user native release build has resumed. A separate real
`cargo check --locked --release --target aarch64-pc-windows-msvc -p gpui-pre-windows`
uses the installed ARM64 FXC and a separate target directory to verify release
shader preparation independently. Neither launch is final release/GUI acceptance;
their outcomes must be recorded before claiming those gates complete.

The finalized builtin-generator patch also resolves the temporary-module root
before changing the child working directory, preserving direct invocation with
a relative build directory. Final patch SHA-256:
`53a16b3de9d1f7d1b050f30bf08ed98f45f75ed62c7a42983a179ca7c35f6c63`;
prepared source: `source-91b53fe8547751e4525bd3a3`. Running the real generator with
`--debug=OFF build/relative-root-check --embed-modules` completed all 198 modules,
13 native modules and 92 internal functions. Its builtin binary payload was
byte-identical to the absolute-root build:
`0581552f1efce5c9c7f97494626e264663d3f6cc6743ed33521ad51c9532fc94`.

Both earlier long-lived guest command transports ended with `prlctl` exit 255,
without compiler exit reports. Windows had not rebooted, and a subsequent scoped
inventory found no surviving build drivers. Logs were truncated during native
C++ compilation and Rust checking; no source failure was established from that
interruption. A 20-second guest session survived concurrent guest commands, so
ordinary concurrent `prlctl exec` alone did not reproduce it. The full package
build was resumed alone with the finalized patch; no diagnostic outcome is being
invented for the lost sessions.

The interrupted renderer check had already run the actual release shader build
script and produced `shaders_bytes.rs` (368,876 bytes). All 18 generated arrays
were materialized only as verification artifacts and accepted by the installed
ARM64 SDK `fxc /dumpbin`: nine vertex shaders reported `vs_4_1` and nine fragment
shaders reported `ps_4_1`, all with DXBC containers. This proves real source-based
release shader preparation, not completion of the interrupted Cargo check or
GPU/GUI execution. No precompiled shader was added to the repository or used as
a replacement build input.

`bun run website:build` passed, including native binding generation, WASM release
compilation, website typechecking, and Vite production output. Existing Rust
future-incompatibility/dead-code, Vite native-config import, generated wasm-bindgen
eval, and bundle-size warnings were not suppressed. The final documentation and
preload regression run again passed 9 tests / 3,922 assertions. Tools TypeScript
and focused formatting checks passed. No desktop automation was resumed after
the user's instruction to keep all GUI actions manual.

### First native ARM64 release artifact and STL ABI diagnosis

The finalized-patch native build completed with exit zero in 697.37 seconds,
including 1,260 native edges and an optimized Cargo release link. The frozen
guest candidate `application-arm64-release.exe` is 86,235,648 bytes with SHA-256
`31f8d8c82b11adc712cee4bb502170caa4993a2e3025f1d5834b938c131d11c0`.
Its preserved `application-arm64-release.pdb` is 539,480,064 bytes, GUID
`{0C997B17-9EB4-A3EE-4C4C-44205044422E}`, age 1, matching the EXE CodeView record.
The direct-import audit passed: ARM64, 18 reviewed normal imports and 11 reviewed
delay imports, no violations. Unlike the debug candidate, this release really
uses the upstream delay-load policy.

The fresh isolated runtime probe verified that hash and native execution
(`ProcessMachine=0`, `NativeMachine=0xAA64`), but exited `0xC0000409` before either
session result. Private TEMP was empty. This is a failed release qualification,
not success inferred from compilation or import checks.

Installed ARM64 LLDB 21.1.8 initially could not start because `liblldb.dll` imports
`python311.dll`. The official Python 3.11.9 ARM64 embedded ZIP was added only to
the workspace's tools directory; archive SHA-256
`1a6dae49d15320270a7141f93b574ff7686a7a526efa65e63ddbebf9b409929a`.
`python311.dll` verified Authenticode Valid with Python Software Foundation as
signer. Process-local PATH made `lldb --version` succeed. No system Python,
certificate store or trust policy was changed. The first LLDB command attempt
stopped at argument parsing before launching the application; the corrected,
bounded attempt captured the fault. LLDB itself reported an internal error on
quit, so its wrapper exit code is not a runtime verdict.

The captured addresses were resolved offline with the exact PDB using
`llvm-symbolizer`, avoiding another application launch. The crash is `abort` from
`WTFCrashWithInfo`, reached at pinned WebKit
`TimeWithDynamicClockType.cpp:144`: `RELEASE_ASSERT(a.m_type == b.m_type)` in
`operator<=>`, called through `Condition::waitUntilUnchecked` by parallel GC.
Disassembly proves argument displacement: the prebuilt comparison consumes its
two operands in x0/x1 and returns the ordering in w0; the newly compiled caller
supplies a hidden result address in x0 and operands in x1/x2.

A minimal C++ call returning `std::partial_ordering`, compiled with LLVM 21.1.8
and the actual `/std:c++23preview` flag, reproduces the ABI difference. Installed
MSVC 14.51 emits an `sret(std::partial_ordering)` hidden pointer. The existing
fixed 14.44 CRT/STL emits a direct `i8` return, matching the prebuilt JSC. The same
direct return was confirmed with the native Windows compiler, including inside
the real VS developer environment, when passed `/winsysroot` for the fixed SDK.
No assertion, clock comparison, memory-layout shim or compiler safety check was
changed.

The already acquired/licensed fixed SDK was copied to guest-private
`sysroot-14.44`, preserving original libraries. The metadata-free transfer archive
SHA-256 is `76407c7a3c61477504efba8ffec86772de54f5e32ddc326fe751ed1d8d1a3e88`.
Installed Visual Studio remains unchanged. A distinct
`application-arm64-release-msvc1444.exe` build now uses the existing packager's
`--winsysroot` option; the failed EXE and matching PDB stay frozen for comparison.

Inspection of the actual manifest rejected that first fixed-sysroot attempt:
upstream `resolveConfig` silently ignored explicit `--winsysroot` on a Windows
host. The owned `native-release-1444.cmd` process tree was stopped before accepting
its output. The canonical patch now resolves explicit Windows sysroots before
the cross-host branch, reuses the existing completeness/case checks for any
selected sysroot, and includes the existing serviced UCRT overlay whenever a
sysroot is used. Native builds with no explicit sysroot keep their existing VS
environment behavior. No duplicate resolver or new download/SDK layer was added.

Patch SHA-256 is now
`7612bacc3187cc317bc17fbe7ebd7054d1c771b8b00087ccecafb4f47b9b99ca`,
with clean prepared source `source-c8dc96fc79861685eff8c1b0`; it applies to the
exact pinned checkout. The native manifest now contains target
`CFLAGS=/winsysroot .../sysroot-14.44 /MT`, linker `/winsysroot:` for that directory,
and `/libpath:` for serviced UCRT `10.0.26100.8249/arm64`. These observed generated
arguments distinguish the real fix from merely passing an ignored option.

A direct native regression used the real toolchain resolver against both prepared
source revisions. Before the fix it failed with `explicit native sysroot was
ignored` (`actual: undefined`). After the fix it passed explicit-path selection,
real SDK validation, rejection of a missing SDK, and unchanged no-sysroot native
behavior. This was a throwaway executable check of the pinned build interface,
not a source-text assertion or a mock toolchain test.

### Native ARM64 release qualification completed to headless-probe scope

The corrected explicit-sysroot build completed in 796.11 seconds with exit zero.
The frozen guest artifact is
`guest-build/application-arm64-release-msvc1444.exe`,
86,260,736 bytes, SHA-256
`6175737a9ade8db3e4d8bdba8ebf63865e8d54dd4d07e5be5baa3b3a43d1ef29`.
Its separately retained PDB is 534,163,456 bytes, GUID
`{A3D900F2-FCD5-23A7-4C4C-44205044422E}`, age 1, matching the EXE's CodeView
record; debug information is unstripped and has no conflicting types. The failed
14.51 candidate and its own PDB were not overwritten.

A fresh verified copy in a unique private temporary directory ran `--check-graph`
with empty PATH, private TEMP/TMP, cleared Bun/Node overrides
and the existing 60-second outer limit. `IsWow64Process2` returned process `0`,
native `0xAA64`. Both sessions reported initial count 0, three presses, count 3,
and clean shutdown. Both graph/policy and same-process restart PASS lines were
present. Exit was zero, the EXE hash was unchanged, and private TEMP remained
empty. This original runtime reproduction no longer triggers the fail-fast.

The actual corrected EXE passed the retained main-image import gate: 18 reviewed
normal imports and 11 reviewed delay imports, no violations. The final package's
generated shader binding file is byte-identical to the 18 SDK-disassembled DXBC
outputs, SHA-256
`1f4a1402802fcfc81724ce1a93c12c06a9bf6b171043a3af322a81f36355bb05`.
Neither result is exhaustive runtime/system dependency closure.

The English and Chinese distribution, runtime-strategy and runtimes guides now
distinguish native ARM64 release headless evidence from public release/GUI
acceptance. Troubleshooting records the preload, command-line length, symbolic
link and STL ABI failures and their actual remedies. The website consumes these
guides directly. Final verification passed: 9 tests / 3,922 assertions for the
preload regression and website suites, tools TypeScript, focused formatting,
and the full website production build. Existing compiler/bundler warnings were
not hidden. There is no SDK API change or QuickJS release-contract change.

Temporary transfer entrypoints and ABI/config/debugger command scripts were
removed after verification; EXEs, PDBs, build caches and diagnostic logs remain.
No files were staged or committed and the user's index was not rewritten.
The public Windows status remains experimental. Release GUI is deliberately
left to the user, who prohibited further agent desktop operation; physical-device
coverage, independent standard-user/read-only-installation evidence, exhaustive
dependency closure and publisher signing remain separate unqualified gates.

### Publication-safe path references

Removed workstation usernames, private workspace identities and literal local
artifact paths from maintained guides and research records. Repository citations
now use relative links; dependency citations retain their original pinned commit
or crate version through public source URLs. Historical captures explicitly label
normalized artifact identities rather than claiming that files were relocated.
The sanitized sample preserves module identities, addresses, UUIDs and timings.

Runnable examples use repository-relative outputs or explicit tool, sysroot and
installation-prefix inputs. English and Chinese guides remain synchronized, and
the website consumes those sources directly. The historical recording helper now
uses the platform temporary-directory API; its TypeScript transpiles, but its
retired gallery entry is absent, so full helper execution is not claimed.

Verification passed: identifying-workstation-reference scan, scoped documentation
path scan, four evidence JSON parses, nine preload/website tests with 3,922
assertions, and the final website production build. Every Linux installation
example command rejects an empty prefix before executing a command. No installer,
desktop operation, staging or commit was performed. Frozen EXEs/PDBs and raw
diagnostic logs remain untouched. Platform-defined filesystem contracts, upstream
source snapshots and intentional absolute-path test data were not mechanically
rewritten as if they were local installation configuration.

### User-confirmed native ARM64 release GUI (2026-09-17)

After being directed to launch `application-arm64-release-msvc1444.exe --gui`,
the user supplied a screenshot of the real Windows fixture window. It displays
`Dynamic: dynamic-module-ok`, `Worker: 42`, `Resource: embedded-resource-ok`,
`Native extraction: blocked`, `Child interpreter: guarded`, and `Count: 3`.
The user subsequently reported normal exit and normal window resizing.

This adds manual basic rendering, counter interaction, resizing and closure
evidence for the release candidate identified above by SHA-256. It supersedes
the earlier absence of release GUI evidence, not the separate debug candidate's
status. The observations are user-provided; no agent desktop operation or repeat
GUI run was performed. No numeric GUI exit code, isolated GUI environment or
frame-time measurement was captured. This is VM evidence, not physical-device,
independent-standard-user, read-only-installation, full dependency-closure or
signing qualification. The bilingual distribution, runtime-strategy and runtimes
guides now distinguish this manual evidence from the isolated headless results.

Documentation verification passed after this update: all eight website tests
with 3,920 assertions and the full website production build. Existing compiler
and bundler warnings remain visible. Runtime source and frozen artifacts were
not changed; no new GUI run was requested.

### Independent standard-user and read-only installation acceptance (2026-09-17)

The user explicitly authorized a dedicated local standard account, changes only
to the new qualification directory's ACLs, and cleanup afterward. Signing was
deferred. The frozen native ARM64 release candidate remained unchanged:
`application-arm64-release-msvc1444.exe`, SHA-256
`6175737a9ade8db3e4d8bdba8ebf63865e8d54dd4d07e5be5baa3b3a43d1ef29`.
The VM reported Windows build `10.0.26100.0`.

The temporary account had a SID distinct from the builder, belonged only to
built-in Users (`S-1-5-32-545`), and ran at medium integrity (`S-1-16-8192`). Its
token contained no Administrators SID, including deny-only membership; the local
account database independently excluded administrator membership. This was a
separate account, not the builder's filtered administrator token.

Two fresh EXE copies ran under that identity:

| Case | Installation access checks | Runtime result |
| --- | --- | --- |
| Writable control | Directory create-new and non-truncating EXE write-open succeeded. | Native ARM64, both graph/input/restart sessions and both PASS lines, exit 0; 3,692 ms elapsed. |
| Read-only installation | The account had read/execute access only; directory create-new and EXE `Open`/`Write` both returned access denied before and after execution. | Native ARM64, both graph/input/restart sessions and both PASS lines, exit 0; 637 ms elapsed. |

The read-only copy used the relative identity `安装 with spaces Δ/应用.exe`.
Its directory and EXE were owned by Administrators; the test account could not
change the installation through inherited write permissions. Working files and
TEMP/TMP were outside the installation. Both application processes used an empty
PATH, private writable TEMP/TMP and cleared Bun/Node overrides. Each installation
contained only its EXE. `IsWow64Process2` returned process `0`, native `0xAA64`;
stderr was empty, the temporary directories were empty at exit, and both EXE
hashes remained unchanged. The recorded durations are individual completion
times, not a controlled performance comparison or startup benchmark.

The disposable controller returned exit 0. It removed the temporary account,
profile and test installation with no cleanup errors, and rechecked the original
EXE hash. Its scripts and staging directory were subsequently removed from both
hosts. No maintained qualification wrapper, account-management feature, global
security-policy change or signing operation was introduced. The original EXE
and PDB remain retained; the newly supplied Windows source checkout was not
used to rebuild or replace the qualification candidate.

Earlier disposable-harness failures did not reach the application. First, a
credentialed process launch exceeded the documented 1,024-character limit of
[CreateProcessWithLogonW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createprocesswithlogonw).
A short PowerShell command with redirected stdin fixed that boundary. Next, a
member-type check incorrectly compared a localized principal label with English
`User`; resolving members through their SIDs removed that locale assumption.
Preflight also corrected a collision with PowerShell's constant `$Error`
variable and exercised actual access-denied handling. Each failed attempt
removed its account/profile/installation and preserved the source hash.

Hash-verified private reports are retained under `target/windows-isolation-evidence/`
(not published or committed). The accepted `result.json` has SHA-256
`d9cf7b86cd57d0041b1a6b6ce7cb37723567a874dbfc655aab334aacfc8f1777`.
These results close the standard-user and read-only-installation headless gates
for this ARM64 release fixture in this VM. They do not extend the earlier manual
GUI observations to that account or installation, and do not establish exhaustive
dynamic dependency closure, physical-device coverage, broader desktop behavior
or publisher signing.

### Developer-guide consolidation and website verification (2026-09-17)

The English guides and their Chinese counterparts now separate responsibilities:
distribution owns packaging commands, prerequisites, platform evidence and release
checks; runtimes explains selection and workflow; runtime strategy describes the
architecture; troubleshooting maps symptoms to remedies. Candidate hashes, PDB
identities and investigation chronology remain in this journal rather than being
repeated in developer guides. Hot-reload pointers and ADR-0017 now link to the
maintained platform status without changing the runtime ownership decision.

The guides distinguish the successful writable-installation control from the
access-denied checks in the read-only installation. They also distinguish the
separate basic GUI observation from both headless isolation checks, and preserve
the outstanding dependency-closure, broader desktop/device and signing limits.
No runtime source, frozen candidate, PDB or SDK API changed during this refinement.

Final verification after the guide edits:

- `bun --conditions=browser test scripts/solid-jsx.test.ts examples/website/tests`:
  9 passed, 0 failed, 3,921 assertions.
- `bun run website:build`: passed, including native binding generation, WASM
  generation, route generation, TypeScript checking and production bundling.
- A disposable check of 11 edited Markdown files validated 110 local links and
  59 Markdown anchors with no failures. The focused publication-hygiene scan
  found no personal username or workstation-root paths in those guides.
- The local production website rendered the distribution guide in Chromium.
  Its table-of-contents jump and language switch displayed the revised static
  packaging introduction and command blocks in English and Chinese.

Existing Rust future-incompatibility/dead-code, Vite native-config import,
generated WASM glue `eval`, and chunk-size warnings remain visible. These checks
do not constitute another Windows candidate build, runtime qualification, or
publisher-signing run. The local preview server and browser tab were stopped
after inspection; no maintained validation wrapper was added.
