Only keep the key test cases, Meaningless testing is prohibited.

Avoid excessive fallback.

When encountering design problems, actively refactor rather than compromise and settle.

During development, there's no need to maintain compatibility with old designs or data structures—don't keep any historical wrappers.

Use English for source comments, user-facing text, and maintained documentation.
Keep optional translated documents in explicitly named copies (for example,
`README.zh-CN.md`). Preserve intentional multilingual input and Unicode fixtures.

## Documentation and website synchronization

After every change, including fixes, refactors, and merges, inspect the related
documentation and website content. Update affected guides, API descriptions,
examples, navigation, and capability notes in the same task. Treat this as part of
completion. Use `docs/README.md` to find the authoritative guides and
`examples/website/README.md` to find website content and verification commands.
Regenerate derived API content from its source. Run the relevant documentation
and website checks, then report what was synchronized and any verification limits.
If a change affects neither documentation nor website content, state why.

## Agent skills

### Issue tracker

Issues and specs live as markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the canonical labels `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repo with root `CONTEXT.md` and `docs/adr/`. See `docs/agents/domain.md`.

### Native UI development and performance

For GPUI layout, rendering, lists, tasks, caching, or performance diagnosis, use `.agents/skills/gpui-performance/SKILL.md`. For solid-gpui application development, use `.agents/skills/solid-gpui/SKILL.md`. The repeatable measurement workflow is in `docs/performance-analysis.md`.
