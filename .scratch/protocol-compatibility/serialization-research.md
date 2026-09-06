# `protocol.ts` 序列化方案调查

## 结论

**当前不应因为“想换编码器”直接把 v3 改成 SCALE。** 当前真正的问题是 wire schema、语义模型和校验逻辑分散在 TypeScript 与 Rust 两套实现中；MessagePack 不是当前测量出的瓶颈。

建议分两层决策：

1. **短期保留 v3 MessagePack。** 把 `protocol.ts` 收敛为深模块：上层只使用语义化的 `Snapshot`、`Patch`、`Command`、`Event` DTO，不再构造或消费 tuple 下标；把帧、编码、结构校验、领域校验分开。保留现有 positional wire 和 golden-vector 合同，补齐 MessagePack decoder 的容器长度限制。
2. **下一次有意破坏 wire 的 v4 优先评估 Protobuf。** 用单一 `.proto` 作为跨语言 schema，生成 Rust/TypeScript wire bindings，外面保留手写语义校验器。Protobuf 的 `oneof`、显式 `optional` 和字段号演进规则正好解决当前多态 payload、稀疏 Style 和 tuple 维护风险。

SCALE 只在“Rust 是绝对主导端、协议长期严格 lockstep、TypeScript 端愿意维护一套手写/生成 codec、极致字节紧凑性优先于演进性”的前提下成立。这个仓库的 TypeScript/Bun renderer 是一等 peer，因此不推荐它作为默认方案。

---

## 1. 当前实现与实际约束

### 1.1 Wire 结构

仓库当前协议是：

```text
[u32 little-endian payload length][MessagePack payload]
```

`MAX_FRAME_SIZE` / `MAX_FRAME_LENGTH` 均为 16 MiB。四字节 frame 层已经独立处理分片、合并帧、截断和超限；它不依赖 MessagePack 的具体结构。来源：`docs/protocol.md` §1、`packages/solid-gpui/src/protocol.ts`、`crates/solid-gpui/src/protocol.rs`。

v3 的四种消息均为 positional array：

```text
Snapshot [3, 1, surfaceId, epoch, baseRevision, revision, nodes]
Event    [3, 2, surfaceId, epoch, revision, sequence, nodeId, listenerId, eventType, payload]
Patch    [3, 3, surfaceId, epoch, baseRevision, revision, operations]
Command  [3, 4, surfaceId, epoch, afterRevision, requestId, nodeId, kind, payload]
```

协议当前是 lockstep：没有协商、双版本解码或 resync。新增 capability 通过当前 tuple 的 optional tail 表达；`ADR-0011` 明确拒绝保留历史 arity、地图化 wire 和默认填充兼容分支。

### 1.2 复杂度在哪里

`packages/solid-gpui/src/protocol.ts` 为 981 行；Rust semantic protocol 为 1,381 行，`src/protocol/wire/` 另有约 3,169 行。复杂度不是单纯的 MessagePack API 调用，而是：

- TypeScript tuple 类型、编码构造和 event validator；
- Rust `serde` tuple structs、`#[serde(untagged)]` 枚举、双向转换和独立 validator；
- `RootContainer`、`NodeGraph`、`dispatch.ts` 继续直接依赖 tuple 下标；
- `docs/protocol.md`、golden vectors、Rust/TS 两组测试同步维护。

特别是 `Style` 当前为固定长度的约 42 项数组。`style.ts` 会为未设置属性编码 `null` 或默认码；`wire/node.rs` 需要按位置维护同一字段序列。它保证了紧凑和当前 deterministic shape，但字段重排或插入会影响整个协议。

多态数据也依赖手工 tag：`HostProperties`、`PatchOperation`、`EventPayload`、`CommandValue` 和菜单项均需要 discriminator；Rust 端大量使用 untagged enum 按 shape 尝试匹配。

### 1.3 性能证据

旧的 TypeScript snapshot benchmark fixture 已随前端切换和关键测试收敛一起删除，仓库当前没有可复现的 encode 性能基线。因此，换 Protobuf/SCALE 不能仅凭“更快”成立；任何 codec 迁移都必须先针对当前 Solid renderer、Rust materialization 和 GPUI first draw 建立新的端到端 benchmark。

### 1.4 一个应立即处理的 MessagePack 风险

`decodeWire()` 当前只传入 `{ useBigInt64: false }`。`@msgpack/msgpack` v3.1.3 的 Decoder 默认 `maxStrLength`、`maxBinLength`、`maxArrayLength`、`maxMapLength`、`maxExtLength` 为 `UINT32_MAX`；当前外层 16 MiB frame 限制了实际 payload 字节数，但不等同于语义上的数组/映射元素数限制。应在保留 MessagePack 时显式设置与协议一致的容器上限，并继续保留各字段和领域 validator。

---

## 2. MessagePack：保留并深模块化

### 2.1 一手资料事实

