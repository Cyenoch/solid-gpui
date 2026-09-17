# Distributing a native application

The runtime strategy assigns Embedded Bun to production packaging for Bun-based
applications and QuickJS to applications whose main capabilities live in Rust.
External Bun serves rapid development. The first part of this guide documents
the QuickJS website package, which is the only pipeline this repository ships and
verifies. The second part documents an experimental Embedded Bun workflow that
links a Bun/JSC runtime and the application's module graph into one executable;
it has limited runtime verification today, not release support. See
[Embedded Bun static applications](#embedded-bun-static-applications) for its
inputs and current qualification gates, and
[runtime strategy](runtime-strategy.md) for the intended roles.

The website is the reference application package. It compiles its Solid UI into
one ESM module, embeds that module in the Rust executable, and runs it with
QuickJS. Production packages omit inline source maps. End users need neither Bun
nor Node, a repository checkout, nor a JavaScript bundle beside the executable.
Rust owns native services and rendering;
the same website composition also runs through Bun during development.

## What you ship

Three artifacts have three owners, and only the last one is a deliverable:

- **Bundle** — the output of `bun --bun vite build`: one JavaScript entry module
  for the host to execute. It contains no native code and no installer.
- **Native executable** — the GPUI host built by Cargo for a target and profile.
  `bun run generate` (`solid-gpui prepare`) and a Vite build both build one for the
  build host; a cross-target executable comes from the same manifest with an
  explicit `target`. `.solid-gpui/artifacts.json` records the executable and bundle
  paths, and `solid-gpui preview` runs the two together on the build host.
- **Distributable** — executable plus bundle plus assets, metadata, licences, and a
  signature, assembled by your own packaging script. No build command in this guide
  produces one.

Platform capability follows the same split: this repository verifies the QuickJS
website package on the targets recorded below, while the Embedded Bun static
packager remains experimental and claims no supported or verified target. A passing
local build of your own application is evidence for your application, not a
qualification of another platform.

## Build environment

Install the Bun version pinned in `.bun-version` and use rustup with the toolchain
in `rust-toolchain.toml`. Run builds from the repository root; the packaging task
installs the committed Bun workspace lock before building. Build on the operating
system and architecture you intend to ship.

- **macOS:** Install Xcode and select its developer directory. The selected
  toolchain must provide the macOS SDK, C/C++ compiler, and Metal tools. Verify
  `xcrun --sdk macosx --find clang`, `xcrun --sdk macosx --find metal`, and
  `xcrun --sdk macosx --find metallib` before the release build. Packaging also
  uses the system `ditto`, `otool`, `plutil`, and `codesign` tools.
- **Windows:** Use the MSVC Rust toolchain and install Visual Studio C++ Build
  Tools with the Windows SDK. Release builds compile GPUI's HLSL shaders with
  `fxc.exe`; GPUI finds it through `PATH` or the installed Windows SDK. Set
  `GPUI_FXC_PATH` to its full path when using a custom SDK layout. Packaging uses
  Windows PowerShell's `Compress-Archive` and `Expand-Archive` commands.
  Debug builds embed HLSL source and its includes in the executable, then compile
  them in memory using the Windows `d3dcompiler_47.dll` system component. They
  require neither the build-machine source directory nor extracted shader files;
  changing HLSL source requires rebuilding the executable.
- **Linux:** The candidate workflow uses Ubuntu 24.04. Install a C/C++ build
  toolchain and the native packages below; other distributions need their
  equivalent development packages. Packaging also requires `tar` and `ldd`.

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends \
  build-essential libfontconfig1-dev libfreetype6-dev libxcb1-dev \
  libxkbcommon-dev libxkbcommon-x11-dev pkg-config
```

A native window requires a desktop session with a working graphics stack. The
embedded-bundle check runs without a display and does not validate that stack.

## Build and verify

The website package uses the [approved project icon](../assets/branding/README.md).
macOS bundles include `solid-gpui.icns` and declare it in `CFBundleIconFile`.
Windows builds embed the ICO through the Windows SDK resource compiler.
Linux archives include hicolor PNG icons and a matching desktop-entry `Icon` key.
The shared navigation embeds its PNG in the JavaScript bundle, so it also works
outside the checkout. Regenerate brand assets only when the source artwork changes;
normal package builds use the checked-in files.

Run this command **on each destination operating system**:

```sh
bun run task website-package
```

The command builds workspace packages and native bindings, creates a release
executable for the build machine's Rust host triple, and writes the archive and
its SHA-256 checksum to `dist/website/`. Pass an output directory as the task's
optional argument to change that location: `bun run task website-package ./dist/candidate`.
It extracts the archive into a temporary directory, verifies every packaged file,
and runs its version and embedded-bundle checks outside the checkout with Bun
absent from `PATH`. The bundle check evaluates the shipped JavaScript in the real
QuickJS VM and consumes ordered Snapshots and Patches under one 15-second
deadline. It validates the tree and each native component identity, requires
actual content from both the SDK and website native modules, then shuts down
the runtime. The router's small loading Snapshot alone cannot pass this check.

| Build host        | Artifact                                       | Launch after extraction                                         |
| ----------------- | ---------------------------------------------- | --------------------------------------------------------------- |
| macOS             | `.zip` containing `Solid GPUI.app`             | Open the `.app`; optionally move it to `/Applications`          |
| Linux             | `.tar.gz` containing `usr/bin` and `usr/share` | Run `./usr/bin/solid-gpui-website` from the extracted directory |
| Windows with MSVC | `.zip` containing `solid-gpui-website.exe`     | Open the executable                                             |

Archives identify the version and exact Rust target, such as
`solid-gpui-website-0.2.0-aarch64-apple-darwin.zip`. They also include license
notices, an internal `SHA256SUMS`, and this guide. Unix archives include the
observed native library dependencies in `NATIVE-DEPENDENCIES.txt`. A checksum
detects corruption; publisher authentication requires signing.

`THIRD-PARTY-NOTICES.md` is the existing dependency inventory, not a complete
collection of third-party license texts. The package includes that inventory and
the project's MIT `LICENSE`; assembling and verifying the complete
redistribution notices remains release work before public distribution.

The `Website Packages` workflow builds native candidates on macOS ARM64,
Linux x86-64, and Windows x86-64 and uploads the verified archives. It is manually
triggered for release qualification; normal pull requests use the development
and cross-platform host checks. See [continuous integration](ci.md) for triggers,
caching, and the other candidate workflows. Artifacts are
unsigned candidates, not published releases. These checks do not establish
display, GPU, accessibility, input-method, menu, or notification correctness on
Windows/Linux. Qualify those behaviors on real desktop sessions before release.

## macOS

The `.app` contains its executable, application identifier, English metadata,
version, and license resources. The build defaults `MACOSX_DEPLOYMENT_TARGET` to
`13.0` and records the same value in `Info.plist`; this is a compiler target, not
a claim that the oldest selected macOS version has been tested. Set it explicitly
to your tested minimum when building. A native build produces one architecture;
an Intel release needs an Intel build and its own smoke run.

Packaging rejects non-system dynamic libraries and adds an ad hoc signature for
local evaluation. A publicly distributed application needs your Developer ID
signature and notarization. After extracting a candidate, sign the final `.app`,
submit a ZIP using `xcrun notarytool submit --keychain-profile <profile> --wait`,
staple the accepted ticket with `xcrun stapler staple`, and create the final ZIP
again. Verify its signature and Gatekeeper assessment, then regenerate all
checksums because signing/stapling changes package contents. Keep credentials in
the macOS Keychain or release secrets. Follow Apple's current
[notarization workflow](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)
and [bundle metadata reference](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html).

For example, before notarization, replace the candidate's ad hoc signature:

```sh
codesign --force --options runtime --timestamp \
  --sign "Developer ID Application: YOUR NAME (TEAMID)" "Solid GPUI.app"
codesign --verify --strict --verbose=2 "Solid GPUI.app"
ditto -c -k --keepParent "Solid GPUI.app" Website-notarization.zip
```

QuickJS is an interpreter; this package does not embed Bun's JSC/JIT runtime.
App icons, document associations, entitlements, sandboxing, updates, and installer
branding belong to the application being shipped and need their own validation.

## Linux

The portable archive relies on the destination's system libraries and GPU drivers;
it is not a universal static binary. Build on the oldest distribution baseline
you support and inspect `NATIVE-DEPENDENCIES.txt` for its actual requirements.
The candidate workflow uses Ubuntu 24.04. Test both Wayland and X11 with the
intended fonts, graphics stack, desktop portals, and display scaling.

Set `INSTALL_PREFIX` to the intended system installation prefix, then run from
the extracted package directory. The commands reject an unset or empty prefix:

```sh
sudo install -Dm755 usr/bin/solid-gpui-website "${INSTALL_PREFIX:?}/bin/solid-gpui-website"
sudo install -Dm644 usr/share/applications/io.github.cyenoch.solid-gpui-website.desktop \
  "${INSTALL_PREFIX:?}/share/applications/io.github.cyenoch.solid-gpui-website.desktop"
sudo mkdir -p "${INSTALL_PREFIX:?}/share/doc/solid-gpui-website"
sudo cp usr/share/doc/solid-gpui-website/* "${INSTALL_PREFIX:?}/share/doc/solid-gpui-website/"
```

The desktop entry uses an executable name resolved through the desktop session's
`PATH`, as specified by the [Desktop Entry standard](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html).
Uninstall by removing these application-specific paths. For distribution-managed
installation, use the included `usr/` tree as the package payload and declare
dependencies for the chosen baseline in a `.deb`/RPM recipe. AppImage and Flatpak
are separate deployment targets requiring runtime and portal qualification;
this repository does not present an untested wrapper as support for either.

## Windows

Build with the MSVC Rust target, Visual Studio C++ Build Tools, and the Windows SDK.
The portable executable uses the Windows GUI subsystem, so launching it does not
open a console window. It needs a working native graphics stack and any runtime
DLLs required by the selected native toolchain. A hosted runner has development
dependencies already installed; repeat qualification on a clean Windows machine.
Inspect executable imports with `dumpbin /DEPENDENTS`, and deploy the appropriate
[Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method?view=msvc-170)
if your build depends on it. Do not assume copying the executable proves that all
runtime dependencies are present.

For public distribution, sign the final executable with SignTool using your
publisher certificate and a timestamp service, then regenerate archive checksums.
Use Microsoft's [SignTool reference](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool)
for certificate and timestamp options. An MSI or MSIX installer adds installation,
uninstallation, identity, and update policy; select it for your application after
the portable executable is qualified. This repository supplies the portable
bundle and verification job, not an untested installer or automatic updater.

## Embedded Bun static applications

**Experimental.** The destination needs no sidecar JavaScript, no repository
checkout, and no separate Bun. Windows ARM64 release has the most coverage; signing,
broader installation policy, every other release target, and broader desktop and
physical-device coverage remain open. Repository CI does not yet build or verify a
static application package. A successful build alone does not prove that the result
is releasable. No target is currently claimed as supported or verified; the
[platform status table](#platform-status-and-current-evidence) below is the only
qualification evidence, and an unsupported triple fails early with the supported
matrix.

Three routes reach the same driver. An application uses the public CLI or the
library; this repository uses the script.

| Route | Use |
| --- | --- |
| `solid-gpui embedded package [flags]` | Public CLI shipped with `@solid-gpui/vite`. |
| `packageEmbeddedApplication({ sdkRoot, ... })` from `@solid-gpui/vite/embedded` | Library API for a build script; returns the packaging report. |
| `bun packages/solid-gpui-vite/src/embedded/command.ts [flags]` (npm script `embedded:package`) | This repository's checkout-local entry, same flags plus `--sdk-root`. |

A consumer passes `sdkRoot` explicitly: an SDK checkout that owns the pinned
Bun/Rust backend. That is the supported seam — no consumer imports repository-private
files, and the driver is never copied. The library accepts
`{ sdkRoot, entry, output, bun, target?, profile?, application?, assets?, workers?, baseExecutable?, cacheDir?, sourceCheckout?, ninja?, macosSdk?, deploymentTarget?, winsysroot?, prepareOnly? }`,
where `application` is the application-owned Cargo input:
`{ manifest, package, features?, main? }`. `manifest` is the application manifest
(package or workspace root), `package` is its package name, `features` adds that
package's features, and `main` is a Rust entry `include!`d into the generated bin
crate.

The driver prepares a patched checkout of the pinned Bun revision, builds Bun's
native graph against prebuilt WebKit, serializes the Vite-built application entry
with the pinned Bun serializer, and compiles the application's Rust crate graph
against that native graph. It writes the executable to `--output` and returns a
report: the output path and its SHA-256, the graph SHA-256, the Rust triple and
graph target, the profile, the native manifest when one was extracted, the typed
`entry` identity (`role: "application"`, source, identity), and every
`workers` identity. The run prints those identities as `Entry:` and `Worker:` lines
naming the virtual graph keys. Generated Rust exposes `BUN_EMBEDDED_ENTRY` and
`BUN_EMBEDDED_WORKERS`. The application reports its own exit code with
`completeEmbedded(code)` from `@solid-gpui/core/embedded`, guarded by
`supportsEmbeddedCompletion()`; the host reads it through
`EmbeddedBunAdapter::result()`, which carries the full `u32` even though the VM exit
status is only a byte.

### Static packaging inputs

A host-native build, with the serializer built from the pinned revision:

```sh
solid-gpui embedded package \
  --entry "$PWD/dist/app/index.js" \
  --bun "$PINNED_BUN" \
  --output "$PWD/dist/app/solid-gpui-embedded-app"
```

The same flags work through the checkout-local entry
`bun packages/solid-gpui-vite/src/embedded/command.ts` (npm script
`bun run embedded:package`, which adds `--sdk-root`) when the build runs inside the SDK
checkout. Cross-building a Windows x64 executable from macOS adds the target and the
same-revision target-platform Bun; set `WINDOWS_SYSROOT` to your prepared SDK/CRT
directory:

```sh
solid-gpui embedded package \
  --entry "$PWD/dist/app/index.js" \
  --bun "$PINNED_BUN" \
  --base-executable "$PINNED_BUN_WINDOWS_X64" \
  --target x86_64-pc-windows-msvc \
  --winsysroot "$WINDOWS_SYSROOT" \
  --output "$PWD/dist/app/solid-gpui-embedded-app.exe"
```

| Argument | Meaning |
| --- | --- |
| `--entry` (required) | Absolute path to the single ESM file the Vite production build emits; its basename names the graph entry. |
| `--bun` (required) | Absolute path to a Bun built from the revision pinned in `crates/solid-gpui-bun-sys/bun-build.json`. The driver reads `bun --revision` and refuses any other commit with no override, because the serialized payload carries no format version. Nothing here builds, downloads, or installs the serializer for you. |
| `--output` (required) | Executable to write; parent directories are created and its SHA-256 is printed. |
| `--target <triple>` | Rust triple; defaults to the host triple of the pinned `nightly-2026-07-20` toolchain. Supported: `aarch64-`/`x86_64-apple-darwin`, `x86_64-`/`aarch64-pc-windows-msvc`, and `x86_64-`/`aarch64-unknown-linux-gnu`/`-musl`. Anything else is rejected. |
| `--profile debug\|release` | Defaults to `release`; `debug` selects Bun's `debug-no-asan` native profile and Cargo's `dev` profile. |
| `--source <dir>`, `--cache <dir>` | Reuse an existing checkout of the pinned revision, and choose where the prepared checkout is kept (default `target/bun-static`). The cache name derives from the pin, the embedding patch, and the overlay sources, so a changed pin always rebuilds. |
| `--manifest <file>`, `--package <name>`, `--main <file>`, `--feature <name>` | Application-owned Cargo input: the manifest to build (package or workspace root), its package name, the Rust entry `include!`d into the generated bin crate (`--main` is required alongside `--manifest`; on its own it replaces the default entry), and repeatable `--feature` values added to that package. Without them the driver builds the default host entry. |
| `--assets <file>`, `--workers <entry>` | Repeatable. `--assets` values become `--asset` arguments and `--workers` values become extra serializer entry points. Declare every resource and Worker the application loads at run time; a `new Worker` or computed import is not discovered automatically. |
| `--base-executable <file>` | Same-revision Bun for the target platform, required whenever `--target` is not this host's platform and architecture; otherwise the serializer would need a base executable the pin does not cover and stops instead. |
| `--macos-sdk <dir>`, `--deployment-target <version>`, `--winsysroot <dir>` | Forwarded to Bun's build script as `--macos-sdk=`, `--osx-deployment-target=`, and `--winsysroot=`. |
| `--ninja <file>` | Ninja to use when it is not on `PATH`. |
| `--prepare-only` | Build only the native graph, print the `embed-native.json` path, and build no application. The only mode that works for Linux; the three required arguments must still be present although it uses none of them. |
| `--help` | Print the argument list. |

### Static packaging prerequisites

Run the driver with the Bun pinned in `.bun-version`; it re-executes Bun's own
`scripts/build.ts` through the running interpreter.
`crates/solid-gpui-bun-sys/bun-build.json` pins the Bun revision, the rustup
toolchain `nightly-2026-07-20`, and Ninja 1.13.0, so install that toolchain with its
`--target` standard library plus ninja on `PATH` or via `--ninja`, then run
`bun install --frozen-lockfile`. Compile the JSX/TSX inputs with Vite on a working
compiler host before invoking the packager, which consumes compiled JavaScript.

The native half needs LLVM 21.1.x (the pinned build accepts `>=21.1.0 <21.2.0` and
resolves it itself), cmake 3.24 or newer, `git`, Perl for LUT codegen, and the
platform SDK Bun resolves (`brew install llvm@21` and Xcode on macOS). The prepared
checkout installs its own JavaScript dependencies, so the first run needs network
access; caches never skip version or patch validation.

- **macOS:** the SDK resolves through `xcrun` and can be overridden with
  `--macos-sdk`; one older than the `13.0` minimum deployment target is refused, and
  `--deployment-target` records your choice. A non-darwin build host also needs an
  explicitly supplied macOS SDK; that cross-host path has not been qualified.
  Observed host combination (2026-09-17, this workstation): Bun's build system
  prefers Homebrew `/opt/homebrew/opt/llvm@21/bin/clang++` over `PATH`, and stock
  LLVM 21 rejects the `stack_protector_ignore` attribute the macOS 27 SDK places in
  `os/trace_base.h` with `-Werror,-Wunknown-attributes` (59 errors in
  `src/jsc/bindings/c-bindings.cpp`), so the native embed library cannot be built
  with that SDK on this machine. Overrides, no code change: the packager's existing
  `--macos-sdk /Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk
  --deployment-target 26.5`, and, for anything that configures Bun itself (for
  example `cargo build/test --features embedded-bun`, which runs
  `crates/solid-gpui-bun-sys/build.rs`), the `SOLID_GPUI_BUN_MACOS_SDK` and
  `SOLID_GPUI_BUN_DEPLOYMENT_TARGET` environment variables. Ninja 1.13.0 must be on
  `PATH` (this repository's `target/bun-tools/bin/ninja`). Treat this as a
  build-host toolchain requirement — pinned LLVM plus a matching SDK — with those
  explicit knobs; it makes no claim about operating-system support.
- **Windows:** build natively with MSVC, or cross-compile from macOS with LLVM's
  `clang-cl` and `lld-link` plus an xwin-style splat of the MSVC CRT/STL and Windows
  SDK passed through `--winsysroot`/`WINDOWS_SYSROOT` (pinned SDK `10.0.26100`, CRT
  `14.44.17.14`). Cross-compiled ARM64 debug builds additionally need the SDK's
  ARM64 debug CRT (`libcmtd.lib`, `libcpmtd.lib`, `libvcruntimed.lib`), which some
  xwin splats omit: obtain it under the SDK license and keep the original libraries
  instead of substituting release CRT ones. For a native build, initialize the real
  Visual Studio environment first (`VsDevCmd.bat`; ARM64 adds
  `-arch=arm64 -host_arch=arm64`) so `VSINSTALLDIR`, SDK tools, `INCLUDE`, and `LIB`
  are genuine, and extend this build's `PATH` with tools such as Git for Windows'
  Perl rather than changing machine-wide settings or PowerShell execution policy.
- **Linux:** only `--prepare-only` native preparation is implemented; see
  [platform status](#platform-status-and-current-evidence).

Four build-host details decide whether the result can run:

- **Runtime and profile.** The native half links the static MSVC runtime (`/MTd`
  debug, `/MT` release) and the Rust half adds `-Ctarget-feature=+crt-static`, so an
  application must be built with `--target` to keep build scripts and proc macros on
  the dynamic CRT.
- **ABI, not just architecture.** Match the pinned prebuilt WebKit's C++ ABI,
  architecture, and CRT mode; the verified splat is MSVC `14.44.17.14`, which
  `--winsysroot` also selects natively. MSVC `14.51.36231` headers with LLVM 21.1.8
  produced an incompatible ARM64 `std::partial_ordering` return convention and
  tripped a JSC clock-comparison assertion after an otherwise successful release
  link, so a newer installed Visual Studio proves nothing. Keep matching original
  headers and libraries together; never patch JSC layouts or disable the assertion.
- **Symbolic links.** Source archive extraction needs symbolic-link creation on the
  build host (for example zstd's test links). Providing it, such as by enabling
  Developer Mode, is an operator-controlled security decision, not a packager step;
  never ignore extraction failures or fabricate cache stamps. This is a build-host
  requirement, never an application runtime one.
- **Real shader compiler.** A release Windows build needs an executable `fxc.exe`
  (from the SDK, or the path in `GPUI_FXC_PATH`) emitting DXBC `vs_4_1`/`ps_4_1`;
  DXIL is not a substitute, a header/library sysroot is not a compiler, no compiler
  forwarding or precompiled-shader exchange protocol is provided, and there is no
  debug-shader fallback.

`--webkit=prebuilt` is always passed and a source WebKit build is refused at
configure time: the packager reuses the WebKit/JavaScriptCore archives published for
the pinned revision (per OS, architecture, and libc ABI) and never substitutes
another engine. The manifest records that mode so a consumer can reject a
source-WebKit product, and it must not link a separately compiled Bun Rust staticlib
— one Cargo graph, one `std`, Bun's allocator as the sole global allocator. A
generated root crate declaring its own global allocator fails the build.

### What the driver does

1. Resolves the target triple against the supported table; Linux without
   `--prepare-only` stops here, because ELF application graph transport is not
   implemented.
2. Prepares the pinned Bun checkout in the cache, applies
   `crates/solid-gpui-bun-sys/bun_embed.patch`, copies the embedding overlays, and
   re-verifies the pinned revision.
3. Configures Bun's native graph with `--mode=embed-native` and builds it with
   ninja against the prebuilt WebKit archives, compiling every C/C++ object but
   linking no Bun executable and no Rust staticlib.
4. Validates `embed-native.json` for target, revision, `prebuilt` mode, Cargo
   profile, absolute object and archive paths, and the absence of a `bun_rust`
   staticlib.
5. Serializes the entry and any `--workers` entries with `bun build --compile`, the
   target's Bun target name, and `--conditions=browser`, writing to a private
   temporary directory that adopts none of your `bunfig.toml`, `tsconfig.json`, or
   `package.json`, then discards the intermediate executable.
6. Validates the extracted graph: keys under the target's virtual root prefix,
   unique keys, an entry key equal to the packaged entry identity, a payload within
   the container's 32-bit section size, and a PE machine or Mach-O CPU type matching
   the requested architecture.
7. Generates the application crate and a build script replaying the manifest, then
   compiles with `--locked --target <triple>` using the manifest's profile, flags,
   and environment. Before writing `--output` it re-checks the image architecture
   and that the embedded graph still hashes to the serialized payload.

The packager points `solid-gpui-bun-sys` at that manifest, so the crate consumes its
objects, archives, link policy, and compiler settings instead of building its own
embedding library; direct `embedded-bun` library builds remain macOS-only. The graph
becomes an image section (`__BUN,__bun` at 16 KiB alignment on macOS, `.bun` on
Windows) that survives dead stripping and is read straight from the mapped image, so
nothing is unpacked at run time. The six-function C ABI
(`bun_embedded_create`, `bun_embedded_run`, `bun_embedded_result`,
`bun_embedded_wake`, `bun_embedded_terminate`, `bun_embedded_destroy`) keeps the
same contract: a packaged entry is
tagged apart from a disk path, so `start_packaged(entry)` never falls back to the
filesystem and a session that cannot start fails closed with a negative status.
`bun_embedded_result` is additive and reads the application's declared completion
(`{ present, code }`) without changing the run status.

### Default application entry

Without `--main`, the generated crate `include!`s
`crates/solid-gpui-bun-sys/static-application.rs`, which defines the process entry
point, keeps the crate's fixed identity (`solid-gpui-embedded-app 0.1.0`, printed by
`--version`), and adds one verification flag:

- `--check-bundle` starts two sessions in one process, requires each to deliver its
  initial Snapshot within 15 seconds, then shuts the runtime down and fails if any
  step does not complete.
- Any other argument combination is a usage error; with no arguments the application
  runs normally.

`--check-bundle` is a startup smoke check, not acceptance: it sends no input, so it
proves neither interaction nor that every asset, Worker, dynamic import, or service
is reachable. The Windows target uses the GUI subsystem and creates no console
window. `--main <file>` supplies your own `main` instead, `include!`d into the
generated `src/main.rs` after the graph include, so it can call the embedded runtime
directly and owns everything the default entry provides.

### Platform status and current evidence

| Target | Current state |
| --- | --- |
| macOS ARM64, debug | Built and run end to end: a headless two-session probe with counter input and clean shutdown, plus a real GPUI window that accepted interaction. Not standalone; see the limitations below. |
| Windows x64, debug | Cross-linked on macOS, then run alone in a Windows 11 ARM64 virtual machine with no Bun or Node. Two in-process sessions reproduced the same input and shutdown result. |
| macOS ARM64, release | Not built or qualified. |
| Windows x64, release | Not built or qualified. |
| Windows ARM64, debug | Cross-linked through the full packager; two graph/input/restart sessions passed natively in a Windows 11 ARM64 VM. No GUI or physical-device qualification. |
| Windows ARM64, release | Built natively with matching MSVC 14.44 headers/libraries, the pinned release JSC, and real SDK DXBC generation. Passed the full graph probe headlessly as an independent standard user and from a read-only installation. A separate user-controlled GUI run confirmed expected markers, counter interaction, resize, and normal closure. Broader desktop, physical-device, dependency-closure, and signing requirements remain open. |
| Linux | `--prepare-only` native preparation only. |

The probe is a real Vite-compiled Solid UI: Solid JSX, a dynamic `import()`, an
explicitly embedded `node:worker_threads` Worker, a resource read through
`Bun.embeddedFiles`, three counter inputs, and clean shutdown. The main VM and the
Worker both reject native extraction, default `fork()`/`cluster.fork()` reject the
host as an interpreter while external commands still run, and the pinned serializer
folds Vite's dynamic chunk into the entry, so this is not evidence of a separate
final dynamic-module record. These are fixture checks, not filesystem tracing or
application dependency closure.

The standard-user check used a distinct local account with only built-in `Users`
membership, medium integrity, and no Administrators SID, including deny-only.
The writable-installation control allowed file creation and non-truncating EXE
write-open; both operations returned access denied in the read-only installation
before and after execution. The read-only installation path included Unicode and
spaces. Both runs used empty `PATH` and private `TEMP`/`TMP`, passed two sessions,
exited zero, left no temporary files, and preserved the EXE hash. These headless checks do not qualify
GUI operation under standard-user or read-only-installation restrictions.

Do not read the table as broader support:

- **The macOS debug executable is not standalone.** It inherits a non-system UBSan
  runtime from the toolchain, so copying it to a machine lacking that runtime is not
  a supported distribution path and proves no dependency closure.
- **Windows results remain limited fixture evidence, including ARM64 release.** GUI
  correctness on other machines, physical-device coverage, full system dependency
  closure, signing, and public support stay unqualified, so public Windows support
  remains experimental. Release shaders follow the target platform and its
  debug-assertion setting rather than the build host; the ARM64 release build
  generated its full DXBC set, which establishes source-based shader preparation,
  not GPU or release acceptance.
- **Graphics and drivers remain the destination's.** The executable needs the same
  working graphics stack, fonts, and GPU drivers as any native GPUI application, and
  it imports normal OS libraries, such as `icuuc.dll` on Windows; Windows debug
  builds also import `d3dcompiler_47.dll`. Absence of Bun and VC-runtime DLLs says
  nothing about the rest.

### Runtime and packaging limits

- **No bundled native-library extraction.** The graph validator rejects JSC bytecode
  and module-info blobs (the runtime mutates bytecode in place), N-API addons, and
  any `.node`, `.dll`, `.so`, or `.dylib` entry, because those could only be served
  by writing them to disk at run time; integrate such dependencies statically
  instead. Extension checks alone cannot detect a renamed native asset. Embedded
  builds also disable the shared native extraction helper, including Worker VMs, and
  reject FFI graph paths — which is not a sandbox for external filesystem or FFI
  access.
- **No general fork/cluster JavaScript interpreter.** Embedded `fork()` rejects its
  default interpreter and an explicit `execPath` exactly equal to
  `process.execPath`, and `cluster.fork()` uses the same guard. The host is not a Bun
  CLI, and ordinary external process spawning is unchanged. Path aliases and general
  same-EXE spawning are not comprehensively intercepted, and an explicitly supplied
  external interpreter becomes an extra application dependency. Use declared Worker
  entries for in-image concurrency, resolving their paths against
  `import.meta.dirname` rather than the process working directory.
- **Resource closure is the application's responsibility.** Declare application
  Workers and resources with `--workers` and `--assets`. Arbitrary paths opened
  on the destination filesystem are not automatically embedded; include those
  application resources in the graph or provision them explicitly.
- **The packager emits a bare executable.** It attaches no application bundle, icon,
  metadata, entitlements, notices, or archive, so producing a macOS `.app`, an
  archive, or a checksum file remains your release step; follow the macOS bundle,
  signature, and notarization guidance above, and the Windows SignTool guidance.
- **This workflow does not change the website package.** The website keeps using the
  QuickJS pipeline documented above; nothing here replaces or extends that package,
  its archive layout, or its verification.

### Windows qualification gates

These gates are independent: a compiled candidate or a passing static audit does not
qualify the remaining rows. Keep the EXE hash, Windows build, architecture, account
evidence, command, output, and exit code with each result. The project does not
provision accounts, rewrite ACLs, weaken PowerShell policy, or install certificate
trust as part of packaging.

The Windows ARM64 release candidate has passed native-architecture,
release-build/basic-GUI, independent-standard-user headless, and read-only-installation
headless checks. Those results do not qualify broader GUI/device coverage,
dependency closure, or signing; see [platform status](#platform-status-and-current-evidence).

| Gate | Implementation | Evidence required |
| --- | --- | --- |
| Independent standard user | An operator-provisioned local non-administrator account, distinct from the build/test account. | Different user SID, reviewed membership, non-elevated token, and a passing application probe. A filtered administrator is not a standard user. |
| Read-only EXE directory | The verified candidate in an operator-provisioned read/execute-only location, including a Unicode/spaces path, launched in place with working files, logs and TEMP outside it. | Create-new in the directory and non-truncating write-open of the EXE both fail with access denied; startup/input/shutdown pass and the EXE hash is unchanged. |
| Dependency closure | Apply the static gate below, then inspect actual module loads and file accesses for the application's declared scenarios on each supported OS/architecture. | Normal/delay imports reviewed; runtime, graphics, fonts, configuration and resources separately recorded. Sampling is not exhaustive tracing. |
| Release | `--profile release`, the pinned release JSC product, and the Windows SDK DXBC compiler on a capable build host. | The release EXE itself passes the application probe, import audit, and user-controlled GUI scenarios; debug evidence is insufficient. |
| Signing | An explicitly selected publisher certificate and RFC 3161 endpoint after the final link, verified before publishing. | SignTool returns zero for sign and verify, the signer matches, a timestamp is present, and checksums describe the signed bytes. |
| Native ARM64 | The same pinned Bun base and application built for `aarch64-pc-windows-msvc`, never a reused x64 base; keep x64 and ARM64 bases, outputs, and evidence separate. | PE machine is ARM64, the probe passes on ARM64 Windows, and `IsWow64Process2` reports process machine `0` and native machine `0xAA64`. |

Run the account commands in the test account itself, never from an elevated helper:

```powershell
whoami /user /fo csv /nh
whoami /groups /fo csv /nh
Get-LocalGroupMember -SID 'S-1-5-32-544' | Select-Object Name, SID
```

Compare the SID with the separately recorded build account's and inspect both token
groups, including deny-only Administrators, plus account-database membership:
`.NET WindowsIdentity.Groups` alone can omit the administrator SID in a restricted
token. High/System integrity, service accounts, unresolved nested membership, and
unreadable membership data do not pass, and domain accounts need directory-side
evidence. For the read-only gate, use `File.Open` with `CreateNew`/`Write` on a
unique probe name in the EXE directory and `Open`/`Write` on the EXE itself: only
access denied establishes the expected failure, so close any successful handle and
remove only the probe you created. The file's read-only attribute and a non-writable
working directory are not substitutes for an installation-directory check.

Run a `--check-graph` candidate built from `fixtures/embedded-static-check.rs` with
an empty child `PATH`, private writable `TEMP`/`TMP`, and Bun/Node overrides
cleared: drain redirected stdout/stderr concurrently, use a 60-second outer
deadline, terminate only the owned child, and require both session lines, both
`PASS:` lines, and exit zero before re-hashing. Native assertions can still display
dialogs even with `CreateNoWindow`, so GUI testing stays a separate, user-controlled
activity; the default entry's `--check-bundle` has narrower coverage.

The host-side static gate uses LLVM rather than another PE dependency resolver:

```sh
bun scripts/check-embedded-dependencies.ts --exe target/application.exe --arch arm64 --readobj "$LLVM_READOBJ"
```

Set `LLVM_READOBJ` to the selected tool or omit `--readobj` to use `llvm-readobj`
from `PATH`. It emits JSON and compares normal and delay imports against the
reviewed system import set: `0` means this limited contract passed, `1` an
architecture or import violation, and `2` unusable input or tool output. New imports
need review; Bun/JSC DLLs and dynamic CRT imports are not allowed. The report binds
its result to a SHA-256 and refuses an image changed during inspection, but it does
not resolve API sets, follow OS DLL dependencies, authenticate loaded modules, trace
`LoadLibrary`, or prove graphics, font, or resource closure, and a DLL name alone
cannot establish a minimum Windows version. Full scenario evidence needs a
separately authorized Windows image/file trace covering startup, Workers, dynamic
imports, graphics, fonts, input, and shutdown; audit optional `.env`/`bunfig.toml`
reads too, because this workflow has not changed the pinned serializer's runtime
autoload defaults. Do not add a custom loader, ETW service, or API-set resolver to
manufacture a green result.

### Signing order

Embed the graph, compile, and link first; sign the final artifact; then compute the
published checksums. Signing before any later modification, or rewriting the image
or appending payload after signing, invalidates the evidence. On macOS the
deliverable is normally a signed and notarized `.app` containing this executable;
assemble and sign the final bundle, not just the contained binary. On Windows,
sign the final `.exe` with SignTool and a timestamp service, then verify, as described
in the Windows section above. Keep signing credentials out of the package and the
build cache. Publisher signing of the Embedded Bun candidates remains deferred,
so the gate stays open until a real certificate, timestamp, and successful verification
exist; supplying a certificate never authorizes silent approval of hardware or
provider prompts.

## Apply the pattern to another application

Keep transport selection in the runtime entry: the website's `main.native.tsx` chooses
`StdioTransport`, and `quickjs.tsx` chooses `EmbeddedTransport`. `mountWebsite`
accepts a transport factory and owns the shared UI lifecycle. Generate native
bindings before bundling; ordinary native development builds must not depend on
the bundle whose generation requires those bindings.

The `distribution` Cargo feature enables only the standalone website binary and
QuickJS. Its build script requires an absolute `SOLID_GPUI_WEBSITE_BUNDLE` path,
checks nonempty UTF-8 source, and tracks both the environment and source file for
rebuilds. The binary embeds the copied module with `include_bytes!`, starts
`QuickJsAdapter::from_source`, and passes it to `solid_gpui::run_application`.
That entry shares the normal host lifecycle and shutdown behavior without
interpreting development-host CLI arguments. It adds no temporary-file launcher
or second application process.

For Bun-led applications that need Bun services, use the
[Embedded Bun static packaging workflow](#embedded-bun-static-applications) when
its current limits are acceptable, or retain an explicit Bun runtime and package
it with the host. The `host-release` task creates a macOS process-host archive; it
does not turn that host into a self-contained application. Building the
`embedded-bun` Cargo feature directly (outside the static packager) still works
only on macOS and refuses other targets, so the static packager is what reaches
Windows and the other architectures — and it remains experimental. Do not
silently switch a service-dependent Bun application to QuickJS during packaging.
