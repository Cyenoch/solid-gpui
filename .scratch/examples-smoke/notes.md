# Examples launch smoke

## Coverage

`scripts/examples-smoke.sh` enumerates every `*.tsx` entry directly under `packages/react-gpui/examples/` and launches each through one release `react-gpui-host` build:

- `counter`
- `drag-reorder`
- `dropdown`
- `focus-flow`
- `gallery`
- `keyboard`
- `multi-surface`
- `notes`
- `rich-text`
- `selectable-text`
- `stress`
- `text-input`
- `todo`
- `virtual-list`

Each run uses `--runtime process -- bun run <entry>` from the examples directory, `REACT_GPUI_LOG=info`, a five-second startup watchdog, SIGTERM teardown with bounded SIGKILL fallback, and a process-group check for leaked members.

## First launch verification

The first direct target run built the release host once, then launched all 14 entries. Every entry emitted the `protocol=v3` startup diagnostic and exited with the expected SIGTERM status (`-15`); every post-teardown process-group check was clean.

| Example | Startup diagnostic | Run elapsed | Verdict |
| --- | ---: | ---: | --- |
| `counter` | 0.016 s | 0.045 s | PASS |
| `drag-reorder` | 0.016 s | 0.043 s | PASS |
| `dropdown` | 0.013 s | 0.040 s | PASS |
| `focus-flow` | 0.016 s | 0.044 s | PASS |
| `gallery` | 0.016 s | 0.042 s | PASS |
| `keyboard` | 0.016 s | 0.043 s | PASS |
| `multi-surface` | 0.016 s | 0.043 s | PASS |
| `notes` | 0.016 s | 0.044 s | PASS |
| `rich-text` | 0.012 s | 0.039 s | PASS |
| `selectable-text` | 0.016 s | 0.043 s | PASS |
| `stress` | 0.016 s | 0.043 s | PASS |
| `text-input` | 0.015 s | 0.041 s | PASS |
| `todo` | 0.016 s | 0.043 s | PASS |
| `virtual-list` | 0.016 s | 0.042 s | PASS |

The first target run took 39.562 s total, including the one-time release build; the launch loop itself took 0.595 s. A warm `make examples-smoke` rerun took 0.93 s wall-clock (0.892 s reported by the harness), with a 0.579 s launch loop.

## Decisions

- Keep `examples-smoke` standalone rather than adding it to `make ci`. The measured cold target was 39.562 s, above the approximately 25 s CI budget threshold, while the warm launch loop is sub-second. The target still builds once and loops over all entries, so local/release verification does not rebuild per example.
- Use a five-second per-example startup watchdog. The observed startup times were 0.012–0.016 s; five seconds is a generous cold-cache floor and preserves the required bounded failure behavior.
- The target does not touch or reuse gallery processes: each host is an independent process group with its own temporary output directory.

## Rot found

No example launch rot was found. No example source changes were needed.
