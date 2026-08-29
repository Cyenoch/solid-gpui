# VirtualList scroll offset

## Status

Implemented and verified through Rust wire, headless host, TypeScript handle, and cross-language golden tests.

## Contract

`VirtualListHandle.getScrollOffset()` returns logical layout pixels from the precise native ListState pixel seam. `scrollToOffset(offset)` accepts finite non-negative pixels and delegates range clamping to GPUI. A pre-layout write is acknowledged by the command protocol but has no native effect until the list has laid out.