- MessagePack 是通用对象格式，原生有 integer、nil、boolean、float、string、binary、array、map、extension 等类型；array 是有长度的序列，map 是 key/value 集合。规范：<https://github.com/msgpack/msgpack/blob/master/spec.md>。
- MessagePack 的整数与 float32/float64 有不同 wire forms；`@msgpack/msgpack` 的 `forceFloat32` 只影响非整数 number，不能强制整数编码为 float。仓库 `ADR-0003` 因此要求 producer-byte stability、跨语言语义等价和允许的 numeric dual forms，而不是所有字节完全相同。
- `rmp-serde` 可以将 Rust tuple/struct 映射为 MessagePack；`Vec<u8>` 需要 binary 配置/`serde_bytes`，否则可能落成数组。来源：<https://docs.rs/rmp-serde/latest/rmp_serde/>。
- 当前 JS runtime 兼容 TypeScript/Bun；其 v3.1.3 README 和 Decoder 实现说明了 `forceFloat32`、单对象 decode、`decodeMultiStream` 与 max 容器选项。来源：<https://raw.githubusercontent.com/msgpack/msgpack-javascript/v3.1.3/README.md>、<https://raw.githubusercontent.com/msgpack/msgpack-javascript/v3.1.3/src/Decoder.ts>。

### 2.2 更好的 MessagePack 接口

不要让 `RootContainer` 看到 wire tuple。外部 seam 可以收敛为：

```ts
export interface ProtocolCodec {
  encodeFrame(message: OutboundMessage): Uint8Array;
  push(chunk: Uint8Array | ArrayBuffer): InboundMessage[];
}

type OutboundMessage = SnapshotMessage | PatchMessage | CommandMessage;
type InboundMessage = EventMessage;
```

`SnapshotMessage`、`PatchMessage`、`CommandMessage`、`EventMessage` 使用带 `kind` 的普通不可变 DTO；`ProtocolCodec` 内部再映射到当前 positional MessagePack。`FrameDecoder` 应是独立的 `FrameCodec`，不与 MessagePack validator 互相知道实现细节。

建议拆成以下内部模块，而不是继续扩展一个 981 行文件：

- `protocol/constants.ts`：版本、消息/事件/命令码、资源上限；
- `protocol/types.ts`：语义 DTO 和 discriminated union；
- `protocol/codec-msgpack.ts`：tuple ↔ MessagePack；
- `protocol/validate.ts`：结构、资源、数值和组合不变量；
- `protocol/frame.ts`：四字节 LE framing。

这是代码组织上的改进，不改变 v3 wire。调用方只迁移到 DTO constructor / decoder 结果；旧 tuple 不保留 alias 或 wrapper。

### 2.3 优点与缺点

**优点：** 零新增跨语言 schema 工具链；保留已验证的 Bun/Rust runtime、frame 和 golden vectors；迁移风险小；能立即消除 `dispatch.ts`、`nodes.ts`、`root-container.ts` 的大量下标访问。

**缺点：** 若仍保留 positional tuple，字段位置依旧是 wire 合同；Rust 与 TS 仍各自需要一份 wire mapping。DTO 只改善 seam 和 locality，不自动生成跨语言 schema。若进一步把 tuple 改成 MessagePack map，可获得按 key 的可演进性，但会增加每个节点的 key 开销，且与 `ADR-0011` 的现行 v3 决策冲突，应视为 v4 设计而不是局部重构。

**裁决：** 对当前 v3，这是最高杠杆、最低风险的第一步；但它不是长期跨语言 schema 的最终解。

---

## 3. Protobuf：v4 的首选 schema-driven 方向

### 3.1 一手资料事实

- Protobuf wire 是 field number + wire type 的 key/value record；wire type 让旧 parser 跳过不认识的新字段。字符串、bytes、嵌入 message 使用 length-delimited；`float` 使用 fixed32；`uint32` 使用 varint。官方编码指南：<https://protobuf.dev/programming-guides/encoding/>。
- Proto3 字段号一旦投入使用不能修改；删除字段必须 reserve field number/name，不能重用。显式 `optional` 提供 presence，官方指南推荐其用于兼容性；repeated 标量默认 packed。官方语言指南：<https://protobuf.dev/programming-guides/proto3/>。
- `oneof` 原生表达互斥多态字段，适合 `EventPayload`、`PatchOperation`、`Command`、`HostProperties` 和 `CommandValue`，不再需要通过 tuple shape 试探。
- Protobuf binary serialization **不是 canonical**：字段可以以不同顺序出现，unknown fields 和 deterministic serialization 也不等于全球唯一字节表示。官方说明：<https://protobuf.dev/programming-guides/serialization-not-canonical/>。因此迁移后仍应保留“各 producer 自己的 bytes 稳定 + 两端语义等价”的 golden 合同，不应改成盲目要求 TS/Rust 每个字节永远相同。
- TypeScript 可用 Buf 的 `@bufbuild/protobuf` + `protoc-gen-es`：该项目声明支持纯 TypeScript、Bun、protobuf conformance 和标准 protoc plugin。来源：<https://github.com/bufbuild/protobuf-es>。Google 官方 `google-protobuf` JS runtime 对当前 ESM/Bun 集成不如 protobuf-es 合适；选择 runtime 前仍需做 bundle/build smoke。
- Rust `prost` 从 proto2/proto3 生成 Rust struct/enum，支持 `encoded_len`，但 `prost-build` 通常需要 `protoc`；它不提供 runtime reflection。来源：<https://docs.rs/prost/latest/prost/>、<https://docs.rs/prost/latest/prost/trait.Message.html>。

