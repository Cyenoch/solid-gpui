# Serialization Options for `protocol.ts`

## Conclusion

**Do not replace v3 with SCALE merely to switch encoders.** The actual problem is that the wire schema, semantic model, and validation logic are spread across separate TypeScript and Rust implementations; MessagePack is not the bottleneck identified by current measurements.

Separate the decision into two stages:

1. **Keep v3 MessagePack in the short term.** Make `protocol.ts` a deep module: callers use semantic `Snapshot`, `Patch`, `Command`, and `Event` DTOs without constructing or consuming tuple indexes. Separate framing, encoding, structural validation, and domain validation. Preserve the existing positional wire format and golden-vector contract, and add container-length limits to the MessagePack decoder.
2. **Prioritize Protobuf for the next deliberate wire-breaking v4.** Use one `.proto` as the cross-language schema, generate Rust/TypeScript wire bindings, and retain handwritten semantic validators outside them. Protobuf's `oneof`, explicit `optional`, and field-number evolution rules directly address the current polymorphic payloads, sparse Style properties, and tuple maintenance risks.

SCALE makes sense only when Rust is the overwhelmingly dominant endpoint, the protocol remains strictly lockstep over the long term, the TypeScript side accepts maintaining a handwritten/generated codec, and minimum byte size takes priority over evolvability. This repository's TypeScript/Bun renderer is a first-class peer, so SCALE is not recommended as the default.

---

## 1. Current Implementation and Actual Constraints

### 1.1 Wire structure

The repository's current protocol is:

```text
[u32 little-endian payload length][MessagePack payload]
```

`MAX_FRAME_SIZE` and `MAX_FRAME_LENGTH` are both 16 MiB. The four-byte framing layer already handles fragmentation, coalesced frames, truncation, and oversized frames independently of MessagePack's structure. Sources: `docs/protocol.md` §1, `packages/solid-gpui/src/protocol.ts`, and `crates/solid-gpui/src/protocol.rs`.

All four v3 message types are positional arrays:

```text
Snapshot [3, 1, surfaceId, epoch, baseRevision, revision, nodes]
Event    [3, 2, surfaceId, epoch, revision, sequence, nodeId, listenerId, eventType, payload]
Patch    [3, 3, surfaceId, epoch, baseRevision, revision, operations]
Command  [3, 4, surfaceId, epoch, afterRevision, requestId, nodeId, kind, payload]
```

The protocol is currently lockstep: there is no negotiation, dual-version decoding, or resynchronization. New capabilities use optional tails on the existing tuples; `ADR-0011` explicitly rejects retaining historical arities, map-based wire formats, and compatibility branches that fill defaults.

### 1.2 Sources of complexity

`packages/solid-gpui/src/protocol.ts` has 981 lines; the Rust semantic protocol has 1,381 lines, with approximately 3,169 more in `src/protocol/wire/`. The complexity extends beyond MessagePack API calls:

- TypeScript tuple types, encoding construction, and event validators;
- Rust `serde` tuple structs, `#[serde(untagged)]` enums, bidirectional conversion, and separate validators;
- direct tuple-index dependencies in `RootContainer`, `NodeGraph`, and `dispatch.ts`;
- synchronized maintenance of `docs/protocol.md`, golden vectors, and separate Rust/TS test suites.

In particular, `Style` is currently a fixed-length array of approximately 42 entries. `style.ts` encodes unset properties as `null` or default codes; `wire/node.rs` must maintain the same positional field sequence. This provides compactness and the current deterministic shape, but reordering or inserting fields affects the entire protocol.

Polymorphic data also relies on manual tags: `HostProperties`, `PatchOperation`, `EventPayload`, `CommandValue`, and menu items all need discriminators. Rust frequently uses untagged enums to try matching by shape.

### 1.3 Performance evidence

The old TypeScript snapshot benchmark fixture was removed during the frontend transition and the consolidation of key tests. The repository currently has no reproducible encoding-performance baseline. Switching to Protobuf/SCALE therefore cannot be justified simply as faster; any codec migration must first establish a new end-to-end benchmark covering the current Solid renderer, Rust materialization, and GPUI first draw.

### 1.4 A MessagePack risk to address immediately

`decodeWire()` currently passes only `{ useBigInt64: false }`. The Decoder in `@msgpack/msgpack` v3.1.3 defaults `maxStrLength`, `maxBinLength`, `maxArrayLength`, `maxMapLength`, and `maxExtLength` to `UINT32_MAX`. The outer 16 MiB frame limit bounds actual payload bytes, but it does not impose semantic limits on array/map element counts. While retaining MessagePack, explicitly configure container limits consistent with the protocol and keep field and domain validators.

---

## 2. MessagePack: Retain It Behind a Deep Module

### 2.1 Facts from primary sources

