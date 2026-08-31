# Third-Party Notices

This inventory accompanies the host release archive `react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz` and the companion Bun packages from this checkout.
It records each resolved dependency's name, version, SPDX license identifier, and source provenance for the generated release artifacts.
The inventory is generated from the resolved Cargo graph and `bun pm licenses --all` output; it is not a substitute for the license texts.
The archive embeds the project-owned Apache-2.0 text as `LICENSE`; that same text covers the independently authored local `ztracing` stub. Each npm package carries its own `LICENSE` in its tarball.
Full third-party license texts are intentionally not copied into this inventory; they remain available from the referenced registry or git source. This keeps the artifact an inventory rather than a 670-crate license-text bundle.

**Generated:** 2026-08-31
**Generation command:** `bash scripts/third-party-notices.sh`

The host archive keeps this single inventory next to `LICENSE`. The npm tarballs intentionally remain lean and carry only their own package `LICENSE`; consumers of the dev package receive the JavaScript dependency inventory through the repository or release archive rather than duplicating it in every npm tarball.
License groups below repeat a package when its declared expression contains multiple SPDX identifiers. Totals count package records within that group.

## Rust dependencies

The current Cargo inventory contains 659 third-party packages and 1152 package-license records; local workspace records are listed separately below.

### 0BSD

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| adler2 | 2.0.1 | 0BSD | registry (crates.io) |

**Total 0BSD: 1 package records.**

