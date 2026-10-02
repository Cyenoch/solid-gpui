# Renderer-wide native selection and search

Status: resolved
Execution: integrated and locally qualified; see integration record for platform limits
Blocked by: none

Implement spec outcome 8. Provide cross-element text selection/search owned by a surface/native renderer with stable node identities, copy semantics, drag/keyboard behavior, styled text and virtualization/removal policy. Reuse native text geometry and existing selectable text/control paths; avoid global JS selection mirrors. Expose bounded meaningful public commands/events through an existing generated native service where possible so no competing schema edits. Define search revision invalidation and per-surface ownership. Key tests cover cross-node Unicode ranges/copy, changed/reordered/deleted content, independent surfaces and cleanup; native acceptance where available. Docs/translations/examples and website synchronization required.

## Final integration

Resolved in the integrated delivery. See [integration record](../integration.md),
[delivery note](../delivery-06.md), and `../qualification/` for
checks, measured counterexamples, and explicit unqualified platforms. Nothing
was published or installed into the user's active application.