- MessagePack is a general-purpose object format with native integer, nil, boolean, float, string, binary, array, map, and extension types. An array is a length-prefixed sequence; a map is a collection of key/value pairs. Specification: <https://github.com/msgpack/msgpack/blob/master/spec.md>.
- MessagePack integers and float32/float64 have different wire forms. `@msgpack/msgpack`'s `forceFloat32` affects only noninteger numbers and cannot force integers to encode as floats. Accordingly, repository `ADR-0003` requires producer-byte stability, cross-language semantic equivalence, and permitted numeric dual forms, rather than identical bytes everywhere.
- `rmp-serde` maps Rust tuples/structs to MessagePack. `Vec<u8>` needs binary configuration/`serde_bytes`; otherwise, it may become an array. Source: <https://docs.rs/rmp-serde/latest/rmp_serde/>.
- The current JS runtime supports TypeScript/Bun. Its v3.1.3 README and Decoder implementation document `forceFloat32`, single-object decoding, `decodeMultiStream`, and maximum-container options. Sources: <https://raw.githubusercontent.com/msgpack/msgpack-javascript/v3.1.3/README.md>, <https://raw.githubusercontent.com/msgpack/msgpack-javascript/v3.1.3/src/Decoder.ts>.

### 2.2 A better MessagePack interface

Keep wire tuples out of `RootContainer`. The external boundary can be reduced to:

```ts
export interface ProtocolCodec {
  encodeFrame(message: OutboundMessage): Uint8Array;
  push(chunk: Uint8Array | ArrayBuffer): InboundMessage[];
}

type OutboundMessage = SnapshotMessage | PatchMessage | CommandMessage;
type InboundMessage = EventMessage;
```

`SnapshotMessage`, `PatchMessage`, `CommandMessage`, and `EventMessage` are ordinary immutable DTOs with a `kind` field; `ProtocolCodec` maps them internally to the current positional MessagePack format. `FrameDecoder` should be an independent `FrameCodec`; it and the MessagePack validator should not know each other's implementation details.

Use the following internal modules instead of continuing to expand a 981-line file:

- `protocol/constants.ts`: versions, message/event/command codes, and resource limits;
- `protocol/types.ts`: semantic DTOs and discriminated unions;
- `protocol/codec-msgpack.ts`: tuple ↔ MessagePack mapping;
- `protocol/validate.ts`: structural, resource, numeric, and combination invariants;
- `protocol/frame.ts`: four-byte LE framing.

This changes code organization without changing the v3 wire format. Callers migrate only to DTO constructors/decoder results; do not retain aliases or wrappers for the old tuples.

### 2.3 Benefits and drawbacks

**Benefits:** No new cross-language schema toolchain; preservation of the verified Bun/Rust runtime, framing, and golden vectors; low migration risk; immediate removal of extensive index-based access in `dispatch.ts`, `nodes.ts`, and `root-container.ts`.

**Drawbacks:** With positional tuples, field positions remain the wire contract, and Rust and TS still each need a wire mapping. DTOs improve the boundary and locality without automatically generating a cross-language schema. Replacing tuples with MessagePack maps would enable key-based evolution, but adds key overhead to every node and conflicts with the current v3 decision in `ADR-0011`. Treat that as a v4 design, not a local refactor.

**Decision:** For current v3, this is the first step with the greatest benefit and lowest risk. It is not the final answer for a long-term cross-language schema.

---

## 3. Protobuf: The Preferred Schema-Driven Direction for v4

### 3.1 Facts from primary sources

- The Protobuf wire format consists of key/value records identified by field number and wire type. Wire types let older parsers skip unfamiliar new fields. Strings, bytes, and embedded messages are length-delimited; `float` uses fixed32; `uint32` uses varint. Official encoding guide: <https://protobuf.dev/programming-guides/encoding/>.
- Proto3 field numbers cannot change once used. Deleted field numbers/names must be reserved and never reused. Explicit `optional` provides presence, and the official guide recommends it for compatibility; repeated scalars are packed by default. Official language guide: <https://protobuf.dev/programming-guides/proto3/>.
- `oneof` natively represents mutually exclusive polymorphic fields, fitting `EventPayload`, `PatchOperation`, `Command`, `HostProperties`, and `CommandValue` without probing tuple shapes.
- Protobuf binary serialization **is not canonical**: fields may appear in different orders, and unknown fields or deterministic serialization do not establish one globally unique byte representation. Official explanation: <https://protobuf.dev/programming-guides/serialization-not-canonical/>. After migration, preserve the golden contract of stable bytes per producer plus semantic equivalence across both endpoints; do not blindly require every TS/Rust byte to remain identical forever.
- TypeScript can use Buf's `@bufbuild/protobuf` and `protoc-gen-es`. The project advertises pure TypeScript, Bun support, protobuf conformance, and a standard protoc plugin. Source: <https://github.com/bufbuild/protobuf-es>. Google's official `google-protobuf` JS runtime is less suitable than protobuf-es for the current ESM/Bun integration; bundle/build smoke tests are still required before selecting a runtime.
- Rust `prost` generates Rust structs/enums from proto2/proto3 and supports `encoded_len`, but `prost-build` normally requires `protoc`. It does not provide runtime reflection. Sources: <https://docs.rs/prost/latest/prost/>, <https://docs.rs/prost/latest/prost/trait.Message.html>.