### 3.2 建议的 v4 schema 形状

保持现有 outer frame，不使用 Protobuf 的 `encodeDelimited` 再添加第二个长度前缀：

```text
[u32 little-endian payload length][protobuf Envelope bytes]
```

概念 schema：

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

`Node` 的 `HostProperties` 使用 oneof；`Event` 的 payload 使用一个 oneof，把当前 `eventType` 与 payload 的重复 discriminator 合并；`Command` 的每个 command kind 使用一个带自身参数的 oneof message。`Style` 使用每个属性一个显式 optional field，而不是固定 42 槽；`bytes` 用于图片；所有 ID、epoch、revision、sequence、index 使用 `uint32`；几何和样式浮点使用 `float`。

Patch 的 `UpdateNode` 不应机械删除现有 `change_mask`：当前 `mask + null` 表达“清除某字段”，单纯的 protobuf field presence 不能同时表达“未改变”和“明确清除”。第一版可保留 mask，并规定 mask 与 optional 字段/clear 语义严格一致；更彻底的 v5 形状可以为每个可清除字段定义 `oneof { value; clear; }`。

### 3.3 不能交给 Protobuf 的验证

Protobuf 只负责 wire 类型/基本结构，不负责当前领域合同。decode 后仍需手写 validator，至少包括：

- `protocol_version` 精确匹配当前版本；
- `revision > base_revision`、Patch base 与 retained tree revision 相等；
- Surface、epoch、node/listener identity 和 event sequence；
- Node parent/index 拓扑、RawText/Text 层级和 host property kind 匹配；
- finite/non-negative geometry、u32 业务范围、Style 取值；
- tooltip、路径、文本、图片和文件的 UTF-8/字节上限；
- root event、close request、focus/blur、command result 的 node/listener 约束；
- 每个 oneof variant 与当前 command/event 语义的组合约束。

Protobuf parser 的默认长度能力不是业务上限。应先在 frame 层拒绝 >16 MiB，再在 validator/reader 层约束字符串、bytes、repeated 数量和嵌套深度。Protobuf 官方 C++ `CodedInputStream` 文档也将 total bytes、nested push limit 和 recursion limit 作为资源控制点：<https://protobuf.dev/reference/cpp/api-docs/google.protobuf.io.coded_stream/>。

### 3.4 优点、成本与风险

**优点：** 单一 `.proto` 消除 TS tuple type 与 Rust `wire/*` 的重复 schema；`oneof` 消除 untagged shape ambiguity；optional field 只编码出现的 Style 属性；field-number/reserved 规则提供真实的 additive evolution；float 直接是 fixed32，消除当前整数形式的 f32 dual-form 问题。

**成本：** 引入 `protoc`/Buf、生成文件策略、Rust build.rs、TS package/runtime、依赖许可证和 CI reproducibility；生成 DTO 未必适合直接暴露给 `RootContainer`，仍需 semantic adapter；protobuf decode 后仍然会构造消息对象/字符串/vector，不能承诺零 allocation；需要新的 v4 fixtures 和所有 producer/consumer 迁移。

**未知字段策略：** Protobuf wire/schema 支持跳过未知字段，但“解码后再编码是否保留 unknown fields”取决于 runtime。Solid GPUI 两端是终端 consumer，不是 proxy，因此默认可丢弃未知字段；若未来要做透明 proxy，必须显式选择支持 unknown-field preservation 的 runtime，并另写约束。不能把“Protobuf 支持 unknown field”误写成“所有 Rust/TS runtime 都自动保留”。

**版本策略：** 当前 v3 是 lockstep，换 Protobuf 必须明确升为 v4；不能让一个 decoder 静默同时接受 MessagePack v3 和 Protobuf v4。若要获得 Protobuf 的演进收益，v4 之后应建立“协议 major + schema additive change + capability gate”的规则，并为删除字段永久 reserve。

**裁决：** 如果团队愿意承担一次正式 v4 cutover，Protobuf 是最适合这个仓库的 schema-driven 默认选择。它首先解决维护和演进问题，其次才可能改善 sparse Style 的体积。

---

## 4. SCALE：紧凑，但不是更好的跨语言 schema

### 4.1 一手资料事实

