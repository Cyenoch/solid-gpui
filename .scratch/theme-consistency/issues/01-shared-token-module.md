# Shared appearance token module

Type: task
Status: claimed

Create the canonical `examples/theme.ts` module with immutable light/dark color tokens and a small `useTheme(appearance)` selector. Keep palette policy in the application; do not add a renderer theming context.

## Answer

Implemented in `packages/react-gpui/examples/theme.ts`. Tokens cover canvas/surface layers, border/text hierarchy, accent hover/pressed/soft/focus states, success/danger, disabled, input, and shadow colors. Dark values use a slate/navy surface hierarchy and explicit readable foregrounds.

Status: resolved