### 3.2 Suggested v4 schema shape

Preserve the existing outer frame. Do not add a second length prefix with Protobuf's `encodeDelimited`:

```text
[u32 little-endian payload length][protobuf Envelope bytes]
```

Conceptual schema:

```proto
syntax = "proto3";
package solid_gpui.protocol.v1;

message Envelope {
  uint32 protocol_version = 1;
  oneof body {
    Snapshot snapshot = 2;
    Event event = 3;
    Patch patch = 4;
    Command command = 5;
  }
}

message Snapshot {
  uint32 surface_id = 1;
  uint32 epoch = 2;
  uint32 base_revision = 3;
  uint32 revision = 4;
  repeated Node nodes = 5;
}

message Patch {
  uint32 surface_id = 1;
  uint32 epoch = 2;
  uint32 base_revision = 3;
  uint32 revision = 4;
  repeated PatchOperation operations = 5;
}

message PatchOperation {
  oneof operation {
    Node create = 1;
    UpdateNode update = 2;
    MoveNode move = 3;
    DeleteNode delete = 4;
  }
}
```

Use oneof for `Node`'s `HostProperties`. Give `Event` a oneof payload, combining the duplicated discriminators currently carried by `eventType` and the payload. Represent each `Command` kind as a oneof message with its own parameters. Give each `Style` property an explicit optional field instead of a fixed 42-slot array. Use `bytes` for images; `uint32` for every ID, epoch, revision, sequence, and index; and `float` for geometry and style floating-point values.

Do not mechanically remove the existing `change_mask` from Patch's `UpdateNode`. The current `mask + null` combination means clearing a field; protobuf field presence alone cannot distinguish unchanged from explicitly cleared. The first version can retain the mask, requiring strict agreement between it and optional-field/clear semantics. A more thorough v5 shape could define `oneof { value; clear; }` for each clearable field.

### 3.3 Validation that Protobuf cannot own

Protobuf handles wire types/basic structure, not the current domain contract. Handwritten validators remain necessary after decoding, including at least:

- exact `protocol_version` matching;
- `revision > base_revision`, and equality between a Patch base and the retained tree revision;
- Surface, epoch, node/listener identity, and event sequence;
- Node parent/index topology, RawText/Text hierarchy, and host-property kind matching;
- finite/non-negative geometry, business ranges within u32, and Style values;
- UTF-8/byte limits for tooltips, paths, text, images, and files;
- node/listener constraints for root events, close requests, focus/blur, and command results;
- combination constraints between each oneof variant and current command/event semantics.

A Protobuf parser's default length capacity is not a business limit. Reject frames over 16 MiB first, then constrain strings, bytes, repeated counts, and nesting depth in the validator/reader. The official Protobuf C++ `CodedInputStream` documentation likewise treats total bytes, nested push limits, and recursion limits as resource controls: <https://protobuf.dev/reference/cpp/api-docs/google.protobuf.io.coded_stream/>.

### 3.4 Benefits, costs, and risks

**Benefits:** One `.proto` removes duplicated schema definitions in TS tuple types and Rust `wire/*`. `oneof` removes untagged shape ambiguity. Optional fields encode only present Style properties. Field-number/reserved rules support actual additive evolution. Floats are directly fixed32, removing the current integer-form f32 dual-form issue.

**Costs:** Adds `protoc`/Buf, a generated-file policy, Rust build.rs, a TS package/runtime, dependency-license review, and CI reproducibility requirements. Generated DTOs may not suit direct exposure to `RootContainer`, so a semantic adapter is still needed. Protobuf decoding still constructs message objects, strings, and vectors; zero allocation cannot be promised. New v4 fixtures and migration of all producers/consumers are required.

**Unknown-field policy:** Protobuf wire/schema supports skipping unknown fields, but retaining them through decode/re-encode depends on the runtime. Both Solid GPUI endpoints are terminal consumers, not proxies, so discarding unknown fields is acceptable by default. A future transparent proxy would require an explicit choice of runtime with unknown-field preservation and a separate contract. Do not equate Protobuf's unknown-field support with automatic preservation in every Rust/TS runtime.

**Version policy:** Current v3 is lockstep, so switching to Protobuf must explicitly advance the version to v4. A decoder must not silently accept both MessagePack v3 and Protobuf v4. To gain Protobuf's evolution benefits, establish protocol-major + additive-schema-change + capability-gate rules after v4, permanently reserving deleted fields.

**Decision:** If the team accepts a formal v4 cutover, Protobuf is the best schema-driven default for this repository. It first addresses maintenance and evolution, with smaller sparse Style payloads as a secondary potential benefit.

---

## 4. SCALE: Compact, but Not a Better Cross-Language Schema

### 4.1 Facts from primary sources

