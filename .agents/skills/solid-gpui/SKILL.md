---
name: solid-gpui
description: Develop solid-gpui applications with SolidJS native pages, routing, themes, input, virtual lists, Vite/Bun hot reload, and generated native APIs. Also use gpui-performance for performance design or diagnosis.
---

# solid-gpui Development

## 1. Find the actual entry point and state ownership

Read the project's CONTEXT.md, package README, and relevant example. Identify the
Solid universal JSX transform, runtime entry point, Surface, host profile, and
extension capabilities. For Vite, Bun, or hot reload, read the
[hot reload guide](../../../docs/hot-reload.md) and verify resolution conditions,
epochs, and resource cleanup. Consult the relevant sections of the
[application conventions](references/application.md).

**Completion criterion**: Explain which state Solid owns and which state native
code owns. Locate the actual application entry point and exported APIs; derive
native behavior from their contracts rather than React, DOM, or Web CSS assumptions.

## 2. Keep changes local

Read signals inside JSX or reactive computations; memoize expensive derived data.
Batch related state changes through the existing runtime. Bind resource lifetimes
to onCleanup/Surface. Keep the routing shell, navigation, and panes mounted across
page changes; replace page content through Outlet.

For layout, high-frequency events, lists, or asynchronous heavy work, also read
[GPUI performance](../gpui-performance/SKILL.md).

**Completion criterion**: Explain the invalidation scope of a local interaction.
Input does not rebuild the whole page, high-frequency listeners have consumers,
and batching preserves event and command order.

## 3. Follow native contracts

Select styles, events, and commands from the actual type definitions. Obtain window
width from the window-size store and colors from the reactive theme. Use generated
extension components or native clients. Protocol changes follow schema → generator
→ checker. List, input, and native-function invariants are in the
[key acceptance scenarios](references/verification.md).

**Completion criterion**: Layout works at narrow and wide sizes; text, controls,
and placeholders remain readable in light and dark themes. Error handling and
resource release are explicit. Generated protocol fields have a canonical source,
and obsolete compatibility wrappers are removed.

## 4. Verify and deliver

Run the key tests and type checks for the changed paths. Inspect UI changes in an
actual native window. For performance changes, retain comparison evidence using
the [performance analysis guide](../../../docs/performance-analysis.md).
Check current repository task names before running commands.

**Completion criterion**: Report trigger → result, checks, and limitations.
Interactions, scroll position, and data correctness survive optimization. Record
reusable findings in one authoritative reference, with their conditions of validity.