### Apache-2.0

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| accesskit | 0.24.1 | Apache-2.0 | registry (crates.io) |
| accesskit_atspi_common | 0.18.1 | Apache-2.0 | registry (crates.io) |
| accesskit_consumer | 0.36.0 | Apache-2.0 | registry (crates.io) |
| accesskit_consumer | 0.37.0 | Apache-2.0 | registry (crates.io) |
| accesskit_consumer | 0.38.0 | Apache-2.0 | registry (crates.io) |
| accesskit_macos | 0.26.3 | Apache-2.0 | registry (crates.io) |
| accesskit_unix | 0.21.1 | Apache-2.0 | registry (crates.io) |
| accesskit_windows | 0.33.1 | Apache-2.0 | registry (crates.io) |
| addr2line | 0.25.1 | Apache-2.0 | registry (crates.io) |
| adler2 | 2.0.1 | Apache-2.0 | registry (crates.io) |
| aes | 0.8.4 | Apache-2.0 | registry (crates.io) |
| ahash | 0.8.12 | Apache-2.0 | registry (crates.io) |
| aligned | 0.4.3 | Apache-2.0 | registry (crates.io) |
| allocator-api2 | 0.2.21 | Apache-2.0 | registry (crates.io) |
| android_system_properties | 0.1.6 | Apache-2.0 | registry (crates.io) |
| anyhow | 1.0.104 | Apache-2.0 | registry (crates.io) |
| arbitrary | 1.4.2 | Apache-2.0 | registry (crates.io) |
| arrayvec | 0.7.8 | Apache-2.0 | registry (crates.io) |
| as-raw-xcb-connection | 1.0.1 | Apache-2.0 | registry (crates.io) |
| as-slice | 0.2.1 | Apache-2.0 | registry (crates.io) |
| ash | 0.38.0+1.3.281 | Apache-2.0 | registry (crates.io) |
| async-broadcast | 0.7.2 | Apache-2.0 | registry (crates.io) |
| async-channel | 2.5.0 | Apache-2.0 | registry (crates.io) |
| async-compression | 0.4.43 | Apache-2.0 | registry (crates.io) |
| async-executor | 1.14.0 | Apache-2.0 | registry (crates.io) |
| async-fs | 2.2.0 | Apache-2.0 | registry (crates.io) |
| async-io | 2.6.0 | Apache-2.0 | registry (crates.io) |
| async-lock | 3.4.2 | Apache-2.0 | registry (crates.io) |
| async-net | 2.0.0 | Apache-2.0 | registry (crates.io) |
| async-process | 2.5.0 | Apache-2.0 | registry (crates.io) |
| async-recursion | 1.1.1 | Apache-2.0 | registry (crates.io) |
| async-signal | 0.2.14 | Apache-2.0 | registry (crates.io) |
| async-task | 4.7.1 | Apache-2.0 | registry (crates.io) |
| async-trait | 0.1.92 | Apache-2.0 | registry (crates.io) |
| atomic | 0.5.3 | Apache-2.0 | registry (crates.io) |
| atomic-waker | 1.1.2 | Apache-2.0 | registry (crates.io) |
| atspi | 0.29.0 | Apache-2.0 | registry (crates.io) |
| atspi-common | 0.13.0 | Apache-2.0 | registry (crates.io) |
| atspi-proxies | 0.13.0 | Apache-2.0 | registry (crates.io) |
| autocfg | 1.5.1 | Apache-2.0 | registry (crates.io) |
| backtrace | 0.3.76 | Apache-2.0 | registry (crates.io) |
| base64 | 0.22.1 | Apache-2.0 | registry (crates.io) |
| bit-set | 0.8.0 | Apache-2.0 | registry (crates.io) |
| bit-set | 0.9.1 | Apache-2.0 | registry (crates.io) |
| bit-vec | 0.8.0 | Apache-2.0 | registry (crates.io) |
| bit-vec | 0.9.1 | Apache-2.0 | registry (crates.io) |
| bit_field | 0.10.3 | Apache-2.0 | registry (crates.io) |
| bitflags | 1.3.2 | Apache-2.0 | registry (crates.io) |
| bitflags | 2.13.1 | Apache-2.0 | registry (crates.io) |
| bitstream-io | 4.10.0 | Apache-2.0 | registry (crates.io) |
| block-buffer | 0.10.4 | Apache-2.0 | registry (crates.io) |
| block-padding | 0.3.3 | Apache-2.0 | registry (crates.io) |
| blocking | 1.7.0 | Apache-2.0 | registry (crates.io) |
| bumpalo | 3.20.3 | Apache-2.0 | registry (crates.io) |
| bytemuck | 1.25.2 | Apache-2.0 | registry (crates.io) |
| bytemuck_derive | 1.12.0 | Apache-2.0 | registry (crates.io) |
| bzip2 | 0.6.1 | Apache-2.0 | registry (crates.io) |
| cbc | 0.1.2 | Apache-2.0 | registry (crates.io) |
| cc | 1.4.4 | Apache-2.0 | registry (crates.io) |
| cexpr | 0.6.0 | Apache-2.0 | registry (crates.io) |
| cfg-if | 1.0.4 | Apache-2.0 | registry (crates.io) |
| cgl | 0.3.2 | Apache-2.0 | registry (crates.io) |
| chrono | 0.4.45 | Apache-2.0 | registry (crates.io) |
| cipher | 0.4.4 | Apache-2.0 | registry (crates.io) |
| clang-sys | 1.9.1 | Apache-2.0 | registry (crates.io) |
| cocoa | 0.26.0 | Apache-2.0 | registry (crates.io) |
| cocoa-foundation | 0.2.1 | Apache-2.0 | registry (crates.io) |
| codespan-reporting | 0.13.1 | Apache-2.0 | registry (crates.io) |
| collections | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| compression-codecs | 0.4.38 | Apache-2.0 | registry (crates.io) |
| compression-core | 0.4.32 | Apache-2.0 | registry (crates.io) |
| concurrent-queue | 2.5.0 | Apache-2.0 | registry (crates.io) |
| console_error_panic_hook | 0.1.7 | Apache-2.0 | registry (crates.io) |
| const-random | 0.1.18 | Apache-2.0 | registry (crates.io) |
| const-random-macro | 0.1.16 | Apache-2.0 | registry (crates.io) |
| core-foundation | 0.10.1 | Apache-2.0 | registry (crates.io) |
| core-foundation-sys | 0.8.7 | Apache-2.0 | registry (crates.io) |
| core-graphics | 0.24.0 | Apache-2.0 | registry (crates.io) |
| core-graphics-types | 0.2.0 | Apache-2.0 | registry (crates.io) |
| core-graphics2 | 0.5.2 | Apache-2.0 | registry (crates.io) |
| core-text | 21.0.0 | Apache-2.0 | registry (crates.io) |
| core-video | 0.5.2 | Apache-2.0 | registry (crates.io) |
| cosmic-text | 0.19.0 | Apache-2.0 | registry (crates.io) |
| cpufeatures | 0.2.17 | Apache-2.0 | registry (crates.io) |
| crc32fast | 1.5.1 | Apache-2.0 | registry (crates.io) |
| crossbeam-deque | 0.8.7 | Apache-2.0 | registry (crates.io) |
| crossbeam-epoch | 0.9.20 | Apache-2.0 | registry (crates.io) |
| crossbeam-queue | 0.3.13 | Apache-2.0 | registry (crates.io) |
| crossbeam-utils | 0.8.22 | Apache-2.0 | registry (crates.io) |
| crypto-common | 0.1.7 | Apache-2.0 | registry (crates.io) |
| ctor | 1.0.13 | Apache-2.0 | registry (crates.io) |
| data-url | 0.3.2 | Apache-2.0 | registry (crates.io) |
| deranged | 0.5.8 | Apache-2.0 | registry (crates.io) |
| derive_refineable | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| digest | 0.10.7 | Apache-2.0 | registry (crates.io) |
| dirs | 6.0.0 | Apache-2.0 | registry (crates.io) |
| dirs-sys | 0.5.0 | Apache-2.0 | registry (crates.io) |
| dispatch2 | 0.3.1 | Apache-2.0 | registry (crates.io) |
| displaydoc | 0.2.7 | Apache-2.0 | registry (crates.io) |
| document-features | 0.2.12 | Apache-2.0 | registry (crates.io) |
| downcast-rs | 1.2.1 | Apache-2.0 | registry (crates.io) |
| dunce | 1.0.5 | Apache-2.0 | registry (crates.io) |
| dyn-clone | 1.0.20 | Apache-2.0 | registry (crates.io) |
| either | 1.18.0 | Apache-2.0 | registry (crates.io) |
| encoding_rs | 0.8.35 | Apache-2.0 | registry (crates.io) |
| enumflags2 | 0.7.12 | Apache-2.0 | registry (crates.io) |
| enumflags2_derive | 0.7.12 | Apache-2.0 | registry (crates.io) |
| enumn | 0.1.14 | Apache-2.0 | registry (crates.io) |
| equivalent | 1.0.2 | Apache-2.0 | registry (crates.io) |
| erased-serde | 0.4.10 | Apache-2.0 | registry (crates.io) |
| errno | 0.3.14 | Apache-2.0 | registry (crates.io) |
| etagere | 0.2.15 | Apache-2.0 | registry (crates.io) |
| euclid | 0.22.14 | Apache-2.0 | registry (crates.io) |
| event-listener | 5.4.2 | Apache-2.0 | registry (crates.io) |
| event-listener-strategy | 0.5.4 | Apache-2.0 | registry (crates.io) |
| fastrand | 2.5.0 | Apache-2.0 | registry (crates.io) |
| fdeflate | 0.3.7 | Apache-2.0 | registry (crates.io) |
| find-msvc-tools | 0.1.11 | Apache-2.0 | registry (crates.io) |
| fixedbitset | 0.5.7 | Apache-2.0 | registry (crates.io) |
| flate2 | 1.1.9 | Apache-2.0 | registry (crates.io) |
| float-ord | 0.3.2 | Apache-2.0 | registry (crates.io) |
| flume | 0.12.0 | Apache-2.0 | registry (crates.io) |
| fnv | 1.0.7 | Apache-2.0 | registry (crates.io) |
| font-types | 0.11.3 | Apache-2.0 | registry (crates.io) |
| font-types | 0.12.4 | Apache-2.0 | registry (crates.io) |
| foreign-types | 0.5.0 | Apache-2.0 | registry (crates.io) |
| foreign-types-macros | 0.2.4 | Apache-2.0 | registry (crates.io) |
| foreign-types-shared | 0.3.1 | Apache-2.0 | registry (crates.io) |
| form_urlencoded | 1.2.2 | Apache-2.0 | registry (crates.io) |
| futures | 0.3.34 | Apache-2.0 | registry (crates.io) |
| futures-channel | 0.3.34 | Apache-2.0 | registry (crates.io) |
| futures-concurrency | 7.7.1 | Apache-2.0 | registry (crates.io) |
| futures-core | 0.3.34 | Apache-2.0 | registry (crates.io) |
| futures-executor | 0.3.34 | Apache-2.0 | registry (crates.io) |
| futures-io | 0.3.34 | Apache-2.0 | registry (crates.io) |
| futures-lite | 2.6.1 | Apache-2.0 | registry (crates.io) |
| futures-macro | 0.3.34 | Apache-2.0 | registry (crates.io) |
| futures-sink | 0.3.34 | Apache-2.0 | registry (crates.io) |
| futures-task | 0.3.34 | Apache-2.0 | registry (crates.io) |
| futures-util | 0.3.34 | Apache-2.0 | registry (crates.io) |
| gethostname | 1.1.0 | Apache-2.0 | registry (crates.io) |
| getrandom | 0.2.17 | Apache-2.0 | registry (crates.io) |
| getrandom | 0.3.4 | Apache-2.0 | registry (crates.io) |
| getrandom | 0.4.3 | Apache-2.0 | registry (crates.io) |
| gif | 0.14.2 | Apache-2.0 | registry (crates.io) |
| gimli | 0.32.3 | Apache-2.0 | registry (crates.io) |
| gl_generator | 0.14.0 | Apache-2.0 | registry (crates.io) |
| glob | 0.3.4 | Apache-2.0 | registry (crates.io) |
| glow | 0.17.0 | Apache-2.0 | registry (crates.io) |
| glutin_wgl_sys | 0.6.1 | Apache-2.0 | registry (crates.io) |
| gpu-allocator | 0.28.0 | Apache-2.0 | registry (crates.io) |
| gpu-descriptor | 0.3.2 | Apache-2.0 | registry (crates.io) |
| gpu-descriptor-types | 0.2.0 | Apache-2.0 | registry (crates.io) |
| gpui | 0.2.2 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_apple | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_linux | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_macos | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_macros | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_platform | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_shared_string | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_util | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_web | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_wgpu | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| gpui_windows | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| half | 2.7.1 | Apache-2.0 | registry (crates.io) |
| hash32 | 0.3.1 | Apache-2.0 | registry (crates.io) |
| hashbrown | 0.14.5 | Apache-2.0 | registry (crates.io) |
| hashbrown | 0.15.5 | Apache-2.0 | registry (crates.io) |
| hashbrown | 0.16.1 | Apache-2.0 | registry (crates.io) |
| hashbrown | 0.17.1 | Apache-2.0 | registry (crates.io) |
| heapless | 0.9.3 | Apache-2.0 | registry (crates.io) |
| heck | 0.4.1 | Apache-2.0 | registry (crates.io) |
| heck | 0.5.0 | Apache-2.0 | registry (crates.io) |
| hermit-abi | 0.5.2 | Apache-2.0 | registry (crates.io) |
| hex | 0.4.3 | Apache-2.0 | registry (crates.io) |
| hkdf | 0.12.4 | Apache-2.0 | registry (crates.io) |
| hmac | 0.12.1 | Apache-2.0 | registry (crates.io) |
| home | 0.5.12 | Apache-2.0 | registry (crates.io) |
| http | 1.5.0 | Apache-2.0 | registry (crates.io) |
| http_client | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| iana-time-zone | 0.1.65 | Apache-2.0 | registry (crates.io) |
| iana-time-zone-haiku | 0.1.2 | Apache-2.0 | registry (crates.io) |
| idna | 1.1.0 | Apache-2.0 | registry (crates.io) |
| idna_adapter | 1.2.2 | Apache-2.0 | registry (crates.io) |
| image | 0.25.10 | Apache-2.0 | registry (crates.io) |
| image-webp | 0.2.4 | Apache-2.0 | registry (crates.io) |
| imgref | 1.12.2 | Apache-2.0 | registry (crates.io) |
| indexmap | 2.14.0 | Apache-2.0 | registry (crates.io) |
| inout | 0.1.4 | Apache-2.0 | registry (crates.io) |
| inventory | 0.3.24 | Apache-2.0 | registry (crates.io) |
| io-surface | 0.16.1 | Apache-2.0 | registry (crates.io) |
| itertools | 0.13.0 | Apache-2.0 | registry (crates.io) |
| itertools | 0.14.0 | Apache-2.0 | registry (crates.io) |
| itoa | 1.0.18 | Apache-2.0 | registry (crates.io) |
| jni-sys | 0.3.1 | Apache-2.0 | registry (crates.io) |
| jni-sys | 0.4.1 | Apache-2.0 | registry (crates.io) |
| jni-sys-macros | 0.4.1 | Apache-2.0 | registry (crates.io) |
| jobserver | 0.1.35 | Apache-2.0 | registry (crates.io) |
| js-sys | 0.3.104 | Apache-2.0 | registry (crates.io) |
| khronos-egl | 6.0.0 | Apache-2.0 | registry (crates.io) |
| khronos_api | 3.1.0 | Apache-2.0 | registry (crates.io) |
| kurbo | 0.13.1 | Apache-2.0 | registry (crates.io) |
| lazy_static | 1.5.0 | Apache-2.0 | registry (crates.io) |
| leak | 0.1.2 | Apache-2.0 | registry (crates.io) |
| leaky-cow | 0.1.1 | Apache-2.0 | registry (crates.io) |
| libc | 0.2.189 | Apache-2.0 | registry (crates.io) |
| libfuzzer-sys | 0.4.13 | Apache-2.0 | registry (crates.io) |
| linebender_resource_handle | 0.1.1 | Apache-2.0 | registry (crates.io) |
| link-section | 0.19.3 | Apache-2.0 | registry (crates.io) |
| linktime-proc-macro | 0.2.3 | Apache-2.0 | registry (crates.io) |
| linux-raw-sys | 0.12.1 | Apache-2.0 | registry (crates.io) |
| linux-raw-sys | 0.4.15 | Apache-2.0 | registry (crates.io) |
| litrs | 1.0.0 | Apache-2.0 | registry (crates.io) |
| lock_api | 0.4.14 | Apache-2.0 | registry (crates.io) |
| log | 0.4.34 | Apache-2.0 | registry (crates.io) |
| lyon | 1.0.19 | Apache-2.0 | registry (crates.io) |
| lyon_algorithms | 1.0.20 | Apache-2.0 | registry (crates.io) |
| lyon_geom | 1.0.19 | Apache-2.0 | registry (crates.io) |
| lyon_path | 1.0.19 | Apache-2.0 | registry (crates.io) |
| lyon_tessellation | 1.0.20 | Apache-2.0 | registry (crates.io) |
| mac-notification-sys | 0.6.15 | Apache-2.0 | registry (crates.io) |
| mach2 | 0.5.0 | Apache-2.0 | registry (crates.io) |
| md-5 | 0.10.6 | Apache-2.0 | registry (crates.io) |
| media | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| memmap2 | 0.9.11 | Apache-2.0 | registry (crates.io) |
| metal | 0.33.0 | Apache-2.0 | registry (crates.io) |
| minimal-lexical | 0.2.1 | Apache-2.0 | registry (crates.io) |
| miniz_oxide | 0.8.9 | Apache-2.0 | registry (crates.io) |
| moxcms | 0.8.1 | Apache-2.0 | registry (crates.io) |
| naga | 29.0.4 | Apache-2.0 | registry (crates.io) |
| ndk-sys | 0.6.0+11769913 | Apache-2.0 | registry (crates.io) |
| no_std_io2 | 0.9.4 | Apache-2.0 | registry (crates.io) |
| notify-rust | 4.18.0 | Apache-2.0 | registry (crates.io) |
| num | 0.4.3 | Apache-2.0 | registry (crates.io) |
| num-bigint | 0.4.8 | Apache-2.0 | registry (crates.io) |
| num-bigint-dig | 0.9.1 | Apache-2.0 | registry (crates.io) |
| num-complex | 0.4.6 | Apache-2.0 | registry (crates.io) |
| num-conv | 0.2.2 | Apache-2.0 | registry (crates.io) |
| num-derive | 0.4.2 | Apache-2.0 | registry (crates.io) |
| num-integer | 0.1.47 | Apache-2.0 | registry (crates.io) |
| num-iter | 0.1.46 | Apache-2.0 | registry (crates.io) |
| num-rational | 0.4.2 | Apache-2.0 | registry (crates.io) |
| num-traits | 0.2.19 | Apache-2.0 | registry (crates.io) |
| num_cpus | 1.17.0 | Apache-2.0 | registry (crates.io) |
| objc2-app-kit | 0.3.2 | Apache-2.0 | registry (crates.io) |
| objc2-core-foundation | 0.3.2 | Apache-2.0 | registry (crates.io) |
| objc2-core-location | 0.3.2 | Apache-2.0 | registry (crates.io) |
| objc2-metal | 0.3.2 | Apache-2.0 | registry (crates.io) |
| objc2-quartz-core | 0.3.2 | Apache-2.0 | registry (crates.io) |
| objc2-user-notifications | 0.3.2 | Apache-2.0 | registry (crates.io) |
| object | 0.37.3 | Apache-2.0 | registry (crates.io) |
| object | 0.39.1 | Apache-2.0 | registry (crates.io) |
| once_cell | 1.21.4 | Apache-2.0 | registry (crates.io) |
| ordered-stream | 0.2.0 | Apache-2.0 | registry (crates.io) |
| parking | 2.2.1 | Apache-2.0 | registry (crates.io) |
| parking_lot | 0.12.5 | Apache-2.0 | registry (crates.io) |
| parking_lot_core | 0.9.12 | Apache-2.0 | registry (crates.io) |
| paste | 1.0.15 | Apache-2.0 | registry (crates.io) |
| pastey | 0.1.1 | Apache-2.0 | registry (crates.io) |
| pathfinder_geometry | 0.5.1 | Apache-2.0 | registry (crates.io) |
| pathfinder_simd | 0.5.6 | Apache-2.0 | registry (crates.io) |
| pbkdf2 | 0.12.2 | Apache-2.0 | registry (crates.io) |
| percent-encoding | 2.3.2 | Apache-2.0 | registry (crates.io) |
| perf | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| pin-project | 1.1.13 | Apache-2.0 | registry (crates.io) |
| pin-project-internal | 1.1.13 | Apache-2.0 | registry (crates.io) |
| pin-project-lite | 0.2.17 | Apache-2.0 | registry (crates.io) |
| piper | 0.2.5 | Apache-2.0 | registry (crates.io) |
| pkg-config | 0.3.34 | Apache-2.0 | registry (crates.io) |
| png | 0.17.16 | Apache-2.0 | registry (crates.io) |
| png | 0.18.1 | Apache-2.0 | registry (crates.io) |
| polling | 3.11.0 | Apache-2.0 | registry (crates.io) |
| pollster | 0.2.5 | Apache-2.0 | registry (crates.io) |
| pollster | 0.4.0 | Apache-2.0 | registry (crates.io) |
| polycool | 0.4.0 | Apache-2.0 | registry (crates.io) |
| portable-atomic | 1.15.0 | Apache-2.0 | registry (crates.io) |
| portable-atomic-util | 0.2.7 | Apache-2.0 | registry (crates.io) |
| powerfmt | 0.2.0 | Apache-2.0 | registry (crates.io) |
| ppv-lite86 | 0.2.21 | Apache-2.0 | registry (crates.io) |
| presser | 0.3.1 | Apache-2.0 | registry (crates.io) |
| prettyplease | 0.2.37 | Apache-2.0 | registry (crates.io) |
| proc-macro-crate | 3.5.0 | Apache-2.0 | registry (crates.io) |
| proc-macro2 | 1.0.107 | Apache-2.0 | registry (crates.io) |
| profiling | 1.0.18 | Apache-2.0 | registry (crates.io) |
| profiling-procmacros | 1.0.18 | Apache-2.0 | registry (crates.io) |
| proptest | 1.10.0 | Apache-2.0 | git (git+https://github.com/proptest-rs/proptest?rev=3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b#3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b) |
| proptest-macro | 0.5.0 | Apache-2.0 | git (git+https://github.com/proptest-rs/proptest?rev=3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b#3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b) |
| psm | 0.1.32 | Apache-2.0 | registry (crates.io) |
| pxfm | 0.1.30 | Apache-2.0 | registry (crates.io) |
| qoi | 0.4.1 | Apache-2.0 | registry (crates.io) |
| quick-error | 1.2.3 | Apache-2.0 | registry (crates.io) |
| quick-error | 2.0.1 | Apache-2.0 | registry (crates.io) |
| quote | 1.0.47 | Apache-2.0 | registry (crates.io) |
| r-efi | 5.3.0 | Apache-2.0 | registry (crates.io) |
| r-efi | 6.0.0 | Apache-2.0 | registry (crates.io) |
| rand | 0.9.5 | Apache-2.0 | registry (crates.io) |
| rand_chacha | 0.9.0 | Apache-2.0 | registry (crates.io) |
| rand_core | 0.9.5 | Apache-2.0 | registry (crates.io) |
| rand_xorshift | 0.4.0 | Apache-2.0 | registry (crates.io) |
| range-alloc | 0.1.5 | Apache-2.0 | registry (crates.io) |
| rangemap | 1.8.0 | Apache-2.0 | registry (crates.io) |
| raw-window-handle | 0.6.2 | Apache-2.0 | registry (crates.io) |
| raw-window-metal | 1.1.0 | Apache-2.0 | registry (crates.io) |
| rayon | 1.12.0 | Apache-2.0 | registry (crates.io) |
| rayon-core | 1.13.0 | Apache-2.0 | registry (crates.io) |
| read-fonts | 0.37.0 | Apache-2.0 | registry (crates.io) |
| read-fonts | 0.41.0 | Apache-2.0 | registry (crates.io) |
| ref-cast | 1.0.27 | Apache-2.0 | registry (crates.io) |
| ref-cast-impl | 1.0.27 | Apache-2.0 | registry (crates.io) |
| refineable | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| regex | 1.13.1 | Apache-2.0 | registry (crates.io) |
| regex-automata | 0.4.18 | Apache-2.0 | registry (crates.io) |
| regex-syntax | 0.8.11 | Apache-2.0 | registry (crates.io) |
| renderdoc-sys | 1.1.0 | Apache-2.0 | registry (crates.io) |
| resvg | 0.46.0 | Apache-2.0 | registry (crates.io) |
| roxmltree | 0.20.0 | Apache-2.0 | registry (crates.io) |
| roxmltree | 0.21.1 | Apache-2.0 | registry (crates.io) |
| rustc-demangle | 0.1.28 | Apache-2.0 | registry (crates.io) |
| rustc-hash | 1.1.0 | Apache-2.0 | registry (crates.io) |
| rustc-hash | 2.1.3 | Apache-2.0 | registry (crates.io) |
| rustc_version | 0.4.1 | Apache-2.0 | registry (crates.io) |
| rustix | 0.38.44 | Apache-2.0 | registry (crates.io) |
| rustix | 1.1.4 | Apache-2.0 | registry (crates.io) |
| rustversion | 1.0.23 | Apache-2.0 | registry (crates.io) |
| rusty-fork | 0.3.1 | Apache-2.0 | registry (crates.io) |
| ryu | 1.0.23 | Apache-2.0 | registry (crates.io) |
| scheduler | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| scoped-tls | 1.0.1 | Apache-2.0 | registry (crates.io) |
| scopeguard | 1.2.0 | Apache-2.0 | registry (crates.io) |
| self_cell | 1.3.0 | Apache-2.0 | registry (crates.io) |
| semver | 1.0.28 | Apache-2.0 | registry (crates.io) |
| serde | 1.0.229 | Apache-2.0 | registry (crates.io) |
| serde_bytes | 0.11.19 | Apache-2.0 | registry (crates.io) |
| serde_core | 1.0.229 | Apache-2.0 | registry (crates.io) |
| serde_derive | 1.0.229 | Apache-2.0 | registry (crates.io) |
| serde_derive_internals | 0.30.0 | Apache-2.0 | registry (crates.io) |
| serde_fmt | 1.1.0 | Apache-2.0 | registry (crates.io) |
| serde_json | 1.0.151 | Apache-2.0 | registry (crates.io) |
| serde_repr | 0.1.21 | Apache-2.0 | registry (crates.io) |
| serde_spanned | 0.6.9 | Apache-2.0 | registry (crates.io) |
| serde_spanned | 1.1.1 | Apache-2.0 | registry (crates.io) |
| serde_urlencoded | 0.7.1 | Apache-2.0 | registry (crates.io) |
| sha2 | 0.10.9 | Apache-2.0 | registry (crates.io) |
| shlex | 1.3.0 | Apache-2.0 | registry (crates.io) |
| shlex | 2.0.1 | Apache-2.0 | registry (crates.io) |
| signal-hook-registry | 1.4.8 | Apache-2.0 | registry (crates.io) |
| simplecss | 0.2.2 | Apache-2.0 | registry (crates.io) |
| siphasher | 1.0.3 | Apache-2.0 | registry (crates.io) |
| skrifa | 0.40.0 | Apache-2.0 | registry (crates.io) |
| skrifa | 0.44.0 | Apache-2.0 | registry (crates.io) |
| smallvec | 1.15.2 | Apache-2.0 | registry (crates.io) |
| smol | 2.0.2 | Apache-2.0 | registry (crates.io) |
| smol_str | 0.3.6 | Apache-2.0 | registry (crates.io) |
| spirv | 0.4.0+sdk-1.4.341.0 | Apache-2.0 | registry (crates.io) |
| stable_deref_trait | 1.2.1 | Apache-2.0 | registry (crates.io) |
| stacker | 0.1.25 | Apache-2.0 | registry (crates.io) |
| stacksafe | 1.0.3 | Apache-2.0 | registry (crates.io) |
| stacksafe-macro | 1.0.3 | Apache-2.0 | registry (crates.io) |
| static_assertions | 1.1.0 | Apache-2.0 | registry (crates.io) |
| sum_tree | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| svg_fmt | 0.4.5 | Apache-2.0 | registry (crates.io) |
| svgtypes | 0.16.1 | Apache-2.0 | registry (crates.io) |
| swash | 0.2.10 | Apache-2.0 | registry (crates.io) |
| syn | 2.0.119 | Apache-2.0 | registry (crates.io) |
| syn | 3.0.4 | Apache-2.0 | registry (crates.io) |
| sys-locale | 0.3.2 | Apache-2.0 | registry (crates.io) |
| tauri-winrt-notification | 0.7.3 | Apache-2.0 | registry (crates.io) |
| tempfile | 3.27.0 | Apache-2.0 | registry (crates.io) |
| thiserror | 1.0.69 | Apache-2.0 | registry (crates.io) |
| thiserror | 2.0.20 | Apache-2.0 | registry (crates.io) |
| thiserror-impl | 1.0.69 | Apache-2.0 | registry (crates.io) |
| thiserror-impl | 2.0.20 | Apache-2.0 | registry (crates.io) |
| time | 0.3.55 | Apache-2.0 | registry (crates.io) |
| time-core | 0.1.9 | Apache-2.0 | registry (crates.io) |
| tinyvec | 1.12.0 | Apache-2.0 | registry (crates.io) |
| tinyvec_macros | 0.1.1 | Apache-2.0 | registry (crates.io) |
| toml | 0.8.23 | Apache-2.0 | registry (crates.io) |
| toml | 1.1.4+spec-1.1.0 | Apache-2.0 | registry (crates.io) |
| toml_datetime | 0.6.11 | Apache-2.0 | registry (crates.io) |
| toml_datetime | 1.1.1+spec-1.1.0 | Apache-2.0 | registry (crates.io) |
| toml_edit | 0.22.27 | Apache-2.0 | registry (crates.io) |
| toml_edit | 0.25.13+spec-1.1.0 | Apache-2.0 | registry (crates.io) |
| toml_parser | 1.1.3+spec-1.1.0 | Apache-2.0 | registry (crates.io) |
| toml_write | 0.1.2 | Apache-2.0 | registry (crates.io) |
| toml_writer | 1.1.2+spec-1.1.0 | Apache-2.0 | registry (crates.io) |
| ttf-parser | 0.25.1 | Apache-2.0 | registry (crates.io) |
| typeid | 1.0.3 | Apache-2.0 | registry (crates.io) |
| typenum | 1.20.1 | Apache-2.0 | registry (crates.io) |
| unarray | 0.1.4 | Apache-2.0 | registry (crates.io) |
| unicode-bidi | 0.3.18 | Apache-2.0 | registry (crates.io) |
| unicode-bidi-mirroring | 0.4.0 | Apache-2.0 | registry (crates.io) |
| unicode-ccc | 0.4.0 | Apache-2.0 | registry (crates.io) |
| unicode-ident | 1.0.24 | Apache-2.0 | registry (crates.io) |
| unicode-linebreak | 0.1.5 | Apache-2.0 | registry (crates.io) |
| unicode-properties | 0.1.4 | Apache-2.0 | registry (crates.io) |
| unicode-script | 0.5.8 | Apache-2.0 | registry (crates.io) |
| unicode-segmentation | 1.13.3 | Apache-2.0 | registry (crates.io) |
| unicode-vo | 0.1.0 | Apache-2.0 | registry (crates.io) |
| unicode-width | 0.2.2 | Apache-2.0 | registry (crates.io) |
| unicode-xid | 0.2.6 | Apache-2.0 | registry (crates.io) |
| url | 2.5.8 | Apache-2.0 | registry (crates.io) |
| usvg | 0.46.0 | Apache-2.0 | registry (crates.io) |
| utf8_iter | 1.0.4 | Apache-2.0 | registry (crates.io) |
| util_macros | 0.1.0 | Apache-2.0 | git (git+https://github.com/zed-industries/zed?rev=6805d952f9f3d702f760aa11b1547df8a625fa16#6805d952f9f3d702f760aa11b1547df8a625fa16) |
| uuid | 1.25.0 | Apache-2.0 | registry (crates.io) |
| value-bag | 1.13.2 | Apache-2.0 | registry (crates.io) |
| value-bag-serde1 | 1.13.2 | Apache-2.0 | registry (crates.io) |
| version_check | 0.9.5 | Apache-2.0 | registry (crates.io) |
| wait-timeout | 0.2.1 | Apache-2.0 | registry (crates.io) |
| waker-fn | 1.2.0 | Apache-2.0 | registry (crates.io) |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 | registry (crates.io) |
| wasip2 | 1.0.4+wasi-0.2.12 | Apache-2.0 | registry (crates.io) |
| wasm-bindgen | 0.2.127 | Apache-2.0 | registry (crates.io) |
| wasm-bindgen-futures | 0.4.77 | Apache-2.0 | registry (crates.io) |
| wasm-bindgen-macro | 0.2.127 | Apache-2.0 | registry (crates.io) |
| wasm-bindgen-macro-support | 0.2.127 | Apache-2.0 | registry (crates.io) |
| wasm-bindgen-shared | 0.2.127 | Apache-2.0 | registry (crates.io) |
| wasm_thread | 0.3.3 | Apache-2.0 | git (git+https://github.com/zed-industries/wasm_thread?rev=0cf96c7708dfb97ccf3da50347e25edcf75d6937#0cf96c7708dfb97ccf3da50347e25edcf75d6937) |
| web-sys | 0.3.104 | Apache-2.0 | registry (crates.io) |
| web-time | 1.1.0 | Apache-2.0 | registry (crates.io) |
| weezl | 0.1.12 | Apache-2.0 | registry (crates.io) |
| wgpu | 29.0.4 | Apache-2.0 | registry (crates.io) |
| wgpu-core | 29.0.4 | Apache-2.0 | registry (crates.io) |
| wgpu-core-deps-apple | 29.0.4 | Apache-2.0 | registry (crates.io) |
| wgpu-core-deps-emscripten | 29.0.4 | Apache-2.0 | registry (crates.io) |
| wgpu-core-deps-wasm | 29.0.4 | Apache-2.0 | registry (crates.io) |
| wgpu-core-deps-windows-linux-android | 29.0.4 | Apache-2.0 | registry (crates.io) |
| wgpu-hal | 29.0.4 | Apache-2.0 | registry (crates.io) |
| wgpu-naga-bridge | 29.0.4 | Apache-2.0 | registry (crates.io) |
| wgpu-types | 29.0.4 | Apache-2.0 | registry (crates.io) |
| winapi | 0.3.9 | Apache-2.0 | registry (crates.io) |
| winapi-i686-pc-windows-gnu | 0.4.0 | Apache-2.0 | registry (crates.io) |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | Apache-2.0 | registry (crates.io) |
| windows | 0.61.3 | Apache-2.0 | registry (crates.io) |
| windows | 0.62.2 | Apache-2.0 | registry (crates.io) |
| windows-collections | 0.2.0 | Apache-2.0 | registry (crates.io) |
| windows-collections | 0.3.2 | Apache-2.0 | registry (crates.io) |
| windows-core | 0.61.2 | Apache-2.0 | registry (crates.io) |
| windows-core | 0.62.2 | Apache-2.0 | registry (crates.io) |
| windows-future | 0.2.1 | Apache-2.0 | registry (crates.io) |
| windows-future | 0.3.2 | Apache-2.0 | registry (crates.io) |
| windows-implement | 0.60.2 | Apache-2.0 | registry (crates.io) |
| windows-interface | 0.59.3 | Apache-2.0 | registry (crates.io) |
| windows-link | 0.1.3 | Apache-2.0 | registry (crates.io) |
| windows-link | 0.2.1 | Apache-2.0 | registry (crates.io) |
| windows-numerics | 0.2.0 | Apache-2.0 | registry (crates.io) |
| windows-numerics | 0.3.1 | Apache-2.0 | registry (crates.io) |
| windows-registry | 0.5.3 | Apache-2.0 | registry (crates.io) |
| windows-result | 0.3.4 | Apache-2.0 | registry (crates.io) |
| windows-result | 0.4.1 | Apache-2.0 | registry (crates.io) |
| windows-strings | 0.4.2 | Apache-2.0 | registry (crates.io) |
| windows-strings | 0.5.1 | Apache-2.0 | registry (crates.io) |
| windows-sys | 0.59.0 | Apache-2.0 | registry (crates.io) |
| windows-sys | 0.61.2 | Apache-2.0 | registry (crates.io) |
| windows-targets | 0.52.6 | Apache-2.0 | registry (crates.io) |
| windows-threading | 0.1.0 | Apache-2.0 | registry (crates.io) |
| windows-threading | 0.2.1 | Apache-2.0 | registry (crates.io) |
| windows-version | 0.1.7 | Apache-2.0 | registry (crates.io) |
| windows_aarch64_gnullvm | 0.52.6 | Apache-2.0 | registry (crates.io) |
| windows_aarch64_msvc | 0.52.6 | Apache-2.0 | registry (crates.io) |
| windows_i686_gnu | 0.52.6 | Apache-2.0 | registry (crates.io) |
| windows_i686_gnullvm | 0.52.6 | Apache-2.0 | registry (crates.io) |
| windows_i686_msvc | 0.52.6 | Apache-2.0 | registry (crates.io) |
| windows_x86_64_gnu | 0.52.6 | Apache-2.0 | registry (crates.io) |
| windows_x86_64_gnullvm | 0.52.6 | Apache-2.0 | registry (crates.io) |
| windows_x86_64_msvc | 0.52.6 | Apache-2.0 | registry (crates.io) |
| wio | 0.2.2 | Apache-2.0 | registry (crates.io) |
| wit-bindgen | 0.57.1 | Apache-2.0 | registry (crates.io) |
| x11rb | 0.13.2 | Apache-2.0 | registry (crates.io) |
| x11rb-protocol | 0.13.2 | Apache-2.0 | registry (crates.io) |
| xkeysym | 0.2.1 | Apache-2.0 | registry (crates.io) |
| yazi | 0.2.1 | Apache-2.0 | registry (crates.io) |
| zed-font-kit | 0.14.1-zed | Apache-2.0 | git (git+https://github.com/zed-industries/font-kit?rev=94b0f28166665e8fd2f53ff6d268a14955c82269#94b0f28166665e8fd2f53ff6d268a14955c82269) |
| zeno | 0.3.3 | Apache-2.0 | registry (crates.io) |
| zerocopy | 0.8.56 | Apache-2.0 | registry (crates.io) |
| zerocopy-derive | 0.8.56 | Apache-2.0 | registry (crates.io) |
| zeroize | 1.9.0 | Apache-2.0 | registry (crates.io) |
| zeroize_derive | 1.5.0 | Apache-2.0 | registry (crates.io) |
| zune-core | 0.5.3 | Apache-2.0 | registry (crates.io) |
| zune-inflate | 0.2.54 | Apache-2.0 | registry (crates.io) |
| zune-jpeg | 0.5.15 | Apache-2.0 | registry (crates.io) |

**Total Apache-2.0: 481 package records.**

### Apache-2.0 WITH LLVM-exception

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| ar_archive_writer | 0.5.3 | Apache-2.0 WITH LLVM-exception | registry (crates.io) |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception | registry (crates.io) |
| linux-raw-sys | 0.4.15 | Apache-2.0 WITH LLVM-exception | registry (crates.io) |
| rustix | 0.38.44 | Apache-2.0 WITH LLVM-exception | registry (crates.io) |
| rustix | 1.1.4 | Apache-2.0 WITH LLVM-exception | registry (crates.io) |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception | registry (crates.io) |
| wasip2 | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception | registry (crates.io) |
| wit-bindgen | 0.57.1 | Apache-2.0 WITH LLVM-exception | registry (crates.io) |

**Total Apache-2.0 WITH LLVM-exception: 8 package records.**

### BSD-2-Clause

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| arrayref | 0.3.9 | BSD-2-Clause | registry (crates.io) |
| av1-grain | 0.2.5 | BSD-2-Clause | registry (crates.io) |
| mach2 | 0.5.0 | BSD-2-Clause | registry (crates.io) |
| rav1e | 0.8.1 | BSD-2-Clause | registry (crates.io) |
| v_frame | 0.3.9 | BSD-2-Clause | registry (crates.io) |
| zerocopy | 0.8.56 | BSD-2-Clause | registry (crates.io) |
| zerocopy-derive | 0.8.56 | BSD-2-Clause | registry (crates.io) |

**Total BSD-2-Clause: 7 package records.**

### BSD-3-Clause

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| avif-serialize | 0.8.9 | BSD-3-Clause | registry (crates.io) |
| bindgen | 0.71.1 | BSD-3-Clause | registry (crates.io) |
| encoding_rs | 0.8.35 | BSD-3-Clause | registry (crates.io) |
| exr | 1.74.2 | BSD-3-Clause | registry (crates.io) |
| lebe | 0.5.3 | BSD-3-Clause | registry (crates.io) |
| moxcms | 0.8.1 | BSD-3-Clause | registry (crates.io) |
| pxfm | 0.1.30 | BSD-3-Clause | registry (crates.io) |
| ravif | 0.13.0 | BSD-3-Clause | registry (crates.io) |
| sha1_smol | 1.0.1 | BSD-3-Clause | registry (crates.io) |
| subtle | 2.6.1 | BSD-3-Clause | registry (crates.io) |
| tiny-skia | 0.11.4 | BSD-3-Clause | registry (crates.io) |
| tiny-skia-path | 0.11.4 | BSD-3-Clause | registry (crates.io) |

**Total BSD-3-Clause: 12 package records.**

### BSL-1.0

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| ryu | 1.0.23 | BSL-1.0 | registry (crates.io) |

**Total BSL-1.0: 1 package records.**

### bzip2-1.0.6

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| libbz2-rs-sys | 0.2.5 | bzip2-1.0.6 | registry (crates.io) |

**Total bzip2-1.0.6: 1 package records.**

### CC0-1.0

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| dunce | 1.0.5 | CC0-1.0 | registry (crates.io) |
| hexf-parse | 0.2.1 | CC0-1.0 | registry (crates.io) |
| imgref | 1.12.2 | CC0-1.0 | registry (crates.io) |
| tiny-keccak | 2.0.2 | CC0-1.0 | registry (crates.io) |

**Total CC0-1.0: 4 package records.**

### GPL-2.0-only

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| self_cell | 1.3.0 | GPL-2.0-only | registry (crates.io) |

**Total GPL-2.0-only: 1 package records.**

### ISC

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| libloading | 0.8.9 | ISC | registry (crates.io) |

**Total ISC: 1 package records.**

### LGPL-2.1-or-later

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| r-efi | 5.3.0 | LGPL-2.1-or-later | registry (crates.io) |
| r-efi | 6.0.0 | LGPL-2.1-or-later | registry (crates.io) |

**Total LGPL-2.1-or-later: 2 package records.**

### MIT

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| accesskit | 0.24.1 | MIT | registry (crates.io) |
| accesskit_atspi_common | 0.18.1 | MIT | registry (crates.io) |
| accesskit_consumer | 0.36.0 | MIT | registry (crates.io) |
| accesskit_consumer | 0.37.0 | MIT | registry (crates.io) |
| accesskit_consumer | 0.38.0 | MIT | registry (crates.io) |
| accesskit_macos | 0.26.3 | MIT | registry (crates.io) |
| accesskit_unix | 0.21.1 | MIT | registry (crates.io) |
| accesskit_windows | 0.33.1 | MIT | registry (crates.io) |
| addr2line | 0.25.1 | MIT | registry (crates.io) |
| adler2 | 2.0.1 | MIT | registry (crates.io) |
| aes | 0.8.4 | MIT | registry (crates.io) |
| ahash | 0.8.12 | MIT | registry (crates.io) |
| aho-corasick | 1.1.5 | MIT | registry (crates.io) |
| aligned | 0.4.3 | MIT | registry (crates.io) |
| aligned-vec | 0.6.4 | MIT | registry (crates.io) |
| allocator-api2 | 0.2.21 | MIT | registry (crates.io) |
| android_system_properties | 0.1.6 | MIT | registry (crates.io) |
| anyhow | 1.0.104 | MIT | registry (crates.io) |
| arbitrary | 1.4.2 | MIT | registry (crates.io) |
| arg_enum_proc_macro | 0.3.4 | MIT | registry (crates.io) |
| arrayvec | 0.7.8 | MIT | registry (crates.io) |
| as-raw-xcb-connection | 1.0.1 | MIT | registry (crates.io) |
| as-slice | 0.2.1 | MIT | registry (crates.io) |
| ash | 0.38.0+1.3.281 | MIT | registry (crates.io) |
| ashpd | 0.13.13 | MIT | registry (crates.io) |
| async-broadcast | 0.7.2 | MIT | registry (crates.io) |
| async-channel | 2.5.0 | MIT | registry (crates.io) |
| async-compression | 0.4.43 | MIT | registry (crates.io) |
| async-executor | 1.14.0 | MIT | registry (crates.io) |
| async-fs | 2.2.0 | MIT | registry (crates.io) |
| async-io | 2.6.0 | MIT | registry (crates.io) |
| async-lock | 3.4.2 | MIT | registry (crates.io) |
| async-net | 2.0.0 | MIT | registry (crates.io) |
| async-process | 2.5.0 | MIT | registry (crates.io) |
| async-recursion | 1.1.1 | MIT | registry (crates.io) |
| async-signal | 0.2.14 | MIT | registry (crates.io) |
| async-task | 4.7.1 | MIT | registry (crates.io) |
| async-trait | 0.1.92 | MIT | registry (crates.io) |
| atomic | 0.5.3 | MIT | registry (crates.io) |
| atomic-waker | 1.1.2 | MIT | registry (crates.io) |
| atspi | 0.29.0 | MIT | registry (crates.io) |
| atspi-common | 0.13.0 | MIT | registry (crates.io) |
| atspi-proxies | 0.13.0 | MIT | registry (crates.io) |
| autocfg | 1.5.1 | MIT | registry (crates.io) |
| av-scenechange | 0.14.1 | MIT | registry (crates.io) |
| backtrace | 0.3.76 | MIT | registry (crates.io) |
| base64 | 0.22.1 | MIT | registry (crates.io) |
| bit-set | 0.8.0 | MIT | registry (crates.io) |
| bit-set | 0.9.1 | MIT | registry (crates.io) |
| bit-vec | 0.8.0 | MIT | registry (crates.io) |
| bit-vec | 0.9.1 | MIT | registry (crates.io) |
| bit_field | 0.10.3 | MIT | registry (crates.io) |
| bitflags | 1.3.2 | MIT | registry (crates.io) |
| bitflags | 2.13.1 | MIT | registry (crates.io) |
| bitstream-io | 4.10.0 | MIT | registry (crates.io) |
| block | 0.1.6 | MIT | registry (crates.io) |
| block-buffer | 0.10.4 | MIT | registry (crates.io) |
| block-padding | 0.3.3 | MIT | registry (crates.io) |
| block2 | 0.6.2 | MIT | registry (crates.io) |
| blocking | 1.7.0 | MIT | registry (crates.io) |
| built | 0.8.1 | MIT | registry (crates.io) |
| bumpalo | 3.20.3 | MIT | registry (crates.io) |
| bytemuck | 1.25.2 | MIT | registry (crates.io) |
| bytemuck_derive | 1.12.0 | MIT | registry (crates.io) |
| byteorder | 1.5.0 | MIT | registry (crates.io) |
| byteorder-lite | 0.1.0 | MIT | registry (crates.io) |
| bytes | 1.12.1 | MIT | registry (crates.io) |
| bzip2 | 0.6.1 | MIT | registry (crates.io) |
| calloop | 0.14.4 | MIT | registry (crates.io) |
| calloop-wayland-source | 0.4.1 | MIT | registry (crates.io) |
| cbc | 0.1.2 | MIT | registry (crates.io) |
| cc | 1.4.4 | MIT | registry (crates.io) |
| cexpr | 0.6.0 | MIT | registry (crates.io) |
| cfg-if | 1.0.4 | MIT | registry (crates.io) |
| cfg_aliases | 0.2.2 | MIT | registry (crates.io) |
| cgl | 0.3.2 | MIT | registry (crates.io) |
| chrono | 0.4.45 | MIT | registry (crates.io) |
| cipher | 0.4.4 | MIT | registry (crates.io) |
| cocoa | 0.26.0 | MIT | registry (crates.io) |
| cocoa-foundation | 0.2.1 | MIT | registry (crates.io) |
| color_quant | 1.1.0 | MIT | registry (crates.io) |
| compression-codecs | 0.4.38 | MIT | registry (crates.io) |
| compression-core | 0.4.32 | MIT | registry (crates.io) |
| concurrent-queue | 2.5.0 | MIT | registry (crates.io) |
| console_error_panic_hook | 0.1.7 | MIT | registry (crates.io) |
| const-random | 0.1.18 | MIT | registry (crates.io) |
| const-random-macro | 0.1.16 | MIT | registry (crates.io) |
| convert_case | 0.10.0 | MIT | registry (crates.io) |
| convert_case | 0.11.0 | MIT | registry (crates.io) |
| core-foundation | 0.10.1 | MIT | registry (crates.io) |
| core-foundation-sys | 0.8.7 | MIT | registry (crates.io) |
| core-graphics | 0.24.0 | MIT | registry (crates.io) |
| core-graphics-types | 0.2.0 | MIT | registry (crates.io) |
| core-graphics2 | 0.5.2 | MIT | registry (crates.io) |
| core-text | 21.0.0 | MIT | registry (crates.io) |
| core-video | 0.5.2 | MIT | registry (crates.io) |
| core_maths | 0.1.1 | MIT | registry (crates.io) |
| cosmic-text | 0.19.0 | MIT | registry (crates.io) |
| cpufeatures | 0.2.17 | MIT | registry (crates.io) |
| crc32fast | 1.5.1 | MIT | registry (crates.io) |
| crossbeam-deque | 0.8.7 | MIT | registry (crates.io) |
| crossbeam-epoch | 0.9.20 | MIT | registry (crates.io) |
| crossbeam-queue | 0.3.13 | MIT | registry (crates.io) |
| crossbeam-utils | 0.8.22 | MIT | registry (crates.io) |
| crunchy | 0.2.4 | MIT | registry (crates.io) |
| crypto-common | 0.1.7 | MIT | registry (crates.io) |
| ctor | 1.0.13 | MIT | registry (crates.io) |
| data-url | 0.3.2 | MIT | registry (crates.io) |
| deranged | 0.5.8 | MIT | registry (crates.io) |
| derive_more | 2.1.1 | MIT | registry (crates.io) |
| derive_more-impl | 2.1.1 | MIT | registry (crates.io) |
| digest | 0.10.7 | MIT | registry (crates.io) |
| dirs | 6.0.0 | MIT | registry (crates.io) |
| dirs-sys | 0.5.0 | MIT | registry (crates.io) |
| dispatch2 | 0.3.1 | MIT | registry (crates.io) |
| displaydoc | 0.2.7 | MIT | registry (crates.io) |
| dlib | 0.5.3 | MIT | registry (crates.io) |
| document-features | 0.2.12 | MIT | registry (crates.io) |
| downcast-rs | 1.2.1 | MIT | registry (crates.io) |
| dyn-clone | 1.0.20 | MIT | registry (crates.io) |
| either | 1.18.0 | MIT | registry (crates.io) |
| embed-resource | 3.0.11 | MIT | registry (crates.io) |
| encoding_rs | 0.8.35 | MIT | registry (crates.io) |
| endi | 1.1.1 | MIT | registry (crates.io) |
| enumflags2 | 0.7.12 | MIT | registry (crates.io) |
| enumflags2_derive | 0.7.12 | MIT | registry (crates.io) |
| enumn | 0.1.14 | MIT | registry (crates.io) |
| equator | 0.4.2 | MIT | registry (crates.io) |
| equator-macro | 0.4.2 | MIT | registry (crates.io) |
| equivalent | 1.0.2 | MIT | registry (crates.io) |
| erased-serde | 0.4.10 | MIT | registry (crates.io) |
| errno | 0.3.14 | MIT | registry (crates.io) |
| etagere | 0.2.15 | MIT | registry (crates.io) |
| euclid | 0.22.14 | MIT | registry (crates.io) |
| event-listener | 5.4.2 | MIT | registry (crates.io) |
| event-listener-strategy | 0.5.4 | MIT | registry (crates.io) |
| fastrand | 2.5.0 | MIT | registry (crates.io) |
| fax | 0.2.7 | MIT | registry (crates.io) |
| fdeflate | 0.3.7 | MIT | registry (crates.io) |
| filedescriptor | 0.8.3 | MIT | registry (crates.io) |
| find-msvc-tools | 0.1.11 | MIT | registry (crates.io) |
| fixedbitset | 0.5.7 | MIT | registry (crates.io) |
| flate2 | 1.1.9 | MIT | registry (crates.io) |
| float-cmp | 0.9.0 | MIT | registry (crates.io) |
| float-ord | 0.3.2 | MIT | registry (crates.io) |
| float_next_after | 1.0.0 | MIT | registry (crates.io) |
| flume | 0.12.0 | MIT | registry (crates.io) |
| fnv | 1.0.7 | MIT | registry (crates.io) |
| font-types | 0.11.3 | MIT | registry (crates.io) |
| font-types | 0.12.4 | MIT | registry (crates.io) |
| fontconfig-parser | 0.5.8 | MIT | registry (crates.io) |
| fontdb | 0.23.0 | MIT | registry (crates.io) |
| foreign-types | 0.5.0 | MIT | registry (crates.io) |
| foreign-types-macros | 0.2.4 | MIT | registry (crates.io) |
| foreign-types-shared | 0.3.1 | MIT | registry (crates.io) |
| form_urlencoded | 1.2.2 | MIT | registry (crates.io) |
| freetype-sys | 0.20.1 | MIT | registry (crates.io) |
| futures | 0.3.34 | MIT | registry (crates.io) |
| futures-channel | 0.3.34 | MIT | registry (crates.io) |
| futures-concurrency | 7.7.1 | MIT | registry (crates.io) |
| futures-core | 0.3.34 | MIT | registry (crates.io) |
| futures-executor | 0.3.34 | MIT | registry (crates.io) |
| futures-io | 0.3.34 | MIT | registry (crates.io) |
| futures-lite | 2.6.1 | MIT | registry (crates.io) |
| futures-macro | 0.3.34 | MIT | registry (crates.io) |
| futures-sink | 0.3.34 | MIT | registry (crates.io) |
| futures-task | 0.3.34 | MIT | registry (crates.io) |
| futures-util | 0.3.34 | MIT | registry (crates.io) |
| generic-array | 0.14.7 | MIT | registry (crates.io) |
| getrandom | 0.2.17 | MIT | registry (crates.io) |
| getrandom | 0.3.4 | MIT | registry (crates.io) |
| getrandom | 0.4.3 | MIT | registry (crates.io) |
| gif | 0.14.2 | MIT | registry (crates.io) |
| gimli | 0.32.3 | MIT | registry (crates.io) |
| glob | 0.3.4 | MIT | registry (crates.io) |
| glow | 0.17.0 | MIT | registry (crates.io) |
| gpu-allocator | 0.28.0 | MIT | registry (crates.io) |
| gpu-descriptor | 0.3.2 | MIT | registry (crates.io) |
| gpu-descriptor-types | 0.2.0 | MIT | registry (crates.io) |
| half | 2.7.1 | MIT | registry (crates.io) |
| harfrust | 0.5.2 | MIT | registry (crates.io) |
| hash32 | 0.3.1 | MIT | registry (crates.io) |
| hashbrown | 0.14.5 | MIT | registry (crates.io) |
| hashbrown | 0.15.5 | MIT | registry (crates.io) |
| hashbrown | 0.16.1 | MIT | registry (crates.io) |
| hashbrown | 0.17.1 | MIT | registry (crates.io) |
| heapless | 0.9.3 | MIT | registry (crates.io) |
| heck | 0.4.1 | MIT | registry (crates.io) |
| heck | 0.5.0 | MIT | registry (crates.io) |
| hermit-abi | 0.5.2 | MIT | registry (crates.io) |
| hex | 0.4.3 | MIT | registry (crates.io) |
| hkdf | 0.12.4 | MIT | registry (crates.io) |
| hmac | 0.12.1 | MIT | registry (crates.io) |
| home | 0.5.12 | MIT | registry (crates.io) |
| http | 1.5.0 | MIT | registry (crates.io) |
| http-body | 1.1.0 | MIT | registry (crates.io) |
| iana-time-zone | 0.1.65 | MIT | registry (crates.io) |
| iana-time-zone-haiku | 0.1.2 | MIT | registry (crates.io) |
| idna | 1.1.0 | MIT | registry (crates.io) |
| idna_adapter | 1.2.2 | MIT | registry (crates.io) |
| image | 0.25.10 | MIT | registry (crates.io) |
| image-webp | 0.2.4 | MIT | registry (crates.io) |
| imagesize | 0.14.0 | MIT | registry (crates.io) |
| indexmap | 2.14.0 | MIT | registry (crates.io) |
| inout | 0.1.4 | MIT | registry (crates.io) |
| interpolate_name | 0.2.4 | MIT | registry (crates.io) |
| inventory | 0.3.24 | MIT | registry (crates.io) |
| io-surface | 0.16.1 | MIT | registry (crates.io) |
| is-docker | 0.2.0 | MIT | registry (crates.io) |
| is-wsl | 0.4.0 | MIT | registry (crates.io) |
| itertools | 0.13.0 | MIT | registry (crates.io) |
| itertools | 0.14.0 | MIT | registry (crates.io) |
| itoa | 1.0.18 | MIT | registry (crates.io) |
| jni-sys | 0.3.1 | MIT | registry (crates.io) |
| jni-sys | 0.4.1 | MIT | registry (crates.io) |
| jni-sys-macros | 0.4.1 | MIT | registry (crates.io) |
| jobserver | 0.1.35 | MIT | registry (crates.io) |
| js-sys | 0.3.104 | MIT | registry (crates.io) |
| khronos-egl | 6.0.0 | MIT | registry (crates.io) |
| kurbo | 0.13.1 | MIT | registry (crates.io) |
| lazy_static | 1.5.0 | MIT | registry (crates.io) |
| leak | 0.1.2 | MIT | registry (crates.io) |
| leaky-cow | 0.1.1 | MIT | registry (crates.io) |
| libc | 0.2.189 | MIT | registry (crates.io) |
| libfuzzer-sys | 0.4.13 | MIT | registry (crates.io) |
| libm | 0.2.16 | MIT | registry (crates.io) |
| libredox | 0.1.20 | MIT | registry (crates.io) |
| linebender_resource_handle | 0.1.1 | MIT | registry (crates.io) |
| link-section | 0.19.3 | MIT | registry (crates.io) |
| linktime-proc-macro | 0.2.3 | MIT | registry (crates.io) |
| linux-raw-sys | 0.12.1 | MIT | registry (crates.io) |
| linux-raw-sys | 0.4.15 | MIT | registry (crates.io) |
| litrs | 1.0.0 | MIT | registry (crates.io) |
| lock_api | 0.4.14 | MIT | registry (crates.io) |
| log | 0.4.34 | MIT | registry (crates.io) |
| loop9 | 0.1.5 | MIT | registry (crates.io) |
| lyon | 1.0.19 | MIT | registry (crates.io) |
| lyon_algorithms | 1.0.20 | MIT | registry (crates.io) |
| lyon_geom | 1.0.19 | MIT | registry (crates.io) |
| lyon_path | 1.0.19 | MIT | registry (crates.io) |
| lyon_tessellation | 1.0.20 | MIT | registry (crates.io) |
| mac-notification-sys | 0.6.15 | MIT | registry (crates.io) |
| mach2 | 0.5.0 | MIT | registry (crates.io) |
| malloc_buf | 0.0.6 | MIT | registry (crates.io) |
| maybe-rayon | 0.1.1 | MIT | registry (crates.io) |
| md-5 | 0.10.6 | MIT | registry (crates.io) |
| memchr | 2.8.3 | MIT | registry (crates.io) |
| memmap2 | 0.9.11 | MIT | registry (crates.io) |
| memoffset | 0.9.1 | MIT | registry (crates.io) |
| metal | 0.33.0 | MIT | registry (crates.io) |
| minimal-lexical | 0.2.1 | MIT | registry (crates.io) |
| miniz_oxide | 0.8.9 | MIT | registry (crates.io) |
| naga | 29.0.4 | MIT | registry (crates.io) |
| ndk-sys | 0.6.0+11769913 | MIT | registry (crates.io) |
| new_debug_unreachable | 1.0.6 | MIT | registry (crates.io) |
| no_std_io2 | 0.9.4 | MIT | registry (crates.io) |
| nom | 7.1.3 | MIT | registry (crates.io) |
| nom | 8.0.0 | MIT | registry (crates.io) |
| noop_proc_macro | 0.3.0 | MIT | registry (crates.io) |
| notify-rust | 4.18.0 | MIT | registry (crates.io) |
| num | 0.4.3 | MIT | registry (crates.io) |
| num-bigint | 0.4.8 | MIT | registry (crates.io) |
| num-bigint-dig | 0.9.1 | MIT | registry (crates.io) |
| num-complex | 0.4.6 | MIT | registry (crates.io) |
| num-conv | 0.2.2 | MIT | registry (crates.io) |
| num-derive | 0.4.2 | MIT | registry (crates.io) |
| num-integer | 0.1.47 | MIT | registry (crates.io) |
| num-iter | 0.1.46 | MIT | registry (crates.io) |
| num-rational | 0.4.2 | MIT | registry (crates.io) |
| num-traits | 0.2.19 | MIT | registry (crates.io) |
| num_cpus | 1.17.0 | MIT | registry (crates.io) |
| objc | 0.2.7 | MIT | registry (crates.io) |
| objc-sys | 0.3.5 | MIT | registry (crates.io) |
| objc2 | 0.5.2 | MIT | registry (crates.io) |
| objc2 | 0.6.4 | MIT | registry (crates.io) |
| objc2-app-kit | 0.2.2 | MIT | registry (crates.io) |
| objc2-app-kit | 0.3.2 | MIT | registry (crates.io) |
| objc2-core-foundation | 0.3.2 | MIT | registry (crates.io) |
| objc2-core-location | 0.3.2 | MIT | registry (crates.io) |
| objc2-encode | 4.1.0 | MIT | registry (crates.io) |
| objc2-foundation | 0.2.2 | MIT | registry (crates.io) |
| objc2-foundation | 0.3.2 | MIT | registry (crates.io) |
| objc2-metal | 0.3.2 | MIT | registry (crates.io) |
| objc2-quartz-core | 0.3.2 | MIT | registry (crates.io) |
| objc2-user-notifications | 0.3.2 | MIT | registry (crates.io) |
| object | 0.37.3 | MIT | registry (crates.io) |
| object | 0.39.1 | MIT | registry (crates.io) |
| once_cell | 1.21.4 | MIT | registry (crates.io) |
| oo7 | 0.6.0 | MIT | registry (crates.io) |
| open | 5.4.2 | MIT | registry (crates.io) |
| ordered-float | 5.5.0 | MIT | registry (crates.io) |
| ordered-stream | 0.2.0 | MIT | registry (crates.io) |
| parking | 2.2.1 | MIT | registry (crates.io) |
| parking_lot | 0.12.5 | MIT | registry (crates.io) |
| parking_lot_core | 0.9.12 | MIT | registry (crates.io) |
| paste | 1.0.15 | MIT | registry (crates.io) |
| pastey | 0.1.1 | MIT | registry (crates.io) |
| pathfinder_geometry | 0.5.1 | MIT | registry (crates.io) |
| pathfinder_simd | 0.5.6 | MIT | registry (crates.io) |
| pbkdf2 | 0.12.2 | MIT | registry (crates.io) |
| percent-encoding | 2.3.2 | MIT | registry (crates.io) |
| phf | 0.13.1 | MIT | registry (crates.io) |
| phf_generator | 0.13.1 | MIT | registry (crates.io) |
| phf_macros | 0.13.1 | MIT | registry (crates.io) |
| phf_shared | 0.13.1 | MIT | registry (crates.io) |
| pico-args | 0.5.0 | MIT | registry (crates.io) |
| pin-project | 1.1.13 | MIT | registry (crates.io) |
| pin-project-internal | 1.1.13 | MIT | registry (crates.io) |
| pin-project-lite | 0.2.17 | MIT | registry (crates.io) |
| piper | 0.2.5 | MIT | registry (crates.io) |
| pkg-config | 0.3.34 | MIT | registry (crates.io) |
| png | 0.17.16 | MIT | registry (crates.io) |
| png | 0.18.1 | MIT | registry (crates.io) |
| polling | 3.11.0 | MIT | registry (crates.io) |
| pollster | 0.2.5 | MIT | registry (crates.io) |
| pollster | 0.4.0 | MIT | registry (crates.io) |
| polycool | 0.4.0 | MIT | registry (crates.io) |
| portable-atomic | 1.15.0 | MIT | registry (crates.io) |
| portable-atomic-util | 0.2.7 | MIT | registry (crates.io) |
| postage | 0.5.0 | MIT | registry (crates.io) |
| powerfmt | 0.2.0 | MIT | registry (crates.io) |
| ppv-lite86 | 0.2.21 | MIT | registry (crates.io) |
| presser | 0.3.1 | MIT | registry (crates.io) |
| prettyplease | 0.2.37 | MIT | registry (crates.io) |
| proc-macro-crate | 3.5.0 | MIT | registry (crates.io) |
| proc-macro2 | 1.0.107 | MIT | registry (crates.io) |
| profiling | 1.0.18 | MIT | registry (crates.io) |
| profiling-procmacros | 1.0.18 | MIT | registry (crates.io) |
| proptest | 1.10.0 | MIT | git (git+https://github.com/proptest-rs/proptest?rev=3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b#3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b) |
| proptest-macro | 0.5.0 | MIT | git (git+https://github.com/proptest-rs/proptest?rev=3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b#3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b) |
| psm | 0.1.32 | MIT | registry (crates.io) |
| pulp | 0.22.3 | MIT | registry (crates.io) |
| pulp-wasm-simd-flag | 0.1.1 | MIT | registry (crates.io) |
| qoi | 0.4.1 | MIT | registry (crates.io) |
| quick-error | 1.2.3 | MIT | registry (crates.io) |
| quick-error | 2.0.1 | MIT | registry (crates.io) |
| quick-xml | 0.41.0 | MIT | registry (crates.io) |
| quote | 1.0.47 | MIT | registry (crates.io) |
| r-efi | 5.3.0 | MIT | registry (crates.io) |
| r-efi | 6.0.0 | MIT | registry (crates.io) |
| rand | 0.9.5 | MIT | registry (crates.io) |
| rand_chacha | 0.9.0 | MIT | registry (crates.io) |
| rand_core | 0.9.5 | MIT | registry (crates.io) |
| rand_xorshift | 0.4.0 | MIT | registry (crates.io) |
| range-alloc | 0.1.5 | MIT | registry (crates.io) |
| rangemap | 1.8.0 | MIT | registry (crates.io) |
| raw-cpuid | 11.6.0 | MIT | registry (crates.io) |
| raw-window-handle | 0.6.2 | MIT | registry (crates.io) |
| raw-window-metal | 1.1.0 | MIT | registry (crates.io) |
| rayon | 1.12.0 | MIT | registry (crates.io) |
| rayon-core | 1.13.0 | MIT | registry (crates.io) |
| read-fonts | 0.37.0 | MIT | registry (crates.io) |
| read-fonts | 0.41.0 | MIT | registry (crates.io) |
| reborrow | 0.5.5 | MIT | registry (crates.io) |
| redox_syscall | 0.5.18 | MIT | registry (crates.io) |
| redox_users | 0.5.2 | MIT | registry (crates.io) |
| ref-cast | 1.0.27 | MIT | registry (crates.io) |
| ref-cast-impl | 1.0.27 | MIT | registry (crates.io) |
| regex | 1.13.1 | MIT | registry (crates.io) |
| regex-automata | 0.4.18 | MIT | registry (crates.io) |
| regex-syntax | 0.8.11 | MIT | registry (crates.io) |
| renderdoc-sys | 1.1.0 | MIT | registry (crates.io) |
| resvg | 0.46.0 | MIT | registry (crates.io) |
| rgb | 0.8.53 | MIT | registry (crates.io) |
| rmp | 0.8.15 | MIT | registry (crates.io) |
| rmp-serde | 1.3.1 | MIT | registry (crates.io) |
| roxmltree | 0.20.0 | MIT | registry (crates.io) |
| roxmltree | 0.21.1 | MIT | registry (crates.io) |
| rustc-demangle | 0.1.28 | MIT | registry (crates.io) |
| rustc-hash | 1.1.0 | MIT | registry (crates.io) |
| rustc-hash | 2.1.3 | MIT | registry (crates.io) |
| rustc_version | 0.4.1 | MIT | registry (crates.io) |
| rustix | 0.38.44 | MIT | registry (crates.io) |
| rustix | 1.1.4 | MIT | registry (crates.io) |
| rustversion | 1.0.23 | MIT | registry (crates.io) |
| rusty-fork | 0.3.1 | MIT | registry (crates.io) |
| rustybuzz | 0.20.1 | MIT | registry (crates.io) |
| same-file | 1.0.6 | MIT | registry (crates.io) |
| schemars | 1.2.2 | MIT | registry (crates.io) |
| schemars_derive | 1.2.2 | MIT | registry (crates.io) |
| scoped-tls | 1.0.1 | MIT | registry (crates.io) |
| scopeguard | 1.2.0 | MIT | registry (crates.io) |
| seahash | 4.1.0 | MIT | registry (crates.io) |
| semver | 1.0.28 | MIT | registry (crates.io) |
| serde | 1.0.229 | MIT | registry (crates.io) |
| serde_bytes | 0.11.19 | MIT | registry (crates.io) |
| serde_core | 1.0.229 | MIT | registry (crates.io) |
| serde_derive | 1.0.229 | MIT | registry (crates.io) |
| serde_derive_internals | 0.30.0 | MIT | registry (crates.io) |
| serde_fmt | 1.1.0 | MIT | registry (crates.io) |
| serde_json | 1.0.151 | MIT | registry (crates.io) |
| serde_repr | 0.1.21 | MIT | registry (crates.io) |
| serde_spanned | 0.6.9 | MIT | registry (crates.io) |
| serde_spanned | 1.1.1 | MIT | registry (crates.io) |
| serde_urlencoded | 0.7.1 | MIT | registry (crates.io) |
| sha2 | 0.10.9 | MIT | registry (crates.io) |
| shlex | 1.3.0 | MIT | registry (crates.io) |
| shlex | 2.0.1 | MIT | registry (crates.io) |
| signal-hook-registry | 1.4.8 | MIT | registry (crates.io) |
| simd-adler32 | 0.3.10 | MIT | registry (crates.io) |
| simd_helpers | 0.1.0 | MIT | registry (crates.io) |
| simplecss | 0.2.2 | MIT | registry (crates.io) |
| siphasher | 1.0.3 | MIT | registry (crates.io) |
| skrifa | 0.40.0 | MIT | registry (crates.io) |
| skrifa | 0.44.0 | MIT | registry (crates.io) |
| slab | 0.4.12 | MIT | registry (crates.io) |
| smallvec | 1.15.2 | MIT | registry (crates.io) |
| smol | 2.0.2 | MIT | registry (crates.io) |
| smol_str | 0.3.6 | MIT | registry (crates.io) |
| spin | 0.10.1 | MIT | registry (crates.io) |
| spin | 0.9.9 | MIT | registry (crates.io) |
| stable_deref_trait | 1.2.1 | MIT | registry (crates.io) |
| stacker | 0.1.25 | MIT | registry (crates.io) |
| static_assertions | 1.1.0 | MIT | registry (crates.io) |
| strict-num | 0.1.1 | MIT | registry (crates.io) |
| strum | 0.27.2 | MIT | registry (crates.io) |
| strum_macros | 0.27.2 | MIT | registry (crates.io) |
| svg_fmt | 0.4.5 | MIT | registry (crates.io) |
| svgtypes | 0.16.1 | MIT | registry (crates.io) |
| swash | 0.2.10 | MIT | registry (crates.io) |
| syn | 2.0.119 | MIT | registry (crates.io) |
| syn | 3.0.4 | MIT | registry (crates.io) |
| synstructure | 0.13.2 | MIT | registry (crates.io) |
| sys-locale | 0.3.2 | MIT | registry (crates.io) |
| taffy | 0.13.0 | MIT | registry (crates.io) |
| tauri-winrt-notification | 0.7.3 | MIT | registry (crates.io) |
| tempfile | 3.27.0 | MIT | registry (crates.io) |
| thiserror | 1.0.69 | MIT | registry (crates.io) |
| thiserror | 2.0.20 | MIT | registry (crates.io) |
| thiserror-impl | 1.0.69 | MIT | registry (crates.io) |
| thiserror-impl | 2.0.20 | MIT | registry (crates.io) |
| tiff | 0.11.3 | MIT | registry (crates.io) |
| time | 0.3.55 | MIT | registry (crates.io) |
| time-core | 0.1.9 | MIT | registry (crates.io) |
| tinyvec | 1.12.0 | MIT | registry (crates.io) |
| tinyvec_macros | 0.1.1 | MIT | registry (crates.io) |
| toml | 0.8.23 | MIT | registry (crates.io) |
| toml | 1.1.4+spec-1.1.0 | MIT | registry (crates.io) |
| toml_datetime | 0.6.11 | MIT | registry (crates.io) |
| toml_datetime | 1.1.1+spec-1.1.0 | MIT | registry (crates.io) |
| toml_edit | 0.22.27 | MIT | registry (crates.io) |
| toml_edit | 0.25.13+spec-1.1.0 | MIT | registry (crates.io) |
| toml_parser | 1.1.3+spec-1.1.0 | MIT | registry (crates.io) |
| toml_write | 0.1.2 | MIT | registry (crates.io) |
| toml_writer | 1.1.2+spec-1.1.0 | MIT | registry (crates.io) |
| tracing | 0.1.44 | MIT | registry (crates.io) |
| tracing-attributes | 0.1.31 | MIT | registry (crates.io) |
| tracing-core | 0.1.36 | MIT | registry (crates.io) |
| ttf-parser | 0.25.1 | MIT | registry (crates.io) |
| typeid | 1.0.3 | MIT | registry (crates.io) |
| typenum | 1.20.1 | MIT | registry (crates.io) |
| uds_windows | 1.2.1 | MIT | registry (crates.io) |
| unarray | 0.1.4 | MIT | registry (crates.io) |
| unicode-bidi | 0.3.18 | MIT | registry (crates.io) |
| unicode-bidi-mirroring | 0.4.0 | MIT | registry (crates.io) |
| unicode-ccc | 0.4.0 | MIT | registry (crates.io) |
| unicode-ident | 1.0.24 | MIT | registry (crates.io) |
| unicode-properties | 0.1.4 | MIT | registry (crates.io) |
| unicode-script | 0.5.8 | MIT | registry (crates.io) |
| unicode-segmentation | 1.13.3 | MIT | registry (crates.io) |
| unicode-vo | 0.1.0 | MIT | registry (crates.io) |
| unicode-width | 0.2.2 | MIT | registry (crates.io) |
| unicode-xid | 0.2.6 | MIT | registry (crates.io) |
| url | 2.5.8 | MIT | registry (crates.io) |
| usvg | 0.46.0 | MIT | registry (crates.io) |
| utf8_iter | 1.0.4 | MIT | registry (crates.io) |
| uuid | 1.25.0 | MIT | registry (crates.io) |
| value-bag | 1.13.2 | MIT | registry (crates.io) |
| value-bag-serde1 | 1.13.2 | MIT | registry (crates.io) |
| version_check | 0.9.5 | MIT | registry (crates.io) |
| vswhom | 0.1.0 | MIT | registry (crates.io) |
| vswhom-sys | 0.1.3 | MIT | registry (crates.io) |
| wait-timeout | 0.2.1 | MIT | registry (crates.io) |
| waker-fn | 1.2.0 | MIT | registry (crates.io) |
| walkdir | 2.5.0 | MIT | registry (crates.io) |
| wasi | 0.11.1+wasi-snapshot-preview1 | MIT | registry (crates.io) |
| wasip2 | 1.0.4+wasi-0.2.12 | MIT | registry (crates.io) |
| wasm-bindgen | 0.2.127 | MIT | registry (crates.io) |
| wasm-bindgen-futures | 0.4.77 | MIT | registry (crates.io) |
| wasm-bindgen-macro | 0.2.127 | MIT | registry (crates.io) |
| wasm-bindgen-macro-support | 0.2.127 | MIT | registry (crates.io) |
| wasm-bindgen-shared | 0.2.127 | MIT | registry (crates.io) |
| wasm_thread | 0.3.3 | MIT | git (git+https://github.com/zed-industries/wasm_thread?rev=0cf96c7708dfb97ccf3da50347e25edcf75d6937#0cf96c7708dfb97ccf3da50347e25edcf75d6937) |
| wayland-backend | 0.3.17 | MIT | registry (crates.io) |
| wayland-client | 0.31.15 | MIT | registry (crates.io) |
| wayland-cursor | 0.31.14 | MIT | registry (crates.io) |
| wayland-protocols | 0.32.13 | MIT | registry (crates.io) |
| wayland-protocols-plasma | 0.3.12 | MIT | registry (crates.io) |
| wayland-protocols-wlr | 0.3.12 | MIT | registry (crates.io) |
| wayland-scanner | 0.31.11 | MIT | registry (crates.io) |
| wayland-sys | 0.31.11 | MIT | registry (crates.io) |
| web-sys | 0.3.104 | MIT | registry (crates.io) |
| web-time | 1.1.0 | MIT | registry (crates.io) |
| weezl | 0.1.12 | MIT | registry (crates.io) |
| wgpu | 29.0.4 | MIT | registry (crates.io) |
| wgpu-core | 29.0.4 | MIT | registry (crates.io) |
| wgpu-core-deps-apple | 29.0.4 | MIT | registry (crates.io) |
| wgpu-core-deps-emscripten | 29.0.4 | MIT | registry (crates.io) |
| wgpu-core-deps-wasm | 29.0.4 | MIT | registry (crates.io) |
| wgpu-core-deps-windows-linux-android | 29.0.4 | MIT | registry (crates.io) |
| wgpu-hal | 29.0.4 | MIT | registry (crates.io) |
| wgpu-naga-bridge | 29.0.4 | MIT | registry (crates.io) |
| wgpu-types | 29.0.4 | MIT | registry (crates.io) |
| which | 6.0.3 | MIT | registry (crates.io) |
| winapi | 0.3.9 | MIT | registry (crates.io) |
| winapi-i686-pc-windows-gnu | 0.4.0 | MIT | registry (crates.io) |
| winapi-util | 0.1.11 | MIT | registry (crates.io) |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | MIT | registry (crates.io) |
| windows | 0.61.3 | MIT | registry (crates.io) |
| windows | 0.62.2 | MIT | registry (crates.io) |
| windows-collections | 0.2.0 | MIT | registry (crates.io) |
| windows-collections | 0.3.2 | MIT | registry (crates.io) |
| windows-core | 0.61.2 | MIT | registry (crates.io) |
| windows-core | 0.62.2 | MIT | registry (crates.io) |
| windows-future | 0.2.1 | MIT | registry (crates.io) |
| windows-future | 0.3.2 | MIT | registry (crates.io) |
| windows-implement | 0.60.2 | MIT | registry (crates.io) |
| windows-interface | 0.59.3 | MIT | registry (crates.io) |
| windows-link | 0.1.3 | MIT | registry (crates.io) |
| windows-link | 0.2.1 | MIT | registry (crates.io) |
| windows-numerics | 0.2.0 | MIT | registry (crates.io) |
| windows-numerics | 0.3.1 | MIT | registry (crates.io) |
| windows-registry | 0.5.3 | MIT | registry (crates.io) |
| windows-result | 0.3.4 | MIT | registry (crates.io) |
| windows-result | 0.4.1 | MIT | registry (crates.io) |
| windows-strings | 0.4.2 | MIT | registry (crates.io) |
| windows-strings | 0.5.1 | MIT | registry (crates.io) |
| windows-sys | 0.59.0 | MIT | registry (crates.io) |
| windows-sys | 0.61.2 | MIT | registry (crates.io) |
| windows-targets | 0.52.6 | MIT | registry (crates.io) |
| windows-threading | 0.1.0 | MIT | registry (crates.io) |
| windows-threading | 0.2.1 | MIT | registry (crates.io) |
| windows-version | 0.1.7 | MIT | registry (crates.io) |
| windows_aarch64_gnullvm | 0.52.6 | MIT | registry (crates.io) |
| windows_aarch64_msvc | 0.52.6 | MIT | registry (crates.io) |
| windows_i686_gnu | 0.52.6 | MIT | registry (crates.io) |
| windows_i686_gnullvm | 0.52.6 | MIT | registry (crates.io) |
| windows_i686_msvc | 0.52.6 | MIT | registry (crates.io) |
| windows_x86_64_gnu | 0.52.6 | MIT | registry (crates.io) |
| windows_x86_64_gnullvm | 0.52.6 | MIT | registry (crates.io) |
| windows_x86_64_msvc | 0.52.6 | MIT | registry (crates.io) |
| winnow | 0.7.15 | MIT | registry (crates.io) |
| winnow | 1.0.4 | MIT | registry (crates.io) |
| winreg | 0.55.0 | MIT | registry (crates.io) |
| winsafe | 0.0.19 | MIT | registry (crates.io) |
| wio | 0.2.2 | MIT | registry (crates.io) |
| wit-bindgen | 0.57.1 | MIT | registry (crates.io) |
| x11-clipboard | 0.9.3 | MIT | registry (crates.io) |
| x11rb | 0.13.2 | MIT | registry (crates.io) |
| x11rb-protocol | 0.13.2 | MIT | registry (crates.io) |
| xcursor | 0.3.11 | MIT | registry (crates.io) |
| xim-ctext | 0.3.0 | MIT | git (git+https://github.com/zed-industries/xim-rs.git?rev=16f35a2c881b815a2b6cdfd6687988e84f8447d8#16f35a2c881b815a2b6cdfd6687988e84f8447d8) |
| xim-parser | 0.2.1 | MIT | git (git+https://github.com/zed-industries/xim-rs.git?rev=16f35a2c881b815a2b6cdfd6687988e84f8447d8#16f35a2c881b815a2b6cdfd6687988e84f8447d8) |
| xkbcommon | 0.8.0 | MIT | registry (crates.io) |
| xkeysym | 0.2.1 | MIT | registry (crates.io) |
| xml-rs | 0.8.29 | MIT | registry (crates.io) |
| xmlwriter | 0.1.0 | MIT | registry (crates.io) |
| y4m | 0.8.0 | MIT | registry (crates.io) |
| yazi | 0.2.1 | MIT | registry (crates.io) |
| yeslogic-fontconfig-sys | 6.0.1 | MIT | registry (crates.io) |
| zbus | 5.19.0 | MIT | registry (crates.io) |
| zbus-lockstep | 0.5.2 | MIT | registry (crates.io) |
| zbus-lockstep-macros | 0.5.2 | MIT | registry (crates.io) |
| zbus_macros | 5.19.0 | MIT | registry (crates.io) |
| zbus_names | 4.3.4 | MIT | registry (crates.io) |
| zbus_xml | 5.2.1 | MIT | registry (crates.io) |
| zcheapstr | 1.1.0 | MIT | registry (crates.io) |
| zed-font-kit | 0.14.1-zed | MIT | git (git+https://github.com/zed-industries/font-kit?rev=94b0f28166665e8fd2f53ff6d268a14955c82269#94b0f28166665e8fd2f53ff6d268a14955c82269) |
| zed-xim | 0.4.0-zed | MIT | git (git+https://github.com/zed-industries/xim-rs.git?rev=16f35a2c881b815a2b6cdfd6687988e84f8447d8#16f35a2c881b815a2b6cdfd6687988e84f8447d8) |
| zeno | 0.3.3 | MIT | registry (crates.io) |
| zerocopy | 0.8.56 | MIT | registry (crates.io) |
| zerocopy-derive | 0.8.56 | MIT | registry (crates.io) |
| zeroize | 1.9.0 | MIT | registry (crates.io) |
| zeroize_derive | 1.5.0 | MIT | registry (crates.io) |
| zmij | 1.0.23 | MIT | registry (crates.io) |
| zune-core | 0.5.3 | MIT | registry (crates.io) |
| zune-inflate | 0.2.54 | MIT | registry (crates.io) |
| zune-jpeg | 0.5.15 | MIT | registry (crates.io) |
| zvariant | 5.15.0 | MIT | registry (crates.io) |
| zvariant_derive | 5.15.0 | MIT | registry (crates.io) |
| zvariant_utils | 4.2.0 | MIT | registry (crates.io) |

**Total MIT: 581 package records.**

### MIT-0

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| dunce | 1.0.5 | MIT-0 | registry (crates.io) |

**Total MIT-0: 1 package records.**

### MPL-2.0

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| cbindgen | 0.28.0 | MPL-2.0 | registry (crates.io) |
| dwrote | 0.11.5 | MPL-2.0 | registry (crates.io) |
| option-ext | 0.2.0 | MPL-2.0 | registry (crates.io) |

**Total MPL-2.0: 3 package records.**

### NCSA

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| libfuzzer-sys | 0.4.13 | NCSA | registry (crates.io) |

**Total NCSA: 1 package records.**

### Unicode-3.0

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| icu_collections | 2.3.0 | Unicode-3.0 | registry (crates.io) |
| icu_locale_core | 2.3.0 | Unicode-3.0 | registry (crates.io) |
| icu_normalizer | 2.3.0 | Unicode-3.0 | registry (crates.io) |
| icu_normalizer_data | 2.3.0 | Unicode-3.0 | registry (crates.io) |
| icu_properties | 2.3.0 | Unicode-3.0 | registry (crates.io) |
| icu_properties_data | 2.3.0 | Unicode-3.0 | registry (crates.io) |
| icu_provider | 2.3.1 | Unicode-3.0 | registry (crates.io) |
| litemap | 0.8.3 | Unicode-3.0 | registry (crates.io) |
| potential_utf | 0.1.6 | Unicode-3.0 | registry (crates.io) |
| tinystr | 0.8.4 | Unicode-3.0 | registry (crates.io) |
| unicode-ident | 1.0.24 | Unicode-3.0 | registry (crates.io) |
| writeable | 0.6.4 | Unicode-3.0 | registry (crates.io) |
| yoke | 0.8.3 | Unicode-3.0 | registry (crates.io) |
| yoke-derive | 0.8.2 | Unicode-3.0 | registry (crates.io) |
| zerofrom | 0.1.8 | Unicode-3.0 | registry (crates.io) |
| zerofrom-derive | 0.1.7 | Unicode-3.0 | registry (crates.io) |
| zerotrie | 0.2.5 | Unicode-3.0 | registry (crates.io) |
| zerovec | 0.11.8 | Unicode-3.0 | registry (crates.io) |
| zerovec-derive | 0.11.6 | Unicode-3.0 | registry (crates.io) |

**Total Unicode-3.0: 19 package records.**

### Unlicense

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| aho-corasick | 1.1.5 | Unlicense | registry (crates.io) |
| byteorder | 1.5.0 | Unlicense | registry (crates.io) |
| byteorder-lite | 0.1.0 | Unlicense | registry (crates.io) |
| memchr | 2.8.3 | Unlicense | registry (crates.io) |
| same-file | 1.0.6 | Unlicense | registry (crates.io) |
| walkdir | 2.5.0 | Unlicense | registry (crates.io) |
| winapi-util | 0.1.11 | Unlicense | registry (crates.io) |

**Total Unlicense: 7 package records.**

### Zlib

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| bytemuck | 1.25.2 | Zlib | registry (crates.io) |
| bytemuck_derive | 1.12.0 | Zlib | registry (crates.io) |
| dispatch2 | 0.3.1 | Zlib | registry (crates.io) |
| foldhash | 0.1.5 | Zlib | registry (crates.io) |
| foldhash | 0.2.0 | Zlib | registry (crates.io) |
| glow | 0.17.0 | Zlib | registry (crates.io) |
| miniz_oxide | 0.8.9 | Zlib | registry (crates.io) |
| objc2-app-kit | 0.3.2 | Zlib | registry (crates.io) |
| objc2-core-foundation | 0.3.2 | Zlib | registry (crates.io) |
| objc2-core-location | 0.3.2 | Zlib | registry (crates.io) |
| objc2-metal | 0.3.2 | Zlib | registry (crates.io) |
| objc2-quartz-core | 0.3.2 | Zlib | registry (crates.io) |
| objc2-user-notifications | 0.3.2 | Zlib | registry (crates.io) |
| raw-window-handle | 0.6.2 | Zlib | registry (crates.io) |
| slotmap | 1.1.1 | Zlib | registry (crates.io) |
| tinyvec | 1.12.0 | Zlib | registry (crates.io) |
| tinyvec_macros | 0.1.1 | Zlib | registry (crates.io) |
| xkeysym | 0.2.1 | Zlib | registry (crates.io) |
| zune-core | 0.5.3 | Zlib | registry (crates.io) |
| zune-inflate | 0.2.54 | Zlib | registry (crates.io) |
| zune-jpeg | 0.5.15 | Zlib | registry (crates.io) |

**Total Zlib: 21 package records.**

## Project-owned Rust crates

These local Cargo records are project-owned rather than third-party dependencies. The `ztracing` row is the independently authored Apache-2.0 stub used by the host graph.

| Crate | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| react-gpui | 0.2.0 | Apache-2.0 | local (project-owned) |
| react-gpui-bun | 0.2.0 | Apache-2.0 | local (project-owned) |
| react-gpui-host | 0.2.0 | Apache-2.0 | local (project-owned) |
| ztracing | 0.1.0 | Apache-2.0 | local (project-owned) |

**Total project-owned Rust crates: 4.**

## Bun dependencies

The following two inventories are the direct outputs of `bun pm licenses --all` for the core and dev package workspaces. Entries marked `dev` are development-only in that workspace.

### `@react-gpui/core`

#### Apache-2.0

| Package | Version | License (SPDX) | Source | Scope |
| --- | --- | --- | --- | --- |
| typescript | 5.9.3 | Apache-2.0 | registry (npm) | dev |

**Total Apache-2.0 (@react-gpui/core): 1 package records.**

#### ISC

| Package | Version | License (SPDX) | Source | Scope |
| --- | --- | --- | --- | --- |
| @msgpack/msgpack | 3.1.3 | ISC | registry (npm) | runtime |

**Total ISC (@react-gpui/core): 1 package records.**

#### MIT

| Package | Version | License (SPDX) | Source | Scope |
| --- | --- | --- | --- | --- |
| @types/node | 20.12.14 | MIT | registry (npm) | dev |
| @types/node | 26.2.0 | MIT | registry (npm) | dev |
| @types/react | 19.2.2 | MIT | registry (npm) | dev |
| @types/ws | 8.5.14 | MIT | registry (npm) | dev |
| bun-types | 1.1.29 | MIT | registry (npm) | dev |
| csstype | 3.2.3 | MIT | registry (npm) | dev |
| prettier | 3.6.2 | MIT | registry (npm) | dev |
| react | 19.2.8 | MIT | registry (npm) | runtime |
| react-reconciler | 0.33.0 | MIT | registry (npm) | runtime |
| scheduler | 0.27.0 | MIT | registry (npm) | runtime |
| undici-types | 5.26.5 | MIT | registry (npm) | dev |
| undici-types | 8.3.0 | MIT | registry (npm) | dev |

**Total MIT (@react-gpui/core): 12 package records.**

### `@react-gpui/dev`

#### Apache-2.0

| Package | Version | License (SPDX) | Source | Scope |
| --- | --- | --- | --- | --- |
| @react-gpui/core | 0.2.0 | Apache-2.0 | local (project-owned) | dev |
| baseline-browser-mapping | 2.11.18 | Apache-2.0 | registry (npm) | runtime |
| typescript | 5.9.3 | Apache-2.0 | registry (npm) | dev |

**Total Apache-2.0 (@react-gpui/dev): 3 package records.**

#### CC-BY-4.0

| Package | Version | License (SPDX) | Source | Scope |
| --- | --- | --- | --- | --- |
| caniuse-lite | 1.0.30001809 | CC-BY-4.0 | registry (npm) | runtime |

**Total CC-BY-4.0 (@react-gpui/dev): 1 package records.**

#### ISC

| Package | Version | License (SPDX) | Source | Scope |
| --- | --- | --- | --- | --- |
| @msgpack/msgpack | 3.1.3 | ISC | registry (npm) | runtime |
| electron-to-chromium | 1.5.413 | ISC | registry (npm) | runtime |
| lru-cache | 5.1.1 | ISC | registry (npm) | runtime |
| picocolors | 1.1.1 | ISC | registry (npm) | runtime |
| semver | 6.3.1 | ISC | registry (npm) | runtime |
| yallist | 3.1.1 | ISC | registry (npm) | runtime |

**Total ISC (@react-gpui/dev): 6 package records.**

#### MIT

| Package | Version | License (SPDX) | Source | Scope |
| --- | --- | --- | --- | --- |
| @babel/code-frame | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/compat-data | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/core | 7.29.6 | MIT | registry (npm) | runtime |
| @babel/generator | 7.29.8 | MIT | registry (npm) | runtime |
| @babel/helper-annotate-as-pure | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/helper-compilation-targets | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/helper-globals | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/helper-module-imports | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/helper-module-transforms | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/helper-plugin-utils | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/helper-string-parser | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/helper-validator-identifier | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/helper-validator-option | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/helpers | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/parser | 7.29.8 | MIT | registry (npm) | runtime |
| @babel/plugin-syntax-jsx | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/plugin-transform-react-jsx | 7.27.1 | MIT | registry (npm) | runtime |
| @babel/template | 7.29.7 | MIT | registry (npm) | runtime |
| @babel/traverse | 7.29.8 | MIT | registry (npm) | runtime |
| @babel/types | 7.29.8 | MIT | registry (npm) | runtime |
| @jridgewell/gen-mapping | 0.3.13 | MIT | registry (npm) | runtime |
| @jridgewell/remapping | 2.3.5 | MIT | registry (npm) | runtime |
| @jridgewell/resolve-uri | 3.1.2 | MIT | registry (npm) | runtime |
| @jridgewell/sourcemap-codec | 1.5.5 | MIT | registry (npm) | runtime |
| @jridgewell/trace-mapping | 0.3.31 | MIT | registry (npm) | runtime |
| @types/node | 20.12.14 | MIT | registry (npm) | dev |
| @types/node | 25.0.0 | MIT | registry (npm) | dev |
| @types/react | 19.2.2 | MIT | registry (npm) | dev |
| @types/react-test-renderer | 19.1.0 | MIT | registry (npm) | dev |
| @types/ws | 8.5.14 | MIT | registry (npm) | dev |
| browserslist | 4.28.8 | MIT | registry (npm) | runtime |
| bun-types | 1.1.29 | MIT | registry (npm) | dev |
| convert-source-map | 2.0.0 | MIT | registry (npm) | runtime |
| csstype | 3.2.3 | MIT | registry (npm) | dev |
| debug | 4.4.3 | MIT | registry (npm) | runtime |
| escalade | 3.2.0 | MIT | registry (npm) | runtime |
| gensync | 1.0.0-beta.2 | MIT | registry (npm) | runtime |
| js-tokens | 4.0.0 | MIT | registry (npm) | runtime |
| jsesc | 3.1.0 | MIT | registry (npm) | runtime |
| json5 | 2.2.3 | MIT | registry (npm) | runtime |
| ms | 2.1.3 | MIT | registry (npm) | runtime |
| node-releases | 2.0.53 | MIT | registry (npm) | runtime |
| prettier | 3.6.2 | MIT | registry (npm) | dev |
| react | 19.2.8 | MIT | registry (npm) | runtime |
| react-is | 19.2.8 | MIT | registry (npm) | dev |
| react-reconciler | 0.33.0 | MIT | registry (npm) | dev |
| react-refresh | 0.18.0 | MIT | registry (npm) | runtime |
| react-test-renderer | 19.2.8 | MIT | registry (npm) | dev |
| scheduler | 0.27.0 | MIT | registry (npm) | dev |
| undici-types | 5.26.5 | MIT | registry (npm) | dev |
| undici-types | 7.16.0 | MIT | registry (npm) | dev |
| update-browserslist-db | 1.3.1 | MIT | registry (npm) | runtime |

**Total MIT (@react-gpui/dev): 52 package records.**

## Project-owned Bun packages

These package manifests carry the project's own Apache-2.0 license and are not third-party dependencies. Their npm tarballs include their own `LICENSE`; the shared JavaScript dependency inventory remains in this release artifact.

| Package | Version | License (SPDX) | Source |
| --- | --- | --- | --- |
| @react-gpui/core | 0.2.0 | Apache-2.0 | local (project-owned) |
| @react-gpui/dev | 0.2.0 | Apache-2.0 | local (project-owned) |

**Total project-owned Bun packages: 2.**
