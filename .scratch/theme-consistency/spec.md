# Theme consistency

## Goal

Make every visual React GPUI example teach and exercise application-owned system appearance handling. A light/dark appearance change must update the same semantic surfaces, text, borders, accents, controls, and interaction states without changing layout or behavior.

## Decision

Use one shared `packages/react-gpui/examples/theme.ts` module for the example family. Twelve examples currently live beside one another and share the same visual vocabulary; a single token module keeps their teaching surface honest and avoids demonstrating twelve copies of palette conditionals.

The module exports:

- `lightTheme` and `darkTheme`, immutable plain color-token objects;
- `Theme`, the union of those token-object types;
- `useTheme(appearance)`, a deliberately small selector returning the light or dark token set.

`useTheme` is not a theming context. Each root owns an explicit `createAppearanceStore`, wires `RootOptions.onAppearance` to that store, and passes the store to the example component. Components call `useAppearance` and `useTheme`; the application remains responsible for palette policy, as documented by the library.

Tokens cover canvas/surface layers, borders, text hierarchy, accent and hover/pressed/soft variants, focus rings, success/danger states, disabled states, input backgrounds, and shadows. Dark values are designed from dark-surface contrast rather than mechanically inverting light hex values.

## Scope

Convert visual styling in gallery, dropdown, focus-flow, drag-reorder, todo, text-input, counter, keyboard, stress, virtual-list, selectable-text, and multi-surface. Preserve layout and behavior. Pure logic and native capability demonstrations remain unchanged except where they carry a visual style.

Keep initial stores at `light` so existing process examples have deterministic first paint; native appearance frames then update them through the real callback. No renderer API or context primitive is added.

## Verification

- Package typecheck and formatting cover all example files and the token module.
- Existing examples-render behavior tests remain appearance-pinned to their light startup where they inspect white geometry.
- A process-only probe drives light and dark appearance events through the host bridge and compares the same root/panel/button `background_rgba` values; state token variants are checked in both sets.