- `parity-scale-codec` explicitly states that both decoder endpoints must independently know the type context. Encoded bytes contain no field names, type identifiers, or schema metadata. Source: <https://github.com/paritytech/parity-scale-codec>.
- SCALE structs/tuples concatenate fields in declaration order; Vec starts with a compact length; Option uses `00`/`01 + value`; enums start with a variant index; integers are little-endian by default. Type order and enum indexes are the wire contract. Polkadot data encoding reference: <https://docs.polkadot.com/reference/parachains/data-encoding/>.
- Rust `parity-scale-codec` provides `Encode`, `Decode`, `Compact`, `DecodeLimit`/memory limits, and related capabilities. Sources: <https://docs.rs/parity-scale-codec/latest/parity_scale_codec/>, <https://docs.rs/parity-scale-codec/latest/parity_scale_codec/trait.DecodeLimit.html>.
- SCALE has no Protobuf-style field tags or unknown-field skipping. Adding fields, reordering structs, changing enum indexes, or changing compact/fixed choices requires explicit versioning or synchronized endpoints. Older decoders cannot reliably skip unfamiliar content field by field.

### 4.2 Interface shape in this repository

The external boundary could be:

```ts
interface ProtocolCodec {
  encodeFrame(message: OutboundMessage): Uint8Array;
  push(chunk: Uint8Array | ArrayBuffer): InboundMessage[];
}
```

Internally, Rust would use `Encode/Decode` and TypeScript a SCALE combinator/generated codec. `RootContainer` would not interact directly with `Compact`, `Option`, or readers/writers. Choose fixed/compact encoding per numeric field: fixed u32 uses more space than current small MessagePack integers; compact encoding saves space for small IDs/lengths, but every TS/Rust field choice must match exactly. Fixed width suits f32, colors, and geometry; strings/bytes/vectors still require length checks before allocation.

`scale-info` can describe Rust types, but turning it into a reviewable, reproducibly generated TypeScript codec requires a custom generator. It is not a cross-language IDL/official TS generation pipeline this project can directly adopt. That replaces the current two wire mappings with three independently verifiable pieces: Rust types, a custom metadata generator, and a TS runtime.

### 4.3 Benefits and drawbacks

**Benefits:** Concise Rust derives, compact structures without field tags, controllable LE/fixed/compact choices, and strong byte-size/CPU benefits in strictly lockstep Rust-first systems.

**Drawbacks:** Not self-describing; no unknown-field skipping; enum indexes/field order form a risky implicit contract. TypeScript/Bun lacks a unified, widely adopted cross-language generation path comparable to Protobuf `.proto` + protoc. Sparse Style represented directly with Option still needs one presence byte per field; using a bitmask reintroduces manual schema design.

**Decision:** SCALE fits codecs within Rust/blockchain ecosystems. It does not solve this project's cross-language protocol schema drift. Select it only if the objective explicitly becomes Rust-first, permanently lockstep, and minimum bytes.

---

## 5. Other Candidates

### FlatBuffers

Official FlatBuffers documentation promises multilingual code generation, direct access to serialized buffers, and forward/backward schema evolution for tables: <https://flatbuffers.dev/>, <https://flatbuffers.dev/schema/>. This is attractive for large Snapshots; Rust/TS also have official language pages and generators: <https://flatbuffers.dev/languages/typescript/>, <https://flatbuffers.dev/languages/rust/>.

However, it requires a `flatc` schema/build pipeline; Patch operations need unions/wrapper tables; the TS object API unpacks/allocates again. FlatBuffers has no built-in wire-format version, and binary field order should not be treated as canonical bytes. Snapshots must still enter the Rust retained tree, so zero-copy does not automatically remove materialization costs. **A FlatBuffers spike is worthwhile only when profiles clearly identify Snapshot decoding/materialization as the bottleneck; it is not the current default.**

### Cap'n Proto

Cap'n Proto provides strongly typed ordinal evolution and zero-copy, but its stream framing uses segment counts/word sizes rather than the current four-byte total-payload prefix. The official other-language list also lacks a mature path that treats Bun TypeScript as a first-class target. Sources: <https://capnproto.org/language.html>, <https://capnproto.org/encoding.html>, <https://capnproto.org/otherlang.html>. **Not recommended.**

### CBOR / JSON / Rust-only codecs

CBOR or JSON can change the representation without automatically solving current schema duplication, tag/variant modeling, or domain validation. Rust-centric codecs such as `bincode`/`rkyv` are even less suitable for a TypeScript/Bun peer. Without an external interoperability requirement, replacing working MessagePack with another generic format is not worthwhile.

---

## 6. Decision Matrix

