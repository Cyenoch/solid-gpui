# Embedded representative coverage

## Seams found

`EmbeddedBunAdapter::start` canonicalizes one TSX entry, then evaluates it on a
Bun/JavaScriptCore runtime thread. The patched Bun wrapper installs the
`process.stdin`/`process.stdout` byte shims and polls bounded native-event and
Fast Refresh queues. Adapter callbacks carry complete framed payloads only;
GPUI objects never cross the FFI boundary. The adapter exposes timeout-aware
commit polling, commit counts, runtime status, and refresh queueing.

The process example audit in
`crates/react-gpui-host/tests/examples_render.rs` starts each entry with Bun,
waits for a Snapshot, applies it to a headless GPUI surface, and checks text and
layout signals. That harness cannot be reused directly: it owns a child process
stdin/stdout reader and sends events through child pipes. Embedded coverage uses
the adapter's public commit seam instead, decoding the same Snapshot payloads.
It deliberately does not duplicate display-backed painting or every process
interaction test.

## Bounded matrix

The feature-gated `react-gpui-host` integration target
`embedded_examples.rs` loads four representative entries serially:

| Entry | Boundary represented | Startup assertion |
| --- | --- | --- |
| `gallery.tsx` | kitchen-sink composition: Image, TextInput, VirtualList, overlays, drag, theme/layout state | `React GPUI Gallery` text |
| `text-input.tsx` | controlled/uncontrolled and multiline input/IME-adjacent setup | `Text input demo` text |
| `virtual-list.tsx` | native list state with a 100,000-item data source and bounded initial range | `Row 0` text |
| `notes.tsx` | asynchronous file-command entry and close-policy setup | `Choose a note to begin` text |

Each case must emit a valid protocol-v3 Snapshot on surface 1, have a bounded
nontrivial node count, contain the expected literal text, remain running while
startup is observed, and shut down with status 0. A nonzero runtime status is
the embedded equivalent of a startup/stderr failure: the adapter's embedded
Bun entrypoint reports evaluation failures on stderr and returns a nonzero
status. The test does not scrape process-global stderr because the embedded
runtime intentionally shares the host stderr stream.

The existing embedded counter test remains the focused native-event proof
(press produces a second commit and shutdown ends the stream). The new target
adds one embedded lifecycle proof: queueing a Fast Refresh import must produce a
new commit without ending the runtime, then the VM must tear down cleanly. The
matrix and lifecycle check are key tests, not a second copy of the process
behavior matrix.

## Gate and budget

`make embedded-bun` runs the host embedded feature check, this host integration
target, and the adapter crate tests. Cases are serial to avoid making global
JSC initialization and multiple VM teardown race; the four entries are small
and the gate is measured before and after the change. A tenfold regression over
the recorded baseline is not acceptable; if the measured cost approaches that
bound, trim the matrix to the smallest distinct boundary set rather than adding
more behavior assertions.

## Boundaries and divergence policy

This target proves renderer startup and the adapter's framed transport/lifecycle
seams. Native GPUI painting, Quartz dialogs, actual IME candidate placement,
and real file-picker UI remain display-backed boundaries. Notes startup proves
that the file-command-bearing module can load under embedded JSC; invoking its
native file commands requires a host command-result loop and is intentionally
not fabricated in this startup matrix. If an entry fails only under embedded,
the test records the reproducible entry/status and the adapter is fixed or the
JSC/platform boundary is documented; no fallback entry or skipped case is
allowed.
