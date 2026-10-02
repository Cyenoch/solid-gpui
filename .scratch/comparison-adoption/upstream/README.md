# Local GPUI upstream fix candidate

`gpui-line-clamp-cache.patch` isolates the already-vendored wrapped-text cache
fix from the original `gpui-pre-0.3.7` archive. The original archive SHA-256 is
`0e87a42bb37c7cb4e76dd1ac0ce88851e46e976e0373a47ab3e0757abffee54d`;
its manifest names Zed `1a28cff4b409169bac058bca40dfbfeb7621d19b`.
This is a local review artifact; no upstream communication has been sent.

`layout_wrapped_line` accepts `max_lines`, but the archive's cache identity omits
that argument. Shaping the same text/font/runs/width with a different line clamp
can therefore reuse the first layout. The fix includes `max_lines` in borrowed
and owned cache keys; unwrapped lines use `None`. The regression changes clamps
within a frame and after `finish_frame`, and confirms equal keys still reuse the
same layout. It uses the current font-generation counter constructor.

Review/apply against an extracted **original** crate directory:

```sh
git apply --check /absolute/path/to/gpui-line-clamp-cache.patch
git apply /absolute/path/to/gpui-line-clamp-cache.patch
```

The patch's `src/text_system/line_layout.rs` path is relative to the crate root;
for a Zed workspace use its `crates/gpui` directory and re-evaluate the current
upstream cache key rather than applying blindly. The patch is an exact archive
diff, containing the source change and regression only, under GPUI's retained
Apache-2.0 terms. It introduces no Solid-specific dependency or runtime contract.

Focused regression in our consuming workspace:

```sh
CARGO_BUILD_JOBS=2 cargo test --locked -p gpui-pre --lib changing_line_clamp_relayouts_within_and_across_frames
```

Archive application is checked separately from execution. Native display,
platform input and performance are outside this deterministic cache regression.
See `../delivery-10.md` for checks actually executed during this delivery.