- `parity-scale-codec` 明确说明 decoder 两端必须分别知道类型上下文，encoded bytes 不含字段名、类型标识或 schema metadata。来源：<https://github.com/paritytech/parity-scale-codec>。
- SCALE 的 struct/tuple 是字段按声明顺序连续拼接；Vec 先写 compact length；Option 以 `00`/`01 + value` 表达；enum 先写 variant index；整数默认 little-endian。类型顺序和 enum index 就是 wire 合同。Polkadot data encoding reference：<https://docs.polkadot.com/reference/parachains/data-encoding/>。
- Rust `parity-scale-codec` 提供 `Encode`、`Decode`、`Compact`、`DecodeLimit`/内存限制等能力。来源：<https://docs.rs/parity-scale-codec/latest/parity_scale_codec/>、<https://docs.rs/parity-scale-codec/latest/parity_scale_codec/trait.DecodeLimit.html>。
- SCALE 没有 Protobuf 式 field tag/unknown-field skip。新增字段、修改 struct 顺序、修改 enum index、改变 compact/fixed 选择都需要显式版本化或两端同步；旧 decoder 不能可靠地按字段跳过未知内容。

### 4.2 在本仓库中的接口形状

可以把外部 seam 设计成：

```ts
interface ProtocolCodec {
  encodeFrame(message: OutboundMessage): Uint8Array;
  push(chunk: Uint8Array | ArrayBuffer): InboundMessage[];
}
```

内部 Rust 使用 `Encode/Decode`，TypeScript 使用一个 SCALE combinator/generated codec；`RootContainer` 不直接接触 `Compact`, `Option` 或 reader/writer。数值需逐字段决定 fixed/compact：u32 fixed 会比当前小 MessagePack integer 更占空间，compact 可节省小 ID/长度，但每个字段都必须在 TS/Rust 侧完全一致。f32、颜色和几何适合固定宽度；string/bytes/vector 仍需先检查长度上限再分配。

`scale-info` 可以作为 Rust 类型描述来源，但把它转成可审查、可稳定生成的 TypeScript codec 需要自建 code generator；它不是本项目可以直接依赖的跨语言 IDL/官方 TS 生成链。这会把当前“两套 wire mapping”变成“Rust type + 自建 metadata generator + TS runtime”三套需要验证的东西。

### 4.3 优点与缺点

**优点：** Rust 端 derive 简洁；无 field tag 时结构紧凑；LE/fixed/compact 选择可控；在严格 lockstep 的 Rust-first 系统中有很好的字节和 CPU 杠杆。

**缺点：** 不 self-describing；没有未知字段跳过；enum index/字段顺序是高风险隐式契约；TypeScript/Bun 没有一个像 Protobuf `.proto` + protoc 那样统一、广泛采用的跨语言生成路径；稀疏 Style 若直接用 Option 仍需为每个字段写 presence byte，若用 bitmask 又重新引入手工 schema 设计。

**裁决：** SCALE 适合作为 Rust/区块链生态内部 codec，不适合作为这个项目想解决“跨语言协议 schema 漂移”的答案。只有把目标明确改成“Rust-first、永远 lockstep、最小 bytes”，才应选择它。

---

## 5. 其他候选

### FlatBuffers

FlatBuffers 官方文档承诺生成多语言代码、直接从 serialized buffer 访问数据、以及 tables 的向前/向后 schema 演进：<https://flatbuffers.dev/>、<https://flatbuffers.dev/schema/>。这对大型 Snapshot 很有吸引力；Rust/TS 也有官方语言页和生成器：<https://flatbuffers.dev/languages/typescript/>、<https://flatbuffers.dev/languages/rust/>。

但它要求 `flatc` schema/build pipeline；Patch operation 需要 union/wrapper table；TS 的 object API 会重新 unpack/allocate；FlatBuffers 自身没有内置 wire format version，且 binary field order 不应被当作 canonical bytes。Snapshot 最终还要进入 Rust retained tree，zero-copy 不会自动消除该 materialization 成本。**只有性能 profile 明确显示 Snapshot decode/materialization 是瓶颈时，才值得做 FlatBuffers spike；不作为当前默认。**

### Cap'n Proto

Cap'n Proto 有强类型 ordinal evolution 和 zero-copy，但其 stream framing 是 segment-count/word-size 结构，不是当前四字节 total payload prefix；官方 other-language 列表也没有一个与 Bun TypeScript 一等匹配的成熟路径。来源：<https://capnproto.org/language.html>、<https://capnproto.org/encoding.html>、<https://capnproto.org/otherlang.html>。**不推荐。**

### CBOR / JSON / Rust-only codec

CBOR 或 JSON 可以改变表示，但不会自动解决当前 schema duplication、tag/variant 建模和领域验证问题；`bincode`/`rkyv` 等 Rust-centric codec 更不适合 TypeScript/Bun peer。除非出现外部 interoperability 需求，不值得从已工作的 MessagePack 再换一套 generic format。

---

## 6. 决策矩阵

