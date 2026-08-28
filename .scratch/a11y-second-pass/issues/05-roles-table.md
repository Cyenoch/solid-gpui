# Accessibility role table and README truth

Status: resolved
Type: task

Refresh public documentation for the implemented expanded/heading-level fields, label-vs-description mapping, and live-region boundary.

## Answer

The README a11y table now includes Expanded and Level columns. It documents `accessibilityExpanded`, positive heading-only `accessibilityLevel`, independent label/description semantics, retained-but-unexpressible `accessibilityDisabled`, and the AccessKit Live property as an upstream gap because pinned GPUI exposes no public live builder/write path. `docs/protocol.md` carries the positional tuple shape and source citations.

Evidence: `packages/react-gpui/README.md:129-159`; `docs/protocol.md:295-324`.
