# Label and description mapping audit

Status: resolved
Type: research

Audit the existing label and description fields for role-independent mapping drift.

## Answer

Implemented mapping is correct and distinct. Pinned GPUI provides separate `aria_label` and `aria_description` builders (`references/zed/crates/gpui/src/elements/div.rs:1271-1283`) and writes the values independently with `Node::set_label` and `Node::set_description` (`references/zed/crates/gpui/src/elements/div.rs:3392-3401`). The React native painter applies both independently (`crates/react-gpui/src/renderer/paint/accessibility.rs:33-38`) for every recognized role and branch using the helper. README and protocol tables now cite those seams. Generic/unknown roles intentionally produce no AccessKit node, so their metadata is not exposed.

## Comments

No role-specific exception or fallback is needed; the shared painter helper preserves one mapping for View, Text, Pressable, TextInput, VirtualList, Image, and RawText.