| 维度 | 当前 MessagePack v3 | MessagePack DTO seam | Protobuf v4 | SCALE | FlatBuffers |
|---|---|---|---|---|---|
| 跨语言 schema | TS/Rust 双写 | 仍双写，但集中 wire adapter | 单一 `.proto` + generated bindings | Rust 类型 + 自建 TS 生成/codec | `.fbs` + `flatc` |
| 多态建模 | 手工 tag/untagged shape | 同 wire，调用方不见 tag | `oneof` | enum index/手工 | union/table |
| 稀疏 Style | 固定槽位，很多 null/default | 不变 | optional fields，自然省略 | 需 bitmask/Option 设计 | table offsets |
| 演进 | 当前 lockstep；尾部规则 | 当前 lockstep | field number/reserved/unknown skip | 版本化；无 unknown skip | table additive，但需额外版本标识 |
| Bun/TS 现成度 | 已验证 | 已验证 | protobuf-es/Buf 可行，需新增工具链 | codec 选择分散，需自建/引入方案 | 官方生成器可行，需更重 builder |
| Rust 现成度 | 已验证 | 已验证 | prost 可行，需 protoc/build.rs | 一流 | 可行 |
| frame 复用 | 是 | 是 | 是，保留外层 4-byte LE | 是 | 是，但 schema/version 自管 |
| 当前必要性 | — | **立即值得** | **v4 长期首选** | 不足 | 仅 profile 驱动 |

---

## 7. 推荐落地顺序

### 阶段 A：不改 v3 wire

1. 定义语义 DTO 和 `ProtocolCodec` seam；`RootContainer`、`SurfaceHost`、`dispatch.ts`、`NodeGraph` 不再读写 tuple 下标。
2. 把 frame、MessagePack mapping、结构校验、领域校验拆开；保留 `ADR-0003` 的三层 golden contract。
3. 为 `@msgpack/msgpack` 设置 max string/bin/array/map/ext 选项；若需要嵌套深度限制则在协议 validator/解析策略中单独实现。资源字节上限仍由领域 validator 负责。
4. 给 `ProtocolCodec` 增加 focused tests：Snapshot/Patch/Command/Event 代表性 DTO、malformed oneof/tag、frame fragmentation/coalescing、资源上限和跨语言 golden vectors。

### 阶段 B：为 v4 做 Protobuf spike

1. 新建单一 schema 文件；先只覆盖 Snapshot、Patch、Command、Event，不让 generated types 穿透 renderer/host domain seam。
2. 以真实 fixtures 测量：20k mixed snapshot、稀疏 Style、style-only patch、pointer/keyboard event storm、图片/文件最大边界；记录 payload size、encode/decode CPU、TS allocations/GC、Rust allocations。
3. 选择 `@bufbuild/protobuf`/`protoc-gen-es` 与 `prost` 的可复现 codegen 方式，确认 Bun ESM、Cargo build、许可证和 CI 不依赖用户机器上的未声明工具。
4. 保留外层 4-byte LE frame；Protobuf payload 不再添加 inner length delimiter。
5. 设计 `UpdateNode` 的 clear semantics、未知字段策略、版本/能力策略，并为删除 field 永久 reserve。
6. 通过 v4 一次性 cutover：更新 host/package、docs、fixtures、golden tests；不要保留 v3/v4 双 decoder 或历史 wrapper。

### 阶段 C：仅在 profile 支持时做 FlatBuffers 对照

若 Protobuf spike 证明大 Snapshot decode/materialization 仍是实际瓶颈，再用同一 fixture 做 FlatBuffers 对照；若只是当前 GPUI first draw 主导，则不引入第三种 schema 工具链。

## 来源索引

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
- FlatBuffers overview/schema/languages: <https://flatbuffers.dev/>、<https://flatbuffers.dev/schema/>、<https://flatbuffers.dev/languages/typescript/>、<https://flatbuffers.dev/languages/rust/>
- Cap'n Proto language/encoding/other languages: <https://capnproto.org/language.html>、<https://capnproto.org/encoding.html>、<https://capnproto.org/otherlang.html>

## 8. 本地验证记录

- `cargo test -p solid-gpui --lib tests::protocol`：11 passed，0 failed；覆盖当前 MessagePack round-trip、frame truncation/oversize、version mismatch、host-property mismatch、command result 和 event payload 合同。
- `bun run task package-test`：6 passed，0 failed；覆盖 signal Patch、跨 root owner、条件挂载、VirtualList 重挂载、render error 恢复和 surface close cleanup。
- 本调查没有改动生产源码；新增的唯一仓库文件是本报告。

---

## 9. 扩展候选面

本节把候选扩大到“有 schema/codegen 且可能用于 Rust + TypeScript”的格式。评估的是本仓库的实际 hot path：TypeScript 生成 Commit Batch，Rust 解析后 materialize retained Host Node tree；不是只比较裸 codec benchmark。

### 9.1 Schema-first 候选

