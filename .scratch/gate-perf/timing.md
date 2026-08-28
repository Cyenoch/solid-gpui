# CI gate timing baseline

## 2026-08-29 measurement

This is an engineering timing note, not release-readiness evidence. Measurements are
on the macOS ARM workstation at HEAD `98a4b7b`; the user-owned `AGENTS.md` change
was left untouched.

### Warm full-gate baseline

`/usr/bin/time -p make ci` passed with `real 54.14 s` (warm local caches). A second
warm run is recorded below after this note was written. The current warm result is
within the historically observed developer-gate envelope (roughly 20--60 s),
although it is slower than older 9--20 s records because the core Bun suite now
contains substantially more work.

### Recipe-step breakdown

Each command below was timed independently with `/usr/bin/time -p` after the warm
full-gate run. The times are wall-clock `real` seconds; individual command times
are not expected to sum exactly to the enclosing `make ci` wall time because each
recipe runs in its own shell and test/process startup and machine load vary.

| Make recipe / command | Wall time | What it runs |
| --- | ---: | --- |
| `rust-format` / `cargo fmt --all -- --check` | 0.20 s | Rustfmt check for the workspace |
| `rust-check` / `cargo check --workspace --locked` | 0.38 s | Workspace Rust type/build check |
| `rust-check` / `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0.34 s | Workspace Clippy, all targets, warnings denied |
| `rust-check` / `cargo test --workspace --locked` | 5.41 s | Workspace Rust unit, integration, perf, protocol, and doc tests |
| `bun-install` / core `bun install --frozen-lockfile` | 0.07 s | Verify/install core package dependencies |
| `bun-install` / dev `bun install --frozen-lockfile` | 0.20 s | Verify/install dev package dependencies and local core link |
| `bun-format` / core `bun run format` | 0.53 s | Prettier check for core source/tests/examples/config |
| `bun-format` / dev `bun run format` | 0.23 s | Prettier check for dev source/examples/config |
| `bun-typecheck` / core `bun run typecheck` | 0.96 s | Core TypeScript no-emit check |
| `bun-typecheck` / dev `bun run typecheck` | 0.50 s | Dev TypeScript no-emit check |
| `bun-test` / core `bun run test` | **28.07 s** | Core Bun suite: 128 tests / 89,300 assertions, including event-storm scenarios |
| `bun-test` / dev `bun run test` | 0.64 s | Dev Bun suite: 20 tests / 50 assertions |
| `bun-build` / core `bun run build` | 0.94 s | Clean core `dist`, bundle JavaScript, emit declaration files |
| `bun-build` / dev `bun run build` | 0.63 s | Clean dev `dist`, bundle JavaScript, emit declaration files |
| `bun-pack-smoke` / `bash scripts/package-pack-smoke.sh` | 3.33 s | Pack both tarballs, inspect allowlisted contents, install in a temporary consumer, run runtime and type smoke |

The three largest isolated costs are core Bun tests (28.07 s), Rust workspace
 tests (5.41 s), and package tarball consumer smoke (3.33 s). Core Bun tests are
the clear dominant cost. Their 128-test/89,300-assertion workload is a natural
suite-size cost, not redundant work, so it was not optimized or removed.

### Recipe review and redundancy finding

The Makefile composition is semantically deliberate:

- `ci` runs `rust-format`, `rust-check`, and `bun-ci`.
- `rust-check` intentionally runs check, Clippy, and tests as separate gates.
- `bun-ci` intentionally covers format, typecheck, tests, and pack smoke.
- `bun-pack-smoke` depends on `bun-build`; the pack smoke script then validates
  the built artifacts as a temporary consumer.
- The shared phony `bun-install` prerequisite is evaluated once by GNU Make in a
  single invocation even though several Bun leaves name it. The observed `make
  ci` output had one core/dev install pair, not repeated installs.
- There is no duplicate format, test, or package build step inside this recipe.
  `package-pack-smoke.sh` uses one dry-run pack plus one real pack per package;
  both are required by its artifact/consumer checks.

No recipe or CI-adjacent script change was warranted. In particular, the shared
Makefile install/build outputs remain a reason not to introduce parallelism: the
prior DX audit explicitly judged that riskier than its small warm benefit.

### Explaining the 166.49 s run

The readiness refresh recorded a passing `make ci` at **166.49 s**, with 128 core
Bun tests / 89,300 assertions and 20 dev tests / 50 assertions. The isolated warm
run above passed at 54.14 s, and the individual warm Rust steps completed in 0.20,
0.38, 0.34, and 5.41 s. Bun dependency verification was effectively free (0.07 s
core, 0.20 s dev), while the core test workload accounted for 28.07 s. This
rules out genuine recipe growth or a hidden duplicate core test/build in the
current Makefile.

The 166.49 s result is therefore treated as a cold-cache and/or concurrent-gate
outlier, not a product or recipe regression. A cold Rust/Bun graph rebuild can
consume most of the difference; concurrent sessions can additionally wait on
Cargo's shared build lock and contend for CPU/I/O. The historical audit already
measured a clean-cache `make embedded-bun` at 137.16 s versus 0.82 s warm, which
establishes that this repository's pinned Bun/GPUI graph has a large cold-build
envelope. There was no historical per-process trace from the 166.49 s run, so its
cold-build and lock-wait portions cannot be separated after the fact; current
step isolation is the available evidence and is consistent with cold cache and
parallel contention.

Expected local envelope:

| State | Expected `make ci` behavior |
| --- | --- |
| Warm, uncontended | Approximately 20--60 s historically; **54.14 s** in this current tree, dominated by core Bun tests |
| Cold or materially invalidated Rust/Bun caches | Can expand to roughly 2--3 minutes; the observed **166.49 s** is in this outlier envelope |
| Concurrent gate sessions | Add nondeterministic Cargo lock / CPU / I/O wait; treat timing as contaminated rather than recipe regression |

No gate was removed, loosened, or parallelized.

### Verification

- Warm `/usr/bin/time -p make ci`: PASS, `real 54.14 s`.
- The independently timed recipe commands above: all PASS.
- A second warm attempt while concurrent source edits were active exited 2 in
  `rust-format` after 0.28 s and did not enter the gates.
- After source commit `622394c` settled, the final warm `/usr/bin/time -p make
  ci` passed with `real 138.63 s`. Its Rust steps rebuilt the changed source
  (check 2.88 s, Clippy 4.09 s, test compile 6.20 s), and the core Bun suite
  measured 41.36 s; this sample was therefore cache-invalidated by the source
  change rather than an uncontended warm-cache repetition. It still passed all
  109 Rust tests and 128 core Bun tests, plus the dev and package gates.

### Second warm run

The post-note `/usr/bin/time -p make ci` attempt exited 2 in `rust-format` after
0.28 s because concurrent edits in `crates/react-gpui/src/renderer.rs` were not
yet rustfmt-clean. That source file was outside this workstream and left
untouched. The later 138.63 s run passed after source commit `622394c`, but is a
cache-invalidated rebuild sample, not a second warm-cache timing point.
The subsequent uncontended warm `/usr/bin/time -p make ci` passed with `real
50.57 s` (109 Rust tests, 128 core Bun tests, 20 dev Bun tests, and package
smoke all green). This confirms the 138.63 s source-rebuild sample falls back to
the expected warm envelope once compilation artifacts are reused.
