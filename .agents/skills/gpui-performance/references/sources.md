# Source Evidence and Scope

Verified on 2026-09-05. These sources explain the rules; reference checkouts are
separate from the version linked by the application.

- Runtime dependency: `gpui-pre 0.3.3` in Cargo.lock. Inspect registry source
  `src/window.rs`, `src/profiler.rs`, `src/view.rs`, and `src/executor.rs`.
- Zed reference commit: `5a9b9558db01a6b906cec2fb70a797affdc58cdd`, available locally
  at `references/zed`.
- Repository measurements: [scroll-performance.md](../../../../docs/scroll-performance.md).

## Primary source: what to inspect and infer

Links pin a commit. Recheck the corresponding functions after upgrades instead of
retaining outdated API examples.

| Entry point | Finding |
| --- | --- |
| [window.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/window.rs): `on_next_frame`, `request_animation_frame`, `draw` | Requesting another frame creates demand. Self-induced redraws cannot establish that the application was already smooth. |
| [profiler.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/profiler.rs): `record_draw_timing`, `record_present`, `FrameDurationSnapshot` | Draw duration and dirty-to-present span different intervals. Present is not display scanout. Derive interval histograms by subtracting cumulative histograms. |
| [view.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/view.rs): `ViewElementCacheKey`, `prepaint` | Cache reuse depends on bounds, mask, text style, and dirty/refresh state. Extracting a component alone does not cache it. |
| [uniform_list.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/elements/uniform_list.rs): `uniform_list`, `measure_item` | Representative-row measurement can determine other rows only when the equal-height constraint holds. |
| [list.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/elements/list.rs): `ListState`, `ListMeasuringBehavior` | Visible-range and all-item measurement have different costs. Persistent state manages scrolling and remeasurement. |
| [context.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/app/context.rs): `notify`, `spawn`, `spawn_in` | notify signals entity changes; spawn receives a weak handle; Tasks need an owned lifetime. |
| [taffy.rs](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/crates/gpui/src/taffy.rs): `compute_layout` | GPUI layout enters Taffy measurement. Profile the actual page to determine nested-flex cost; the entry point alone cannot establish its share of runtime. |

## Existing GPUI skills: adopted and qualified guidance

These skills offer development guidance. Actual source and tests establish runtime
behavior.

- Zed [gpui-bench](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/.agents/skills/gpui-bench/SKILL.md): Adopt production paths, feature isolation, serial measurement, and paired responsiveness/completion metrics. Verify bench APIs, release-fast profiles, and platform support in the target project. This repository's TestAppContext audit does not claim production benchmark qualification.
- Zed [gpui-test](https://github.com/zed-industries/zed/blob/5a9b9558db01a6b906cec2fb70a797affdc58cdd/.agents/skills/gpui-test/SKILL.md): Adopt fixed seeds, GPUI timers, and pending-task diagnostics. Deterministic scheduling verification does not establish actual display behavior.
- AprilNEA [gpui](https://github.com/AprilNEA/gpui-skills/blob/3961d8cb27c345c04b592e79794eff0d278865ed/gpui/SKILL.md) and [gpui-component](https://github.com/AprilNEA/gpui-skills/blob/3961d8cb27c345c04b592e79794eff0d278865ed/gpui-component/SKILL.md): Reviewed notify, Task, background-spawn, flexbox rules, and component/theme indexes. Retain batched notifications, task ownership, and theme consistency. Narrow universal h/v_flex guidance to layouts requiring flexible allocation. Its notify rules also exempt changes with no visible effect; read the conditions along with an “always” heading.
- cnwzhu [gpui-async](https://github.com/cnwzhu/gpui-skills/blob/393a3da0a6e0d379d0e8aaeb7d856951641b47cf/gpui-async/SKILL.md) and [gpui-elements](https://github.com/cnwzhu/gpui-skills/blob/393a3da0a6e0d379d0e8aaeb7d856951641b47cf/gpui-elements/SKILL.md): Retain foreground/background separation, stored Tasks, stable component structure, and theme consistency. Tokio timer examples require that runtime and cannot be transplanted into GPUI deterministic tests. Component extraction improves organization without necessarily reducing element count or layout work.

## Rules established by repository measurements

Kanban's sustained jank came from intrinsic sizing of single-line buttons amplifying
nested measurement. Explicit control sizing improved it and received user acceptance.
Overview improved after replacing an outer flex stack with block layout where no
flexible allocation was needed, also with user acceptance. These are workload-specific
findings, not rules to fix every height or avoid all flex layouts. VirtualList's blank
first frame and invalid ranges after empty/filter updates are covered by native bounds
and actual wire-range tests.

## Skill-writing basis

The entry descriptions, checkable completion criteria, and on-demand references
follow the user-selected public Codex adaptation of
[writing-great-skills](https://github.com/mevatron/mattpocock-skills/blob/139f1166fdbf378db5463fd3aa0d58a68b8e730b/plugins/mattpocock-skills/skills/writing-great-skills/SKILL.md)
and this repository's writing-for-agents guidance. These skills adapt the guidance
without copying lengthy upstream rules or depending on upstream tool commands.