| Dimension | Current MessagePack v3 | MessagePack DTO boundary | Protobuf v4 | SCALE | FlatBuffers |
|---|---|---|---|---|---|
| Cross-language schema | Written separately in TS/Rust | Still separate, but concentrated in wire adapters | One `.proto` + generated bindings | Rust types + custom TS generation/codec | `.fbs` + `flatc` |
| Polymorphism | Manual tags/untagged shapes | Same wire; callers do not see tags | `oneof` | Enum indexes/manual | Union/table |
| Sparse Style | Fixed slots, many null/default values | Unchanged | Optional fields, naturally omitted | Requires bitmask/Option design | Table offsets |
| Evolution | Currently lockstep; tail rules | Currently lockstep | Field numbers/reserved/unknown skipping | Versioned; no unknown skipping | Additive tables, but extra version identification needed |
| Bun/TS readiness | Verified | Verified | protobuf-es/Buf feasible; new toolchain needed | Fragmented codec choices; requires building/adopting a solution | Official generator feasible; heavier builder needed |
| Rust readiness | Verified | Verified | prost feasible; protoc/build.rs needed | First-class | Feasible |
| Frame reuse | Yes | Yes | Yes; retain the outer 4-byte LE frame | Yes | Yes, with schema/version managed separately |
| Current need | — | **Worth doing immediately** | **Preferred long-term v4 option** | Insufficient | Profile-driven only |

---

## 7. Recommended Implementation Sequence

### Stage A: Preserve the v3 wire format

1. Define semantic DTOs and a `ProtocolCodec` boundary; stop tuple-index access in `RootContainer`, `SurfaceHost`, `dispatch.ts`, and `NodeGraph`.
2. Separate framing, MessagePack mapping, structural validation, and domain validation; preserve the three-layer golden contract in `ADR-0003`.
3. Configure maximum string/bin/array/map/ext options for `@msgpack/msgpack`. If nesting-depth limits are needed, implement them separately in protocol validation/parsing. Domain validators still own resource-byte limits.
4. Add focused `ProtocolCodec` tests: representative Snapshot/Patch/Command/Event DTOs, malformed oneof/tags, frame fragmentation/coalescing, resource limits, and cross-language golden vectors.

### Stage B: Run a Protobuf spike for v4

1. Create one schema file, initially covering only Snapshot, Patch, Command, and Event. Keep generated types behind the renderer/host domain boundary.
2. Measure real fixtures: a 20k mixed snapshot, sparse Style, a style-only patch, pointer/keyboard event storms, and maximum image/file boundaries. Record payload size, encode/decode CPU, TS allocations/GC, and Rust allocations.
3. Choose reproducible codegen for `@bufbuild/protobuf`/`protoc-gen-es` and `prost`; confirm Bun ESM, Cargo builds, licenses, and CI do not rely on undeclared tools on a user's machine.
4. Retain the outer 4-byte LE frame; do not add an inner length delimiter to the Protobuf payload.
5. Design `UpdateNode` clear semantics, unknown-field policy, and version/capability policy; permanently reserve deleted fields.
6. Perform a single v4 cutover: update host/package, documentation, fixtures, and golden tests. Do not retain dual v3/v4 decoders or historical wrappers.

### Stage C: Compare FlatBuffers only when supported by profiles

If the Protobuf spike demonstrates that large-Snapshot decoding/materialization remains an actual bottleneck, compare FlatBuffers with the same fixture. If GPUI first draw dominates instead, do not introduce a third schema toolchain.

## Source Index

- MessagePack specification: <https://github.com/msgpack/msgpack/blob/master/spec.md>
- MessagePack JS v3.1.3: <https://raw.githubusercontent.com/msgpack/msgpack-javascript/v3.1.3/README.md>
- Protobuf encoding: <https://protobuf.dev/programming-guides/encoding/>
- Protobuf proto3 language/evolution: <https://protobuf.dev/programming-guides/proto3/>
- Protobuf non-canonical serialization: <https://protobuf.dev/programming-guides/serialization-not-canonical/>
- Protobuf resource limits: <https://protobuf.dev/reference/cpp/api-docs/google.protobuf.io.coded_stream/>
- protobuf-es: <https://github.com/bufbuild/protobuf-es>
- prost: <https://docs.rs/prost/latest/prost/>
- parity-scale-codec: <https://github.com/paritytech/parity-scale-codec>
- SCALE data encoding: <https://docs.polkadot.com/reference/parachains/data-encoding/>
- FlatBuffers overview/schema/languages: <https://flatbuffers.dev/>, <https://flatbuffers.dev/schema/>, <https://flatbuffers.dev/languages/typescript/>, <https://flatbuffers.dev/languages/rust/>
- Cap'n Proto language/encoding/other languages: <https://capnproto.org/language.html>, <https://capnproto.org/encoding.html>, <https://capnproto.org/otherlang.html>

## 8. Local Verification Record

- `cargo test -p solid-gpui --lib tests::protocol`: 11 passed, 0 failed; covers the current MessagePack round trip, frame truncation/oversize, version mismatch, host-property mismatch, command results, and event payload contracts.
- `bun run task package-test`: 6 passed, 0 failed; covers signal Patches, cross-root ownership, conditional mounting, VirtualList remounting, render-error recovery, and surface-close cleanup.
- This investigation changed no production source. This report was the only repository file added.

