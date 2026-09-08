# GPUI dependency patch

Source: crates.io `gpui-pre` 0.3.3, copied from the resolved registry source.
The original Apache-2.0 license and upstream provenance are retained.

Local changes:

- Bundle the two upstream SVG-test font fixtures with their original licenses,
  and use crate-relative test paths; the published crate refers outside its package.

- Include the line limit in wrapped text cache identity, so changing a line clamp
  cannot reuse incompatible geometry within or across frames.

The native bidirectional geometry refactor is tracked in
`../../.scratch/native-production-completion/bidi-implementation.md`.
