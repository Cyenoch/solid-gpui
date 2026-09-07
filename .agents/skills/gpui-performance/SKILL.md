---
name: gpui-performance
description: Design and diagnose GPUI layout, rendering, lists, tasks, caches, and animations; investigate scrolling, resize regressions, FPS, and input latency. Also read solid-gpui for solid-gpui applications.
---

# GPUI Performance

## Choose a branch

- **Development or refactoring**: Follow steps 1, 2, 4, and 5. Describe work bounds
  and invalidation scope before editing.
- **Existing jank**: Follow every step in order. Select the optimization target
  from measurements.
- **Metric explanation only**: Follow steps 1, 3, and the evidence interpretation
  part of step 5 without changing code.

## 1. Identify the actual environment

Read the target's Cargo.lock, feature graph, build profile, and window entry point.
Distinguish the linked GPUI version from the Zed reference checkout. Verify source
recommendations against the linked implementation. Read the
[source and community guidance](references/sources.md), following only the source
entry points needed for the task.

**Completion criterion**: Record dependency versions/commits, executable, build
configuration, window size/scale, target refresh rate, trigger, and monitor state.
Identify whether the symptom concerns the first frame, sustained scrolling,
post-resize behavior, background-task starvation, or the GPU.

## 2. Design per-frame work and ownership

Read the relevant [development rules](references/development.md).

- Identify repeated work across request_layout → prepaint → paint and describe
  how it grows with content size.
- Assign lifetimes to state, scroll handles, input models, tasks, and subscriptions.
  Notify the corresponding entity only when visible state changes.
- Bound list work by the visible range. Choose layout separately for vertical
  content flow and flexible space allocation.
- Document each cache's key, invalidation conditions, dependencies, and behavior
  during scrolling/resize. A static appearance does not prove cache correctness.

**Completion criterion**: Every new high-frequency path has a work bound. Every
long-lived resource has an owner and release condition. Distinguish size constraints
for multiline content from those for single-line controls.

## 3. Establish a feedback loop that can reject the hypothesis

Reproduce through this repository's
[performance analysis guide](../../../docs/performance-analysis.md); use an
equivalent native event-to-presentation path in other projects.

First verify that the action changes content, then measure. Record CPU frame time,
actual frame cadence, and input-to-present latency separately. Use deterministic
TestAppContext tests for correctness and CPU attribution; production benchmarks
must exclude test-support. Change one factor at a time, keeping window, input,
cache state, and measurement code identical between comparisons.

**Completion criterion**: Retain reproduction commands and baseline artifacts.
Use content displacement or completion state to prove that work was preserved.
Include at least one counterexample capable of rejecting the hypothesis. Preserve
manual steps when actual trackpad input cannot be automated.

## 4. Change and verify the original scenario

Reduce repeated work, narrow invalidation, and improve algorithms before tuning
allocations or small constants. Revert experiments whose hypotheses fail. Preserve
single-line text, multiline content, event order, and all functionality.

Run measurements serially, separate from compilation and other benchmarks. Bound
each profile/benchmark to at most five minutes.

**Completion criterion**: Check the original scenario and narrow/wide/repeated
resize cases. Protect correctness with key tests. Report regressions and machine
load. Performance acceptance preserves buttons, input, and reachable content.

## 5. Deliver evidence and rules

Report baseline/candidate, trigger conditions, metric definitions, raw artifacts,
and verification limits. Fixed time budgets belong in controlled measurements;
ordinary unit tests check work bounds or business invariants.

Record reusable findings in one authoritative reference with conditions and
counterexamples. Keep historical failure logs in incident records.

**Completion criterion**: Distinguish deterministic tests, native measurements,
and user-experience confirmation. CPU timings, idle FPS, interval p95 values, and
GPU-free tests do not establish display dropped-frame rates.