---

## 9. Expanded Candidate Set

This section broadens the candidates to formats with schemas/codegen that might support Rust + TypeScript. It evaluates this repository's actual hot path: TypeScript produces a Commit Batch, then Rust parses it and materializes the retained Host Node tree. It does not merely compare isolated codec benchmarks.

### 9.1 Schema-first candidates

| Option | One schema generating Rust + TS | Performance/memory model | Main gaps |
|---|---|---|---|
| Protobuf | `prost` + `protobuf-es/protoc-gen-es` | Tags/varints/fixed32; omitted sparse fields; parsing usually allocates objects/strings | Requires protoc/Buf/build.rs; still needs a semantic adapter |
| Bebop | Official `bebopc` generates Rust and TypeScript from a `.bop` schema | Fixed-width LE numbers, length-prefixed messages/unions, generated encode/decode; theoretically well suited to frequent small messages | Much smaller ecosystem than Protobuf/FlatBuffers; bounded decode APIs, compatibility policy, and long-term maintenance need empirical verification |
| Apache Fory | Official Fory IDL/compiler generates Rust and JavaScript/TypeScript from a `.fdl` schema | xlang, field metadata, unions/optional fields, generated serializers; runtime has graph/depth/container limits | Compiler/JS runtime currently on an alpha line; needs type registration/compatible mode; Bun and node-gyp/dependency paths need verification; heavier runtime than Bebop/Protobuf |
| FlatBuffers | Official `flatc --rust --ts` | Generated accessors read directly from buffers; suits large Snapshots | TS writes require a builder; heavier Patch unions/wrappers; zero-copy benefits decrease after materialization |
| Apache Thrift | Apache compiler Rust and `ts`/`node.ts` targets; Rust/some JS targets support Compact, while the official node.ts matrix primarily supports Binary | Field IDs; structs/unions/optional fields; unknown-field skipping | No `uint32`/`float`, only signed integers and `double`; cannot assume Compact support on TypeScript; runtime is oriented toward RPC/Node |
| Cap'n Proto | Rust plugin + community TS implementation | Word-aligned zero-copy, ordinal evolution | No first-class Bun/TS generator in the official core project; separate segment framing; insufficient TS implementation maturity |
| ASN.1 + RASN | `rasn-compiler` generates Rust; TS backend primarily generates JER type definitions | APER/UPER/OER can be very compact | Rust/TS do not share one mature binary-binding pipeline; excessive ASN.1 complexity |
| Avro | Apache provides schema resolution and a Rust SDK | Records follow writer-schema order; strong schema resolution | Raw data depends on schema; Apache's main project has no first-class TS generator/runtime suited to Bun |

Apache Thrift's official type system and IDL explicitly provide structs, unions, optional fields, and binary, with field IDs in the protocol. Its Compact protocol also defines compact varints and unknown-field skipping, but the official language matrix lists only Binary, not Compact, for `node.ts`. Thrift's basic types omit unsigned integers and 32-bit floats: <https://thrift.apache.org/docs/types.html>, <https://thrift.apache.org/docs/idl>, <https://raw.githubusercontent.com/apache/thrift/master/doc/specs/thrift-compact-protocol.md>, <https://raw.githubusercontent.com/apache/thrift/master/LANGUAGES.md>. Thrift therefore remains a feasible schema/codegen alternative, but does not meet this project's combined requirements for native `u32`/`f32`, minimal glue, and high performance; it does not enter the final top three.

Bebop's official documentation shows that `bebopc` generates TypeScript and Rust from `.bop` schemas, with service artifacts disabled if only records are needed. Its wire format uses little-endian fixed-width numbers, `uint32` lengths, message field indexes, and union discriminators; an unknown message field can be skipped by advancing to the end of the body. Sources: <https://bebop.sh/guide/getting-started-typescript/>, <https://bebop.sh/guide/getting-started-rust/>, <https://bebop.sh/reference/wire-format/>, <https://bebop.sh/reference/union/>. This fits current `u32`/`f32` and high-frequency messages better than Thrift, but ecosystem, toolchain, and bounded decoding must be hard acceptance criteria for a v4 spike. General performance claims in its documentation are not sufficient evidence.

Apache Fory's official IDL documentation explicitly supports generating Rust and JavaScript/TypeScript from one schema, including optional fields, unions, cross-language type registration, and schema-compatible mode: <https://fory.apache.org/docs/compiler/>, <https://fory.apache.org/docs/compiler/generated-code/javascript/>, <https://fory.apache.org/docs/compiler/generated-code/rust/>, <https://fory.apache.org/docs/object-serialization/javascript/schema-evolution/>. It is a stronger additional candidate than Thrift, but the compiler/JavaScript runtime's current alpha status, runtime registration/metadata cost, and Bun compatibility require a spike first.

