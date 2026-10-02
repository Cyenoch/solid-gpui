# Public native acceptance and correct Unicode test input

Status: ready-for-agent
Execution: implemented and merged; final combined acceptance pending
Blocked by: none

Implement spec outcomes 1 (Unicode) and 5. Own TestHost Unicode correction and meaningful CJK/astral controlled acknowledgement test; a public opt-in JS native acceptance interface over native host/test rendering, actual painted bounds/text, stable locators, hit-tested click/type/drag/wheel, screenshot support with explicit platform semantics, owned cleanup and unsupported cases. Reuse production tree/input/events; do not turn MemoryTransport into fake GPUI geometry. Verify native behavior against the linked GPUI APIs. Document supported capabilities and synchronize related guides/translations and website content. Coordinate shared Rust renderer/host changes at integration. Final report must distinguish GPU, deterministic native and physical-input checks.