| 方案 | 单一 schema 生成 Rust + TS | 性能/内存模型 | 主要缺口 |
|---|---|---|---|
| Protobuf | `prost` + `protobuf-es/protoc-gen-es` | tag/varint/fixed32；稀疏字段省略；解析后通常有对象/字符串分配 | 需要 protoc/Buf/build.rs；仍需 semantic adapter |
| Bebop | 官方 `bebopc` 生成 Rust 与 TypeScript；`.bop` schema | 固定宽度 LE 数值、长度前缀 message/union、生成式 encode/decode；理论上很适合高频小消息 | 生态明显小于 Protobuf/FlatBuffers；bounded decode API、兼容策略和长期维护需实测确认 |
| Apache Fory | 官方 Fory IDL/compiler 可生成 Rust 与 JavaScript/TypeScript；`.fdl` schema | xlang、field metadata、union/optional、generated serializers；runtime 有 graph/depth/container limits | compiler/JS runtime 当前 alpha 线；需要 type registration/compatible mode；Bun 与 node-gyp/依赖路径需验证，运行时比 Bebop/Protobuf 更重 |
| FlatBuffers | 官方 `flatc --rust --ts` | 生成 accessor 可直接从 buffer 读取；适合大型 Snapshot | TS 写入需要 builder；Patch union/wrapper 较重；materialize 后 zero-copy 收益下降 |
| Apache Thrift | Apache compiler 的 Rust 与 `ts`/`node.ts` targets；Rust/部分 JS targets 支持 Compact，node.ts 官方矩阵主要支持 Binary | 字段 ID；struct/union/optional；可跳过未知字段 | 没有 `uint32`/`float`，只有 signed integer 与 `double`；TypeScript 端不能假定 Compact 可用；运行时带 RPC/Node 取向 |
| Cap'n Proto | Rust plugin + 社区 TS 实现 | word-aligned zero-copy、ordinal evolution | 官方核心项目没有一等 Bun/TS generator；自有 segment framing；TS 实现成熟度不足 |
| ASN.1 + RASN | `rasn-compiler` 可生成 Rust；TS backend 主要生成 JER 类型定义 | APER/UPER/OER 可很紧凑 | Rust/TS 不共享同一成熟的 binary binding pipeline；ASN.1 复杂度过高 |
| Avro | Apache 有 schema resolution 与 Rust SDK | record 按 writer schema 顺序；schema resolution 强 | raw datum 依赖 schema；Apache 主项目没有与 Bun 对应的一等 TS generator/runtime |
Apache Thrift 的官方类型系统和 IDL 明确提供 struct、union、optional field、binary，协议字段使用 field ID；其 Compact protocol 还定义了 compact varint 和未知字段跳过，但官方 language matrix 的 `node.ts` 行只列 Binary，不列 Compact。Thrift 基础类型没有 unsigned integer 和 32-bit float：<https://thrift.apache.org/docs/types.html>、<https://thrift.apache.org/docs/idl>、<https://raw.githubusercontent.com/apache/thrift/master/doc/specs/thrift-compact-protocol.md>、<https://raw.githubusercontent.com/apache/thrift/master/LANGUAGES.md>。因此 Thrift 仍是可落地的 schema/codegen 备用候选，但不满足本项目对原生 `u32`/`f32`、低 glue 和高性能的综合要求，不进入最终前三。
Bebop 的官方文档显示 `.bop` schema 可由 `bebopc` 生成 TypeScript 和 Rust，且可以关闭 service 资产，只生成 records；wire 使用 little-endian fixed-width numbers、`uint32` 长度、message field index 和 union discriminator，未知 message field 可跳到 body 末尾。来源：<https://bebop.sh/guide/getting-started-typescript/>、<https://bebop.sh/guide/getting-started-rust/>、<https://bebop.sh/reference/wire-format/>、<https://bebop.sh/reference/union/>。这比 Thrift 更贴合当前的 `u32`/`f32` 与高频消息，但其生态、工具链和 bounded decode 能力必须作为 v4 spike 的硬验收项，不能直接采用文档中的泛化性能宣传。
Apache Fory 的官方 IDL 文档明确支持一次 schema 生成 Rust 与 JavaScript/TypeScript，并支持 optional、union、跨语言 type registration 和 schema-compatible mode：<https://fory.apache.org/docs/compiler/>、<https://fory.apache.org/docs/compiler/generated-code/javascript/>、<https://fory.apache.org/docs/compiler/generated-code/rust/>、<https://fory.apache.org/docs/object-serialization/javascript/schema-evolution/>。它是比 Thrift 更有力的额外候选，但当前 compiler/JavaScript runtime 的 alpha 状态、运行时注册/metadata 成本和 Bun 兼容性必须先通过 spike。

ASN.1 的 `rasn-compiler` README 明确区分 Rust bindings 与“for JER-encoded ASN.1 data elements”的 TypeScript definitions：<https://docs.rs/crate/rasn-compiler/latest/source/README.md>。这不能直接满足本项目的两端 binary codec 单源要求。Avro 的官方规格重点是 schema-dependent datum 与 writer/reader schema resolution：<https://avro.apache.org/docs/1.11.0/spec.pdf>；它的字段演进很强，但不是 Bun + Rust 的低胶水首选。

### 9.2 Binary format + schema/validation 候选

