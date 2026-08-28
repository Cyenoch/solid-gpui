# Expanded and heading-level semantics

Status: resolved
Type: feature

Add the optional `accessibilityExpanded` and `accessibilityLevel` props and carry them as an append-only accessibility tuple tail. Expanded state uses pinned GPUI's `aria_expanded`; heading level uses `aria_level` and is restricted to the heading role with a positive u32 value.

Evidence: `references/zed/crates/gpui/src/elements/div.rs:1339-1343,1396-1400,1997-2020,3392-3437`; AccessKit 0.24.1 `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/accesskit-0.24.1/src/lib.rs:1992-2006,2117-2123`; renderer seam `crates/react-gpui/src/renderer/paint/accessibility.rs:19-61`.

## Answer

Implemented on both TypeScript and Rust sides. The dropdown and gallery menu triggers advertise expanded state. Level is heading-only and the old seven-field tuple remains valid.

## Comments

The wire tail follows the tooltip precedent: optional fields are appended and `level` retains an `expanded` placeholder when required.
