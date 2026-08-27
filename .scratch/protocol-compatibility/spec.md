# Protocol compatibility

Status: resolved

## Scope

The wire protocol is v3. Every Snapshot, Event, Patch, and Command MessagePack
array carries the protocol version at index 0 and its message discriminator at
index 1. The length-prefixed frame layer does not carry a version of its own.

## Baseline audit

| Direction / decoder | Encoder and header | Decoder | Baseline mismatch behavior |
| --- | --- | --- | --- |
| TypeScript receives host Event | `encodeFrame`/`encodePayload` emit `[PROTOCOL_VERSION, EVENT_KIND, ...]` (`packages/react-gpui/src/protocol.ts:11-18, 773-785`) | `decodeEvent` | A valid v4 Event returned `null`; `RootContainer` and `SurfaceHost` converted that to the generic `received malformed event frame` protocol detail. |
| Rust receives renderer Snapshot | `Snapshot::new` sets `PROTOCOL_VERSION`; Snapshot wire is `[version, 1, ...]` (`crates/react-gpui/src/protocol.rs:87-125`, `protocol/wire/snapshot_patch.rs:9-48`) | `Snapshot::decode` | Rejected with `unsupported protocol version 4`. The error named only the received version. |
| Rust receives renderer Patch | `Patch::new` sets `PROTOCOL_VERSION`; Patch wire is `[version, 3, ...]` (`crates/react-gpui/src/protocol.rs:128-190`, `protocol/wire/snapshot_patch.rs:50-89`) | `Patch::decode` | Rejected with `unsupported protocol version 4`. |
| Rust receives renderer Command | Command wire is `[version, 4, ...]` (`crates/react-gpui/src/protocol.rs:233-282`, `protocol/wire/command.rs:6-136,172-402`) | `Command::decode` | Rejected with `unsupported protocol version 4`. |
| Rust Event decoder | Event constructors set `PROTOCOL_VERSION`; Event wire is `[version, 2, ...]` (`crates/react-gpui/src/protocol.rs:566-1192`, `protocol/wire/event.rs:9-235`) | `Event::decode` | **Bug:** accepted a valid v4 Event and returned `Ok(Event { protocol: 4, ... })`; it had no version check. |

The baseline was exercised against real decoders, not inferred from source:

- TypeScript: `bun -e 'import { encodePayload, decodeEvent } from "./packages/react-gpui/src/protocol.ts"; const payload=encodePayload([4,2,1,1,1,1,0,0,1,null]); console.log({decoded:decodeEvent(payload), payload:[...payload]});'` printed `decoded: null`.
- Rust: a temporary binary encoded valid Snapshot, Patch, Command, and Event values, changed the encoded version byte from `3` to `4`, then called each public `decode` method. It printed:

  ```text
  snapshot: err: unsupported protocol version 4
  patch: err: unsupported protocol version 4
  command: err: unsupported protocol version 4
  event: ok
  ```

Thus an old host/new renderer already failed fast for Snapshot/Patch/Command,
but with a low-value error, while an Event decoder could silently accept drift.

## Decision

1. Keep one supported version constant per language: TypeScript
   `PROTOCOL_VERSION = 3` and Rust `PROTOCOL_VERSION: u32 = 3`. No cross-language
   code generation is introduced.
2. Every decoder rejects a valid frame whose version differs from its supported
   version before message-body validation. Rust returns
   `ProtocolError::UnsupportedProtocol { received, expected }` with:

   ```text
   protocol version mismatch: renderer speaks protocol vN; this host binary speaks protocol v3 — update the host binary / pin @react-gpui/core to a v3 release
   ```

   TypeScript raises `ProtocolVersionMismatchError` with the symmetric
   host/renderer wording and both versions.
3. `RootContainer` and `SurfaceHost` catch decoder exceptions and route them
   through the existing `TransportTerminatedError` typed cause path:
   `{ kind: "protocol", detail }`. Pending commands are rejected and later
   events are dropped; no silent return or transport redesign is introduced.
4. Focused tests pin both language constants to v3, prove current encoders emit
   that version, and reject both adjacent versions (`v2` and `v4`) with
   version-bearing diagnostics. Existing cross-language golden bytes remain
   unchanged; no `make protocol-golden-generate` update is needed.

## Rejected alternatives

- **Version negotiation or dual-version decoding:** rejected. This protocol has
  positional arrays and one version per build; accepting two schemas would hide
  incompatible renderer/host releases and create a second compatibility matrix.
  Upgrade both artifacts together or pin the renderer package to the host's
  supported release.
- **Continue returning `null` for a mismatched TypeScript Event:** rejected.
  Callers could only report malformed MessagePack and lose the actionable
  upgrade/pinning remedy.
- **Change the frame prefix to carry a second version:** rejected. The existing
  MessagePack header already carries the version in every message, and changing
  the length-prefixed framing would invalidate all existing vectors and add no
  compatibility capability.

## Verification

- TypeScript focused protocol, SurfaceHost, and RootContainer tests pass,
  including adjacent-version diagnostics and typed protocol termination.
- Rust focused protocol mismatch test passes for Snapshot, Patch, Command, and
  Event decoders.
- Golden fixture files were not changed; their v3 bytes remain the lock.