| 方案 | 事实 | 为什么不进入前三 |
|---|---|---|
| CBOR + CDDL | IETF 标准；CBOR extensible、self-describing；CDDL 描述 CBOR/JSON | CDDL 是 schema notation，不是统一跨语言 binding generator；需要分别选择 CBOR runtime、Rust typegen、TS typegen/validator |
| MessagePack + 自建 schema generator | 当前 runtime 已验证，数组很紧凑 | 自建 generator 变成项目自己的编译器；维护成本和风险超过保留现有 DTO seam 的收益 |
| SCALE | Rust 端轻量、compact、LE | decoder 需要外部类型上下文；字段顺序/enum index 无 tag 演进；TS 实现和 codegen 分散 |
| SBE | 适合固定布局、低延迟金融消息；schema-driven | 官方 SBE tool targets 是 Java/C++/C/Golang 等，没有本仓库需要的成熟 TS target |
| Rust `rkyv` / `postcard` / `bincode` | Rust-to-Rust 可很快或 zero-copy | 没有 TypeScript wire target，直接违反双端单源要求 |
| WIT / Component Model | 有 Rust 与 JavaScript component bindings | 是 component interface/runtime，不是可直接替换当前 Bun stdio payload 的通用 codec；会改变 Runtime Adapter 和进程模型 |
| Arrow IPC | 官方列式 IPC、FlatBuffers metadata、可做高吞吐零拷贝 batch | Host Node tree 是异构树，Command/Event/patch semantics 不适合列式；会增加一层数据重排 |
| Postcard + `postcard-bindgen` | Rust `no_std`/varint，社区 generator 可输出 JS | 生成器不是 Postcard 官方跨语言链，输出非一等 TypeScript API；仍需审查自定义属性和 bounded decode |

CBOR 的目标包含小代码、合理消息大小和扩展性，但 RFC 8949 明确它是 generic data format；CDDL RFC 8610 是描述语言，二者不提供本项目所需的官方 Rust/TypeScript 同源代码生成链：<https://www.rfc-editor.org/rfc/rfc8949>、<https://www.rfc-editor.org/rfc/rfc8610>。SBE 的工具文档列出的 target language 也不含 TypeScript：<https://github.com/aeron-io/simple-binary-encoding/wiki/Sbe-Tool-Guide>。这些方案可以工作，但会重新制造 glue。
Arrow IPC 的官方格式是列式、带 FlatBuffers metadata 和 body buffers 的数据交换格式：<https://arrow.apache.org/docs/format/Columnar.html>。它适合同构数组 batch，不适合当前异构 Host Node tree。Postcard 官方 README 将其定位为 Rust/Serde/no_std codec；`postcard-bindgen` 是另一个社区 generator，不应视作 Postcard 的官方跨语言契约：<https://github.com/jamesmunns/postcard>、<https://github.com/teamplayer3/postcard-bindgen>。

### 9.3 Cap'n Proto 与 FlatBuffers 的性能边界

FlatBuffers 官方 TypeScript 文档明确区分两种 API：基础 API 直接在 `ByteBuffer` 上访问，object API 会 unpack/pack 成普通对象并牺牲效率；官方 `flatc` 支持 `--rust`、`--ts`、schema conformance 和 size-prefixed buffers：<https://flatbuffers.dev/languages/typescript/>、<https://flatbuffers.dev/flatc/>。这使它成为 Snapshot 读取性能的强候选，但不代表 TypeScript 的 Commit Batch 构造免费。

Cap'n Proto 的编码是 segment tree，语言演进依赖字段 ordinal；它的官方文档与 language list 没有一个与本仓库 Bun/TypeScript 组合相当的 first-party path：<https://capnproto.org/encoding.html>、<https://capnproto.org/language.html>、<https://capnproto.org/otherlang.html>。社区 `capnp-es` 可作为实验对象，但其成熟度和维护风险不符合“代码质量好、少胶水”的硬条件。

---

## 10. 最终推荐的三个方案

### 方案一：Protobuf v4，全协议统一（首选）

**适用优先级：** 综合代码质量、演进能力、性能和低 glue。

- 单一 `protocol.proto`；
- Rust：`prost` + `prost-build`；
- TypeScript：`@bufbuild/protobuf` + `protoc-gen-es`；
- 保留当前四字节 LE outer frame；
- `Envelope.oneof { snapshot, patch, command, event }`；
- `HostProperties`、`PatchOperation`、`EventPayload`、`CommandValue` 使用 `oneof`；
- `Style` 使用显式 `optional`，避免固定 42 槽的 null；
- generated bindings 只停留在 protocol seam，不能让 `RootContainer` 依赖 protobuf-specific API；
- tree topology、revision、surface generation、resource limits 继续由 semantic validator 负责。

**性能判断：** 对小 Event/Command，tag/varint/fixed32 很合适；对稀疏 Style，省略未设置字段可能明显降低 Snapshot/Patch 体积；但 protobuf decode 会 materialize message 对象，不能承诺 FlatBuffers 级别的 zero-copy。

