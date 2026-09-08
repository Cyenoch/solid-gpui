# Key Acceptance Scenarios by Change

| Change | Required behavior | Suitable evidence |
| --- | --- | --- |
| Signals/local interactions | Correct values, button actions, and derived results; updates affect only relevant nodes | Actual HostTree commits or application interactions, without mocks that reproduce the implementation |
| Routing/scrolling | Navigation retains position; content scrolls independently; both ends remain reachable; repeated resize does not regress | Native bounds from the routed Gallery and actual trackpad input |
| VirtualList | Real rows appear initially; empty-to-populated, end-of-list filtering, and reordering work; inner/outer scrolling remains independent | Row bounds, valid committed ranges, and actual row text, beyond item counts |
| Single-line control sizing | Text/icons fit within every size/variant and functionality is preserved | Native bounds and light/dark theme screenshots |
| Input | Controlled values synchronize; selection, caret, IME, and blur behavior remain stable | Native input tests and actual input-method scenarios |
| Extension component Patch | Listeners valid in the initial Snapshot stay valid through property updates, replacement, and removal; failures roll back atomically | Actual producer updates and receiver validation together, exposing rule drift between them |
| Native client | Success, domain errors, cancellation/stale results, and handling after closure | Generated wire and actual handler integration tests |
| Lifecycle | Unmount/window changes release timers, subscriptions, and Tasks belonging to old owners | Owner/entity release and active-task checks |
| Performance | Time improves under the same workload, viewport, build, and monitor state | Native sampling, CPU attribution, and functional invariants, reported separately |

For changed types or generated interfaces, run the relevant type checker, generator,
and consistency checker. Ordinary low-risk styling does not need per-property
assertions. The Gallery audit derives all routes from PAGES; adding a page does not
require a second route list.

## Behavioral acceptance prompts for skill maintenance

These scenarios check whether the skill guides the intended behavior. They are not
claims of completed model evaluations.

1. “Jank after resize; give every card a fixed height”: Measure first, distinguish
   single-line control sizing from multiline content, and preserve descriptions.
2. “VirtualList has itemCount 1000 but is blank”: Check boundary/row bounds and the
   range before changing overscan; item count alone cannot establish visibility.
3. “Navigation jumps to the top”: Follow the diagnosis and acceptance criteria in
   [Route changes must preserve the shell](application.md#route-changes-must-preserve-the-shell).
4. “Just put heavy work in async”: Inspect the executor and work per foreground poll.
5. “FPS 120 proves performance is fine”: Distinguish self-induced repainting, actual
   window frame counts, CPU duration, and input latency.
6. “Some light-theme cards have black text on black backgrounds”: Check semantic
   tokens together and inspect actual child Text colors.
7. “Write a new page with useEffect/React JSX”: Follow the current Solid universal
   runtime example.
8. “Add a field to the generated file”: Start from the schema/Rust API definition,
   then run generation and consistency checks.

When changing the skill, explain the expected path for these prompts. If the rules
cannot produce those outcomes, revise triggers or steps instead of adding synonyms
for existing warnings.
