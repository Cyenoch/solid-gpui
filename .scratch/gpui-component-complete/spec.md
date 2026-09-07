# Complete gpui-component JavaScript integration

Request: Make every gpui-component component usable from JavaScript.

Baseline: solid-gpui `1bf74901244d8af95d7d547de5a0fb5ce362b2fc`; gpui-component `928c3eb776a3d733d9b771f7dea27a6a79242ced` (0.6.0), gpui-pre 0.3.3.

Every public UI component must have a usable generated JS export or an explicitly documented composition/service API. Constructor descriptors, state types and drawing geometry are classified separately; no placeholder aliases count as support. Each adapter uses the linked native component, preserves its essential interaction, and owns entities, subscriptions and tasks for exactly one mounted Host Node.

JS owns application data and composition. Native owns editing, focus, popup interaction, scroll and virtualized delegates. Native callbacks only read committed Rust data or enqueue semantic events; they never synchronously call JS. Long lists render a visible range, not all rows. Child content is represented by repeatable weak-owner projections of the committed tree, with generated typed named slots.

Validation will cover the shared bridge (slots, state identity, listener replacement, owner/epoch teardown), representative complex native interactions, generated API completeness and TS consumers, plus a real native Gallery. Ordinary builder wrappers do not each get duplicate implementation-shaped tests.

Progress: implementation complete for the pinned public inventory. The generated SDK contains 144 JSX components/descriptors and 10 native functions; all 138 ordinary upstream rendering interfaces are mapped in [implementation coverage](implementation-coverage.md). Full workspace CI, both vendored native library suites and the continuous renderer regression passed. Native-window checks and the remaining physical-interaction/CUA verification boundaries are recorded in [acceptance](acceptance.md). Public usage is documented in [the component guide](../../docs/gpui-components.md).