**主要风险：** codegen toolchain、`UpdateNode` 的 clear semantics、unknown field 策略、generated output 是否提交。需要 v4 clean cutover，不能让 v3 MessagePack decoder 同时猜测 protobuf。

### 方案二：FlatBuffers，全协议 schema-first（Snapshot 性能优先）

**适用优先级：** 大 Snapshot/Patch 的 decode、内存占用和读取延迟第一。

- 单一 `protocol.fbs`；
- `flatc --rust --ts` 生成两端 bindings；
- 基础 accessor API 读取 Snapshot，避免 `--gen-object-api` 进入 hot path；
- 使用 tables 表达 Node/Style/Command/Event，使用 explicit union wrapper 表达 PatchOperation/Event payload；
- 仍保留四字节 LE outer frame、file identifier/version 和 verifier；
- Rust host 若最终必须 materialize retained tree，应 benchmark “直接读取 + clone 到 NodeStore”的总成本，而不是只测 accessor。

**性能判断：** 在“只读大 buffer”模型中，这是前三者里最有机会拿到最低分配和最低 decode CPU 的方案。当前项目的 Rust renderer 需要拥有 retained tree，所以收益取决于是否能延后或减少 materialization。

**主要风险：** TypeScript builder API 比 protobuf DTO 更笨重；字符串/向量 offset 构造、union wrapper 和生命周期规则会增加 adapter glue。事件/命令这种小消息未必占优。若真实 profile 仍由 GPUI draw 主导，不值得为理论 zero-copy 引入它。

### 方案三：Bebop，全协议 schema-first，高性能 challenger

**适用优先级：** 追求单一 `.bop` schema、生成式 encoder/decoder、固定宽度数值和低运行时抽象；接受生态规模与长期维护风险。

- 单一 `.bop`；
- `bebopc` 同时生成 Rust 与 TypeScript；
- 关闭 service/client/server 生成，只保留 records；
- 用 `message` 表达可演进字段，用 `union` 表达 Event/Command/Patch 多态；
- 使用当前四字节 LE outer frame；Bebop message 自身已有 body length，但不应替换现有 transport frame；
- 直接使用 `uint32`、`float32`、`bytes`/string，不需要 Thrift 那种有损的类型折衷；
- 禁止把 generated record API 泄漏到 Solid domain；只在 protocol seam 做一个方向适配；
- 对所有 array/map/string/bytes/body 长度、union nesting 和 recursion 做单独上限验证。

**性能判断：** Bebop 的 schema-driven generated path 避免通用 reflection；固定宽度 little-endian 数值与长度前缀适合当前 u32/f32 和高频 Event；message body length 可跳过未知字段。它是 Protobuf 的合理性能 challenger，但目前没有本仓库的实测数据，不能把项目文档的跨格式 benchmark 当成结论。

**主要风险：** 生态和工具链远小于 Protobuf/FlatBuffers；Rust 生成目录会在 build 中自动生成，TypeScript 需要独立 generator 配置；必须确认 Bun ESM、生成器版本 pin、Cargo reproducibility、malformed input 的 bounded decode 和 unknown union branch 行为。若这些检查不过，降级为研究候选，不进入生产前三。

---

## 排名裁决

| 排名 | 方案 | 强项 | 不能接受的条件 |
|---:|---|---|---|
| 1 | Protobuf v4 | 最好的综合平衡；schema/codegen、oneof、optional、演进规则最完整 | 不愿引入 protoc/Buf/codegen pipeline |
| 2 | FlatBuffers | 大 Snapshot 读取的 CPU/内存上限最高 | 不能接受 builder/union glue，或 profile 证明 materialization/draw 才是瓶颈 |
| 3 | Bebop | generated Rust/TS、固定宽度数值、message/union 长度边界、低 reflection 开销 | 生态小；bounded decode、Bun ESM 和长期工具链稳定性未验证 |

**对当前仓库的实际选择：** 现在继续使用 MessagePack v3，并先做 DTO/protocol seam 重构；如果决定正式切换，优先做 Protobuf v4 spike。若性能 profile 证明 Snapshot decode/materialization 是主瓶颈，比较 FlatBuffers；若追求更小的 generated codec 与固定宽度高频消息，比较 Bebop。Thrift Binary 作为有外部 Thrift 互操作需求时的备用候选，不进入前三。

---

## 11. 本地工具链可行性

- 当前工作站能找到 `/opt/homebrew/bin/protoc`，版本为 `libprotoc 36.0`。
- `foryc`、`bebopc`、`flatc`、`thrift`、`capnp` 当前不在 PATH；Fory、Bebop、FlatBuffers、Thrift、Cap'n Proto 的 spike 需要额外安装并纳入可复现的 build/tooling contract。
- 仓库现有 manifest 只包含 `@msgpack/msgpack` 与 `rmp-serde`，没有 protobuf、Fory、Bebop、FlatBuffers、Thrift 或 Cap'n Proto runtime。这个事实支持“先做 DTO seam，再单独做 v4 codec spike”，不支持现在直接引入其中一个作为隐式依赖。
