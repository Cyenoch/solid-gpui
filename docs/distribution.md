# Distributing a native application

The Gallery is the reference application package. It compiles its Solid UI into
one ESM module, embeds that module in the Rust executable, and runs it with
QuickJS. Production packages omit inline source maps. End users need neither Bun
nor Node, a repository checkout, nor a JavaScript bundle beside the executable.
Rust owns native services and rendering;
the same Gallery composition also runs through Bun during development.

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

Run this command **on each destination operating system**:

```sh
bun run task gallery-package
```

The command builds workspace packages and native bindings, creates a release
executable for the build machine's Rust host triple, and writes the archive and
its SHA-256 checksum to `dist/gallery/`. Pass an output directory as the task's
optional argument to change that location: `bun run task gallery-package ./dist/candidate`.
It extracts the archive into a temporary directory, verifies every packaged file,
and runs its version and embedded-bundle checks outside the checkout with Bun
absent from `PATH`. The bundle check evaluates the shipped JavaScript in the real
QuickJS VM and consumes ordered Snapshots and Patches under one 15-second
deadline. It validates the tree and each native component identity, requires
actual content from both the SDK and Gallery native modules, then shuts down
the runtime. The router's small loading Snapshot alone cannot pass this check.

| Build host        | Artifact                                       | Launch after extraction                                         |
| ----------------- | ---------------------------------------------- | --------------------------------------------------------------- |
| macOS             | `.zip` containing `Solid GPUI Gallery.app`     | Open the `.app`; optionally move it to `/Applications`          |
| Linux             | `.tar.gz` containing `usr/bin` and `usr/share` | Run `./usr/bin/solid-gpui-gallery` from the extracted directory |
| Windows with MSVC | `.zip` containing `solid-gpui-gallery.exe`     | Open the executable                                             |

Archives identify the version and exact Rust target, such as
`solid-gpui-gallery-0.2.0-aarch64-apple-darwin.zip`. They also include license
notices, an internal `SHA256SUMS`, and this guide. Unix archives include the
observed native library dependencies in `NATIVE-DEPENDENCIES.txt`. A checksum
detects corruption; publisher authentication requires signing.

`THIRD-PARTY-NOTICES.md` is the existing dependency inventory, not a complete
collection of third-party license texts. The package includes that inventory and
the project's Apache-2.0 `LICENSE`; assembling and verifying the complete
redistribution notices remains release work before public distribution.

The `Gallery Packages` workflow builds native candidates on macOS ARM64,
Linux x86-64, and Windows x86-64 and uploads the verified archives. It is manually
triggered and runs for pull requests changing packaging inputs. Artifacts are
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
  --sign "Developer ID Application: YOUR NAME (TEAMID)" "Solid GPUI Gallery.app"
codesign --verify --strict --verbose=2 "Solid GPUI Gallery.app"
ditto -c -k --keepParent "Solid GPUI Gallery.app" Gallery-notarization.zip
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

To install an extracted package under `/usr/local`, run from its directory:

```sh
sudo install -Dm755 usr/bin/solid-gpui-gallery /usr/local/bin/solid-gpui-gallery
sudo install -Dm644 usr/share/applications/io.github.cyenoch.solid-gpui-gallery.desktop \
  /usr/local/share/applications/io.github.cyenoch.solid-gpui-gallery.desktop
sudo mkdir -p /usr/local/share/doc/solid-gpui-gallery
sudo cp usr/share/doc/solid-gpui-gallery/* /usr/local/share/doc/solid-gpui-gallery/
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

## Apply the pattern to another application

Keep transport selection in the runtime entry: Gallery's `main.tsx` chooses
`StdioTransport`, and `quickjs.tsx` chooses `EmbeddedTransport`. `mountGallery`
accepts a transport factory and owns the shared UI lifecycle. Generate native
bindings before bundling; ordinary native development builds must not depend on
the bundle whose generation requires those bindings.

The `distribution` Cargo feature enables only the standalone Gallery binary and
QuickJS. Its build script requires an absolute `SOLID_GPUI_GALLERY_BUNDLE` path,
checks nonempty UTF-8 source, and tracks both the environment and source file for
rebuilds. The binary embeds the copied module with `include_bytes!`, starts
`QuickJsAdapter::from_source`, and passes it to `solid_gpui::run_application`.
That entry shares the normal host lifecycle and shutdown behavior without
interpreting development-host CLI arguments. It adds no temporary-file launcher
or second application process.

For Bun-led applications that need Bun services, retain an explicit Bun runtime
and package it with the host. The existing `host-release` task creates a macOS
process-host archive; it does not turn that host into a self-contained application.
Embedded Bun remains a separate macOS-specific build path. Do not silently switch
a service-dependent Bun application to QuickJS during packaging.