The ASN.1 `rasn-compiler` README explicitly distinguishes Rust bindings from TypeScript definitions “for JER-encoded ASN.1 data elements”: <https://docs.rs/crate/rasn-compiler/latest/source/README.md>. This does not directly meet the project's single-source binary-codec requirement for both endpoints. Avro's official specification focuses on schema-dependent data and writer/reader schema resolution: <https://avro.apache.org/docs/1.11.0/spec.pdf>. It has strong field evolution, but is not the preferred low-glue choice for Bun + Rust.

### 9.2 Binary format + schema/validation candidates

| Option | Facts | Why it is outside the top three |
|---|---|---|
| CBOR + CDDL | IETF standards; CBOR is extensible and self-describing; CDDL describes CBOR/JSON | CDDL is schema notation, not a unified cross-language binding generator; requires separate choices for CBOR runtime, Rust typegen, and TS typegen/validator |
| MessagePack + custom schema generator | Current runtime is verified; arrays are compact | A custom generator becomes the project's own compiler; maintenance costs and risks exceed the benefit of retaining the existing DTO boundary |
| SCALE | Lightweight, compact, LE on Rust | Decoder needs external type context; field order/enum indexes lack tag-based evolution; TS implementations and codegen are fragmented |
| SBE | Suits fixed-layout, low-latency financial messages; schema-driven | Official SBE tool targets include Java/C++/C/Golang and others, but lack the mature TS target this repository needs |
| Rust `rkyv` / `postcard` / `bincode` | Rust-to-Rust can be fast or zero-copy | No TypeScript wire target; directly violates the single-source requirement for both endpoints |
| WIT / Component Model | Rust and JavaScript component bindings exist | A component interface/runtime, not a generic codec that directly replaces current Bun stdio payloads; would change the Runtime Adapter and process model |
| Arrow IPC | Official columnar IPC with FlatBuffers metadata; supports high-throughput zero-copy batches | Host Nodes form a heterogeneous tree; Command/Event/Patch semantics do not fit columns; would add data rearrangement |
| Postcard + `postcard-bindgen` | Rust `no_std`/varint; community generator can emit JS | Generator is not Postcard's official cross-language pipeline; output is not a first-class TypeScript API; custom attributes and bounded decoding still need review |

CBOR targets small code, reasonable message size, and extensibility, but RFC 8949 explicitly defines it as a generic data format. CDDL RFC 8610 is a description language. Together they do not provide the official single-source Rust/TypeScript generation pipeline needed here: <https://www.rfc-editor.org/rfc/rfc8949>, <https://www.rfc-editor.org/rfc/rfc8610>. The SBE tool documentation likewise omits TypeScript from its target-language list: <https://github.com/aeron-io/simple-binary-encoding/wiki/Sbe-Tool-Guide>. These approaches can work, but recreate glue code.

Arrow IPC's official format is columnar data exchange with FlatBuffers metadata and body buffers: <https://arrow.apache.org/docs/format/Columnar.html>. It suits homogeneous array batches, not the current heterogeneous Host Node tree. Postcard's official README positions it as a Rust/Serde/no_std codec; `postcard-bindgen` is a separate community generator and should not be treated as Postcard's official cross-language contract: <https://github.com/jamesmunns/postcard>, <https://github.com/teamplayer3/postcard-bindgen>.

### 9.3 Performance boundaries of Cap'n Proto and FlatBuffers

The official FlatBuffers TypeScript documentation explicitly distinguishes two APIs: the basic API accesses `ByteBuffer` directly, while the object API unpacks/packs ordinary objects at an efficiency cost. Official `flatc` supports `--rust`, `--ts`, schema conformance, and size-prefixed buffers: <https://flatbuffers.dev/languages/typescript/>, <https://flatbuffers.dev/flatc/>. This makes it a strong Snapshot-read-performance candidate, but does not make TypeScript Commit Batch construction free.

Cap'n Proto encodes a segment tree, with language evolution based on field ordinals. Its official documentation and language list provide no first-party path comparable to this repository's Bun/TypeScript combination: <https://capnproto.org/encoding.html>, <https://capnproto.org/language.html>, <https://capnproto.org/otherlang.html>. Community `capnp-es` is a possible experiment, but its maturity and maintenance risks do not meet the hard requirements for code quality and minimal glue.

---

## 10. Three Final Recommendations

### Option 1: Protobuf v4 for the entire protocol (preferred)

**Priority:** Balance code quality, evolvability, performance, and minimal glue.

- One `protocol.proto`;
- Rust: `prost` + `prost-build`;
- TypeScript: `@bufbuild/protobuf` + `protoc-gen-es`;
- preserve the current four-byte LE outer frame;
- `Envelope.oneof { snapshot, patch, command, event }`;
- use `oneof` for `HostProperties`, `PatchOperation`, `EventPayload`, and `CommandValue`;
- use explicit `optional` for `Style`, avoiding nulls in 42 fixed slots;
- keep generated bindings at the protocol boundary; `RootContainer` must not depend on protobuf-specific APIs;
- semantic validators continue to own tree topology, revisions, surface generations, and resource limits.

