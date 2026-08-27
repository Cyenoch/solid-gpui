# Appearance-aware examples

Type: task
Status: claimed

Convert visual examples to the shared theme module while preserving layout and behavior. Roots must wire `onAppearance` to an explicit per-root appearance store; state styles must use mode-specific tokens.

## Answer

Converted all visual example entries: gallery, dropdown, focus-flow, drag-reorder, todo, text-input, counter, keyboard, stress, virtual-list, selectable-text, and multi-surface. Each standalone root owns an appearance store; components consume `useAppearance` plus `useTheme`, and interaction variants are token-backed.

Status: resolved