**Performance assessment:** Tags/varints/fixed32 fit small Events/Commands. Omitting unset sparse Style fields may substantially reduce Snapshot/Patch sizes, but protobuf decoding materializes message objects and cannot promise FlatBuffers-level zero-copy.

**Main risks:** Codegen toolchain, `UpdateNode` clear semantics, unknown-field policy, and whether generated output is committed. A clean v4 cutover is required; the v3 MessagePack decoder must not also guess protobuf.

### Option 2: FlatBuffers, schema-first across the protocol (Snapshot performance priority)

**Priority:** Decoding, memory use, and read latency for large Snapshots/Patches.

- One `protocol.fbs`;
- generate both endpoints' bindings with `flatc --rust --ts`;
- read Snapshots through the basic accessor API, keeping `--gen-object-api` out of the hot path;
- represent Node/Style/Command/Event with tables and PatchOperation/Event payloads with explicit union wrappers;
- retain the four-byte LE outer frame, file identifier/version, and verifier;
- if the Rust host must ultimately materialize a retained tree, benchmark the total cost of direct reads plus cloning into NodeStore, not only accessors.

**Performance assessment:** For read-only large buffers, this has the strongest chance among the top three of minimizing allocations and decode CPU. This project's Rust renderer needs to own the retained tree, so benefits depend on delaying or reducing materialization.

**Main risks:** The TypeScript builder API is more cumbersome than protobuf DTOs. String/vector offsets, union wrappers, and lifetime rules add adapter glue. Small events/commands may not benefit. If actual profiles remain dominated by GPUI draw, theoretical zero-copy does not justify introducing it.

### Option 3: Bebop, schema-first across the protocol, as a high-performance challenger

**Priority:** One `.bop` schema, generated encoders/decoders, fixed-width numbers, and minimal runtime abstraction, with acceptance of ecosystem-size and long-term-maintenance risks.

- One `.bop`;
- generate Rust and TypeScript together with `bebopc`;
- disable service/client/server generation, retaining only records;
- use `message` for evolvable fields and `union` for Event/Command/Patch polymorphism;
- use the current four-byte LE outer frame; Bebop messages already have a body length, but it must not replace the existing transport frame;
- use `uint32`, `float32`, and `bytes`/string directly, avoiding Thrift's lossy type compromises;
- keep generated record APIs out of the Solid domain; adapt only at the protocol boundary;
- independently limit all array/map/string/bytes/body lengths, union nesting, and recursion.

**Performance assessment:** Bebop's schema-driven generated path avoids generic reflection. Fixed-width little-endian numbers and length prefixes suit current u32/f32 and frequent Events; message body lengths allow unknown-field skipping. It is a reasonable performance challenger to Protobuf, but this repository has no measurements yet. The project's cross-format benchmark claims cannot be treated as a conclusion here.

**Main risks:** Ecosystem and toolchain are much smaller than Protobuf/FlatBuffers. Rust's generated directory is produced automatically during builds, while TypeScript needs separate generator configuration. Bun ESM, generator-version pinning, Cargo reproducibility, bounded decoding of malformed input, and unknown-union-branch behavior all require verification. If these checks fail, retain Bebop as a research candidate outside the production top three.

---

## Ranking Decision

| Rank | Option | Strengths | Unacceptable conditions |
|---:|---|---|---|
| 1 | Protobuf v4 | Best overall balance; most complete schema/codegen, oneof, optional, and evolution rules | Unwillingness to introduce a protoc/Buf/codegen pipeline |
| 2 | FlatBuffers | Greatest CPU/memory performance potential for large Snapshot reads | Unacceptable builder/union glue, or profiles showing that materialization/draw is the bottleneck |
| 3 | Bebop | Generated Rust/TS, fixed-width numbers, message/union length boundaries, low reflection overhead | Small ecosystem; bounded decoding, Bun ESM, and long-term toolchain stability unverified |

**Practical choice for the current repository:** Continue with MessagePack v3 and first refactor the DTO/protocol boundary. If a formal switch is chosen, prioritize a Protobuf v4 spike. Compare FlatBuffers if profiles identify Snapshot decoding/materialization as the main bottleneck; compare Bebop for a smaller generated codec and fixed-width high-frequency messages. Thrift Binary remains a backup for external Thrift interoperability needs, outside the top three.

---

## 11. Local Toolchain Feasibility

- The current workstation has `/opt/homebrew/bin/protoc`, reporting `libprotoc 36.0`.
- `foryc`, `bebopc`, `flatc`, `thrift`, and `capnp` are not currently on PATH. Spikes for Fory, Bebop, FlatBuffers, Thrift, and Cap'n Proto require additional installation captured in a reproducible build/tooling contract.
- The repository's current manifests contain only `@msgpack/msgpack` and `rmp-serde`, with no protobuf, Fory, Bebop, FlatBuffers, Thrift, or Cap'n Proto runtime. This supports refactoring the DTO boundary first and running a separate v4 codec spike; it does not support introducing one of these as an implicit dependency now.
