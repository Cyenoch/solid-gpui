#!/usr/bin/env bun
/**
 * Serializes an already-built application entry (Vite JS output) into Bun's
 * standalone module graph and emits the Rust source that carries it in the
 * final executable image.
 *
 * Design constraints this file implements:
 *
 * - Vite stays the application compiler; the packing phase consumes its output.
 * - The serializer is an explicitly supplied, pinned-revision Bun executable.
 *   Nothing is resolved from `PATH` and no version drift or fallback is
 *   tolerated: the graph payload carries no version field, so format
 *   compatibility is bound to the serializer identity alone.
 * - The graph payload embeds the *target* runtime's virtual path prefix
 *   (`B:/~BUN/root/` on Windows, `/$bunfs/root/` elsewhere, see
 *   `target_base_public_path`). The Windows resolver only admits graph keys
 *   that start with its own prefix, so a Windows product must be serialized
 *   with a Windows compile target.
 * - The payload is validated against the pinned format before it is written:
 *   section length prefix, trailer, offsets record, module record count and
 *   stride, every string pointer, the entry identity, and forbidden graph
 *   contents (JSC bytecode, native addons).
 * - The emitted Rust file declares the graph section as an immutable static so
 *   the runtime maps the payload straight from the executable image. Nothing is
 *   unpacked to disk or copied at build or run time.
 */

import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { basename, isAbsolute, join } from "node:path";
import { parseArgs } from "node:util";

/**
 * Canonical pinned Bun build inputs. Owned by the sys crate so that the
 * serializer revision and the native build never drift apart.
 */
const BUN_BUILD_CONFIG = join(import.meta.dirname, "../crates/solid-gpui-bun-sys/bun-build.json");

/**
 * Reads the pinned Bun revision.
 *
 * The graph payload carries no format version, so a payload produced by any
 * other Bun build is accepted silently by the runtime. The revision is
 * therefore read from the same file the native build consumes and enforced.
 */
async function pinnedBunRevision(): Promise<string> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(await readFile(BUN_BUILD_CONFIG, "utf8"));
  } catch (error) {
    fail(`${BUN_BUILD_CONFIG} could not be read: ${error instanceof Error ? error.message : String(error)}`);
  }
  if (parsed === null || typeof parsed !== "object" || !("revision" in parsed)) {
    fail(`${BUN_BUILD_CONFIG} does not declare a bun revision`);
  }
  const revision = parsed.revision;
  if (typeof revision !== "string" || !/^[0-9a-f]{40}$/.test(revision)) {
    fail(`${BUN_BUILD_CONFIG} does not declare a 40-character hexadecimal bun revision`);
  }
  return revision;
}

/** `StandaloneModuleGraph::TRAILER`. */
const GRAPH_TRAILER = new TextEncoder().encode("\n---- Bun! ----\n");
/** `size_of::<Offsets>()` for the pinned format (repr(C), x86_64/aarch64). */
const GRAPH_OFFSETS_SIZE = 32;
/** `size_of::<CompiledModuleGraphFile>()` for the pinned format (repr(C)). */
const GRAPH_FILE_SIZE = 52;
/** Extensions that can only be satisfied by loading native code from disk. */
const NATIVE_EXTENSIONS = [".node", ".dll", ".so", ".dylib"] as const;

export type EmbeddedGraphTarget = "bun-windows-x64" | "bun-windows-arm64" | "bun-darwin-arm64" | "bun-darwin-x64";

type TargetSpec = {
  /** Host platform whose Bun binary is the compile base unless one is supplied. */
  readonly host: string;
  readonly container: "pe" | "macho";
  /**
   * Machine type the image's own header must declare: `IMAGE_FILE_MACHINE_*`
   * for PE, `CPU_TYPE_*` for Mach-O.
   *
   * The graph key prefix distinguishes Windows from POSIX but *not* x64 from
   * arm64, so without this the serializer's own output — and therefore the
   * compile base behind it — would be accepted for either architecture.
   */
  readonly machine: number;
  /** Human-readable spelling of `machine`, for diagnostics. */
  readonly machineName: string;
  /** `StandaloneModuleGraph` key prefix written into every graph file name. */
  readonly prefix: string;
  /** Attribute value for the generated `#[unsafe(link_section = ...)]`. */
  readonly linkSection: string;
};

const TARGETS: Record<EmbeddedGraphTarget, TargetSpec> = {
  "bun-windows-x64": {
    host: "win32-x64",
    container: "pe",
    machine: 0x8664,
    machineName: "x64",
    prefix: "B:/~BUN/root/",
    linkSection: ".bun",
  },
  "bun-windows-arm64": {
    host: "win32-arm64",
    container: "pe",
    machine: 0xaa64,
    machineName: "arm64",
    prefix: "B:/~BUN/root/",
    linkSection: ".bun",
  },
  "bun-darwin-x64": {
    host: "darwin-x64",
    container: "macho",
    machine: 0x0100_0007,
    machineName: "x86_64",
    prefix: "/$bunfs/root/",
    linkSection: "__BUN,__bun",
  },
  "bun-darwin-arm64": {
    host: "darwin-arm64",
    container: "macho",
    machine: 0x0100_000c,
    machineName: "arm64",
    prefix: "/$bunfs/root/",
    linkSection: "__BUN,__bun",
  },
};

/**
 * `CompiledModuleGraphFile::loader` discriminants, from the canonical
 * `bun_ast::Loader` enum (`src/ast/loader.rs`), which is the source of truth
 * for the serialized byte. The `Bun.build` schema enum is a different,
 * one-based enum and must not be used here.
 */
const LOADER_NAMES: Record<number, string> = {
  0: "jsx",
  1: "js",
  2: "ts",
  3: "tsx",
  4: "css",
  5: "file",
  6: "json",
  7: "jsonc",
  8: "toml",
  9: "wasm",
  10: "napi",
  11: "base64",
  12: "dataurl",
  13: "text",
  14: "bunsh",
  15: "sqlite",
  16: "sqlite_embedded",
  17: "html",
  18: "yaml",
  19: "json5",
  20: "md",
  21: "xml",
};
/** `Loader::Napi`: a native addon the runtime can only load from a file. */
const LOADER_NAPI = 10;

const ENCODING_NAMES: Record<number, string> = { 0: "binary", 1: "latin1", 2: "utf8" };
const MODULE_FORMAT_NAMES: Record<number, string> = { 0: "none", 1: "esm", 2: "cjs" };
const FILE_SIDE_NAMES: Record<number, string> = { 0: "server", 1: "client" };

/** One `CompiledModuleGraphFile` record of the serialized graph. */
export type EmbeddedGraphFile = {
  readonly name: string;
  readonly isEntryPoint: boolean;
  readonly loader: string;
  readonly encoding: string;
  readonly moduleFormat: string;
  readonly side: string;
  readonly contentsLength: number;
  readonly sourcemapLength: number;
  readonly bytecodeOriginPath: string;
};

/** Validated view of a serialized standalone module graph payload. */
export type EmbeddedGraph = {
  readonly target: EmbeddedGraphTarget;
  /** Payload length, excluding the section's 8-byte length header. */
  readonly payloadLength: number;
  /** `Offsets::byte_count`: the region covered by the graph's own pointers. */
  readonly byteCount: number;
  readonly flags: number;
  readonly compileExecArgv: string;
  /** Graph identity of the entry point, exactly as the runtime resolves it. */
  readonly entry: string;
  readonly files: readonly EmbeddedGraphFile[];
};

export type PackageEmbeddedGraphOptions = {
  /** Absolute path to the pinned-revision Bun executable used as serializer. */
  readonly bun: string;
  /** Absolute path to the built application entry (Vite output, already JS). */
  readonly entry: string;
  /** Platform of the executable that will host the graph. */
  readonly target: EmbeddedGraphTarget;
  /** Absolute directory that receives `bun-embedded-graph.{rs,bin}`. */
  readonly outDir: string;
  /** Absolute paths embedded into the graph as resources, with their relative paths preserved. */
  readonly assets?: readonly string[];
  /** Absolute paths to additional entry points (worker scripts) embedded into the graph. */
  readonly workers?: readonly string[];
  /**
   * Absolute path to a target-platform Bun executable used as the compile base.
   *
   * Required whenever `target` is not the serializer's own platform: the
   * serializer would otherwise download that target's Bun package at its own
   * version, and a downloaded base is not covered by the revision pin. The base
   * must be an executable for `target` — the image the serializer writes
   * inherits its machine type, which extraction checks.
   */
  readonly baseExecutable?: string;
};

export type PackagedEmbeddedGraph = {
  /** Absolute path to the generated Rust include source. */
  readonly rustSource: string;
  /** Graph identity of the packaged entry, for `EmbeddedBunAdapter::start_packaged`. */
  readonly entry: string;
  /**
   * SHA-256 of the serialized graph payload, without its 8-byte length header.
   *
   * A consumer that links the section into an executable re-extracts it from
   * the built image and compares this digest, which is what proves the section
   * survived the link byte for byte rather than being dropped, merged, or
   * rewritten.
   */
  readonly graphSha256: string;
};

/** Name of the generated Rust include source inside `outDir`. */
const RUST_SOURCE_NAME = "bun-embedded-graph.rs";
/** Name of the generated section bytes inside `outDir`. */
const SECTION_BYTES_NAME = "bun-embedded-graph.bin";

class EmbeddedGraphError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "EmbeddedGraphError";
  }
}

function fail(message: string): never {
  throw new EmbeddedGraphError(message);
}

function targetSpec(target: string): TargetSpec {
  const spec = (TARGETS as Record<string, TargetSpec | undefined>)[target];
  if (spec === undefined) {
    fail(
      `unsupported embedded graph target ${JSON.stringify(target)}; expected one of ${Object.keys(TARGETS).join(", ")}`,
    );
  }
  return spec;
}

// --- little-endian readers over untrusted bytes -----------------------------

function readU32(bytes: Uint8Array, offset: number, what: string): number {
  if (!Number.isInteger(offset) || offset < 0 || offset + 4 > bytes.length) {
    fail(`${what}: 32-bit field at offset ${offset} is outside the ${bytes.length}-byte payload`);
  }
  return (bytes[offset]! | (bytes[offset + 1]! << 8) | (bytes[offset + 2]! << 16) | (bytes[offset + 3]! << 24)) >>> 0;
}

function readU64(bytes: Uint8Array, offset: number, what: string): number {
  const low = readU32(bytes, offset, what);
  const high = readU32(bytes, offset + 4, what);
  if (high > 0x1f_ffff) {
    fail(`${what}: 64-bit field at offset ${offset} exceeds the supported range`);
  }
  return high * 0x1_0000_0000 + low;
}

function readAscii(bytes: Uint8Array, offset: number, length: number, what: string): string {
  if (!Number.isInteger(offset) || offset < 0 || offset + length > bytes.length) {
    fail(`${what}: ${length}-byte field at offset ${offset} is outside the ${bytes.length}-byte payload`);
  }
  let text = "";
  for (let index = 0; index < length; index += 1) text += String.fromCharCode(bytes[offset + index]!);
  return text;
}

function matchesAscii(bytes: Uint8Array, offset: number, expected: string): boolean {
  if (offset < 0 || offset + expected.length > bytes.length) return false;
  for (let index = 0; index < expected.length; index += 1) {
    if (bytes[offset + index] !== expected.charCodeAt(index)) return false;
  }
  return true;
}

function matchesBytes(bytes: Uint8Array, offset: number, expected: Uint8Array): boolean {
  if (offset < 0 || offset + expected.length > bytes.length) return false;
  for (let index = 0; index < expected.length; index += 1) {
    if (bytes[offset + index] !== expected[index]) return false;
  }
  return true;
}

// --- container extraction (PE `.bun` / Mach-O `__BUN,__bun`) ---------------

type SectionView = {
  /** Raw section bytes as stored in the file. */
  readonly bytes: Uint8Array;
  /** Bytes the loader maps for the section, when the container states it. */
  readonly mappedSize: number;
  readonly description: string;
};

/**
 * A parsed container: the machine type its own header declares, plus every
 * section whose name matches the target's graph section.
 */
type ContainerView = {
  /** `IMAGE_FILE_MACHINE_*` (PE) or `CPU_TYPE_*` (Mach-O). */
  readonly machine: number;
  readonly sections: SectionView[];
};

function peImage(image: Uint8Array): ContainerView {
  if (!matchesAscii(image, 0, "MZ")) fail("PE image: missing MZ signature");
  const lfanew = readU32(image, 0x3c, "PE image e_lfanew");
  if (readU32(image, lfanew, "PE image signature") !== 0x0000_4550) {
    fail("PE image: missing PE\\0\\0 signature");
  }
  const machine = readU32(image, lfanew + 4, "PE image machine type") & 0xffff;
  const sectionCount = readU32(image, lfanew + 6, "PE image section count") & 0xffff;
  const optionalSize = readU32(image, lfanew + 20, "PE image optional header size") & 0xffff;
  const optionalMagic = readU32(image, lfanew + 24, "PE image optional header magic") & 0xffff;
  if (optionalMagic !== 0x10b && optionalMagic !== 0x20b) {
    fail(`PE image: unsupported optional header magic 0x${optionalMagic.toString(16)}`);
  }

  // The runtime locates its graph by comparing only the first four bytes of a
  // section name (`.bun`), and takes the first match in table order, so every
  // section whose name shares that prefix is reported and disambiguated by
  // content instead of by position.
  const tableStart = lfanew + 24 + optionalSize;
  const found: SectionView[] = [];
  for (let index = 0; index < sectionCount; index += 1) {
    const header = tableStart + index * 40;
    if (header + 40 > image.length) fail("PE image: section table extends past the end of the file");
    if (!matchesAscii(image, header, ".bun")) continue;
    const virtualSize = readU32(image, header + 8, "PE .bun virtual size");
    const rawSize = readU32(image, header + 16, "PE .bun raw size");
    const rawOffset = readU32(image, header + 20, "PE .bun raw offset");
    if (rawOffset + rawSize > image.length) {
      fail(
        `PE image: .bun section raw range [${rawOffset}, ${rawOffset + rawSize}) is outside the ${image.length}-byte file`,
      );
    }
    found.push({
      bytes: image.subarray(rawOffset, rawOffset + rawSize),
      mappedSize: Math.max(virtualSize, rawSize),
      description: `PE section ${readAscii(image, header, 8, "PE section name").replace(/\0+$/, "")} (raw ${rawSize} bytes at ${rawOffset}, virtual ${virtualSize} bytes)`,
    });
  }
  return { machine, sections: found };
}

function machoImage(image: Uint8Array): ContainerView {
  const magic = readU32(image, 0, "Mach-O image magic");
  if (magic === 0xcafe_babe || magic === 0xbeb_afeca) {
    fail("Mach-O image: universal (fat) binaries are not supported; the compile base must be a thin Mach-O");
  }
  if (magic !== 0xfeed_facf) fail(`Mach-O image: unsupported magic 0x${magic.toString(16)}`);
  const machine = readU32(image, 4, "Mach-O image CPU type");
  const commandCount = readU32(image, 16, "Mach-O image load command count");
  const commandsSize = readU32(image, 20, "Mach-O image load commands size");
  const commandsStart = 32;
  if (commandsStart + commandsSize > image.length) {
    fail("Mach-O image: load commands extend past the end of the file");
  }

  const found: SectionView[] = [];
  let cursor = commandsStart;
  for (let index = 0; index < commandCount; index += 1) {
    const command = readU32(image, cursor, "Mach-O load command");
    const size = readU32(image, cursor + 4, "Mach-O load command size");
    if (size < 8 || cursor + size > commandsStart + commandsSize) {
      fail(`Mach-O image: load command ${index} has an invalid size ${size}`);
    }
    // LC_SEGMENT_64
    if (command === 0x19 && readAscii(image, cursor + 8, 16, "Mach-O segment name").replace(/\0+$/, "") === "__BUN") {
      const sectionCount = readU32(image, cursor + 64, "Mach-O segment section count");
      const sectionsStart = cursor + 72;
      if (sectionsStart + sectionCount * 80 > cursor + size) {
        fail("Mach-O image: __BUN segment section table exceeds its load command");
      }
      for (let section = 0; section < sectionCount; section += 1) {
        const entry = sectionsStart + section * 80;
        if (readAscii(image, entry, 16, "Mach-O section name").replace(/\0+$/, "") !== "__bun") continue;
        const size = readU64(image, entry + 40, "Mach-O __bun size");
        const offset = readU32(image, entry + 48, "Mach-O __bun offset");
        if (offset + size > image.length) {
          fail(`Mach-O image: __bun range [${offset}, ${offset + size}) is outside the ${image.length}-byte file`);
        }
        found.push({
          bytes: image.subarray(offset, offset + size),
          mappedSize: size,
          description: `__BUN,__bun (${size} bytes at ${offset})`,
        });
      }
    }
    cursor += size;
  }
  return { machine, sections: found };
}

/**
 * Extracts the standalone module graph payload from a compiled Bun image.
 *
 * Section names are not unique identifiers in either container format, and the
 * Windows runtime resolves its graph by a four-byte name prefix in table order,
 * so every name-matching section is probed and the payload is chosen by
 * content: exactly one candidate must carry a structurally valid graph.
 *
 * The image's own header must also declare the machine type of `target`. The
 * graph key prefix separates Windows from POSIX but not x64 from arm64, so
 * without this check a compile base of the wrong architecture — or a host link
 * that ignored the target machine — would be accepted as a valid product.
 */
export function extractGraphPayload(image: Uint8Array, target: EmbeddedGraphTarget): Uint8Array {
  const spec = targetSpec(target);
  const container = spec.container === "pe" ? "PE" : "Mach-O";
  const view = spec.container === "pe" ? peImage(image) : machoImage(image);
  if (view.machine !== spec.machine) {
    fail(
      `${container} image declares machine 0x${view.machine.toString(16)} but ${target} requires ${spec.machineName} ` +
        `(0x${spec.machine.toString(16)}); the compile base must be a ${target} executable`,
    );
  }
  const candidates = view.sections;
  if (candidates.length === 0) {
    fail(`${container} image: no ${spec.linkSection} section found, so the image carries no embedded module graph`);
  }

  const matches: { payload: Uint8Array; description: string }[] = [];
  const rejected: string[] = [];
  for (const section of candidates) {
    try {
      const payload = readSectionBlob(section);
      readGraphLayout(payload);
      matches.push({ payload, description: section.description });
    } catch (error) {
      rejected.push(`${section.description}: ${error instanceof Error ? error.message : String(error)}`);
    }
  }
  if (matches.length !== 1) {
    fail(
      `${container} image: expected exactly one section carrying a valid module graph, found ${matches.length} among ` +
        `${candidates.length} candidate sections` +
        (rejected.length > 0 ? `; rejected: ${rejected.join("; ")}` : "") +
        (matches.length > 1 ? `; matched: ${matches.map((match) => match.description).join("; ")}` : ""),
    );
  }
  return matches[0]!.payload;
}

function readSectionBlob(section: SectionView): Uint8Array {
  if (section.bytes.length < 8) {
    fail(`${section.description} is too small to hold a graph length header`);
  }
  const length = readU64(section.bytes, 0, `${section.description} graph length`);
  if (length === 0) fail(`${section.description} declares an empty graph payload`);
  if (length + 8 > section.bytes.length) {
    fail(`${section.description} declares ${length} payload bytes but only ${section.bytes.length - 8} are stored`);
  }
  if (length + 8 > section.mappedSize) {
    fail(`${section.description} declares ${length} payload bytes but only ${section.mappedSize - 8} are mapped`);
  }
  return section.bytes.subarray(8, 8 + length);
}

// --- graph payload validation ----------------------------------------------

type GraphPointer = { readonly offset: number; readonly length: number };

function readPointer(bytes: Uint8Array, offset: number, limit: number, what: string): GraphPointer {
  const pointer = {
    offset: readU32(bytes, offset, `${what} offset`),
    length: readU32(bytes, offset + 4, `${what} length`),
  };
  if (pointer.offset + pointer.length > limit) {
    fail(
      `${what}: range [${pointer.offset}, ${pointer.offset + pointer.length}) exceeds the ${limit}-byte graph region`,
    );
  }
  return pointer;
}

const utf8 = new TextDecoder("utf-8", { fatal: true });

function readString(bytes: Uint8Array, pointer: GraphPointer, limit: number, what: string): string {
  // An absent string is serialized as an all-zero `StringPointer`; the pinned
  // reader returns an empty `ZStr` for it without inspecting a terminator.
  if (pointer.length === 0) return "";
  if (pointer.offset + pointer.length + 1 > limit) {
    fail(
      `${what}: range [${pointer.offset}, ${pointer.offset + pointer.length + 1}) exceeds the ${limit}-byte graph region`,
    );
  }
  if (bytes[pointer.offset + pointer.length] !== 0) {
    fail(`${what}: missing NUL terminator after ${pointer.length} bytes at offset ${pointer.offset}`);
  }
  try {
    return utf8.decode(bytes.subarray(pointer.offset, pointer.offset + pointer.length));
  } catch {
    fail(`${what}: bytes at offset ${pointer.offset} are not valid UTF-8`);
  }
}

function enumName(names: Record<number, string>, value: number, what: string): string {
  const name = names[value];
  if (name === undefined) fail(`${what}: unknown value ${value}`);
  return name;
}

/**
 * Validates a serialized standalone module graph payload and returns its parsed
 * records.
 *
 * The pinned serializer does not bounds-check its own pointers when it reads the
 * graph back (`slice_to` only debug-asserts), so every length and offset is
 * re-validated here against the declared byte count.
 *
 * Rejects, rather than silently embedding:
 * - truncated or non-terminated payloads (trailer, offsets, string pointers),
 * - graph keys outside the target runtime's prefix (a graph serialized for
 *   another platform would never resolve at runtime),
 * - JSC bytecode and its module-info blob, which the runtime mutates in place
 *   and which version-locks the graph to one engine build,
 * - native addons (`.node`, `.so`, `.dll`, `.dylib`, `napi` loader), which the
 *   runtime can only satisfy by writing the module to a temporary file.
 */
type GraphRecord = {
  readonly name: string;
  readonly contents: GraphPointer;
  readonly sourcemap: GraphPointer;
  readonly bytecode: GraphPointer;
  readonly moduleInfo: GraphPointer;
  readonly bytecodeOriginPath: string;
  readonly encoding: number;
  readonly loader: number;
  readonly moduleFormat: number;
  readonly side: number;
};

type GraphLayout = {
  readonly byteCount: number;
  readonly entryPointId: number;
  readonly flags: number;
  readonly compileExecArgv: string;
  readonly records: readonly GraphRecord[];
};

/**
 * Structurally validates a serialized graph and reads its records.
 *
 * The pinned reader does not bounds-check the pointers it follows when it loads
 * a graph back (`slice_to` only debug-asserts), so every length, offset, and
 * terminator is re-validated here against the declared byte count. This is also
 * what identifies a graph section in a compiled image.
 */
function readGraphLayout(payload: Uint8Array): GraphLayout {
  const trailerOffset = payload.length - GRAPH_TRAILER.length;
  const offsetsOffset = trailerOffset - GRAPH_OFFSETS_SIZE;
  if (offsetsOffset < 0) {
    fail(
      `graph payload of ${payload.length} bytes is too small to be a standalone module graph (needs at least ${GRAPH_OFFSETS_SIZE + GRAPH_TRAILER.length})`,
    );
  }
  if (!matchesBytes(payload, trailerOffset, GRAPH_TRAILER)) {
    fail("graph payload: missing or misplaced '\n---- Bun! ----\n' trailer");
  }

  const byteCount = readU64(payload, offsetsOffset, "graph offsets byte count");
  if (byteCount !== offsetsOffset) {
    fail(`graph payload: offsets declare ${byteCount} graph bytes but the payload holds ${offsetsOffset}`);
  }

  const modules = readPointer(payload, offsetsOffset + 8, byteCount, "graph modules blob");
  const entryPointId = readU32(payload, offsetsOffset + 16, "graph entry point id");
  const compileExecArgv = readPointer(payload, offsetsOffset + 20, byteCount, "graph compile exec argv");
  const flags = readU32(payload, offsetsOffset + 28, "graph flags");

  if (modules.length === 0) fail("graph payload: modules blob is empty");
  if (modules.length % GRAPH_FILE_SIZE !== 0) {
    fail(`graph payload: modules blob is ${modules.length} bytes, not a multiple of ${GRAPH_FILE_SIZE}`);
  }
  const recordCount = modules.length / GRAPH_FILE_SIZE;
  if (entryPointId >= recordCount) {
    fail(`graph payload: entry point id ${entryPointId} is outside the ${recordCount} graph files`);
  }

  const records: GraphRecord[] = [];
  for (let index = 0; index < recordCount; index += 1) {
    const record = modules.offset + index * GRAPH_FILE_SIZE;
    const label = `graph file ${index}`;
    records.push({
      name: readString(payload, readPointer(payload, record, byteCount, `${label} name`), byteCount, `${label} name`),
      contents: readPointer(payload, record + 8, byteCount, `${label} contents`),
      sourcemap: readPointer(payload, record + 16, byteCount, `${label} sourcemap`),
      bytecode: readPointer(payload, record + 24, byteCount, `${label} bytecode`),
      moduleInfo: readPointer(payload, record + 32, byteCount, `${label} module info`),
      bytecodeOriginPath: readString(
        payload,
        readPointer(payload, record + 40, byteCount, `${label} bytecode origin path`),
        byteCount,
        `${label} bytecode origin path`,
      ),
      encoding: payload[record + 48]!,
      loader: payload[record + 49]!,
      moduleFormat: payload[record + 50]!,
      side: payload[record + 51]!,
    });
  }

  return {
    byteCount,
    entryPointId,
    flags,
    compileExecArgv: readString(payload, compileExecArgv, byteCount, "graph compile exec argv"),
    records,
  };
}

/**
 * Validates a serialized standalone module graph payload against the target
 * runtime and returns its records.
 *
 * Rejects, rather than silently embedding:
 * - graph keys outside the target runtime's prefix (a graph serialized for
 *   another platform would never resolve at runtime),
 * - JSC bytecode and its module-info blob, which the runtime mutates in place
 *   and which version-locks the graph to one engine build,
 * - native addons (`.node`, `.so`, `.dll`, `.dylib`, `napi` loader), which the
 *   runtime can only satisfy by writing the module to a temporary file.
 */
export function parseStandaloneGraph(payload: Uint8Array, target: EmbeddedGraphTarget): EmbeddedGraph {
  const spec = targetSpec(target);
  const layout = readGraphLayout(payload);

  const files: EmbeddedGraphFile[] = [];
  const seen = new Set<string>();
  let entry = "";
  for (const [index, record] of layout.records.entries()) {
    const name = record.name;
    const loader = enumName(LOADER_NAMES, record.loader, `graph file ${index} loader`);
    const encoding = enumName(ENCODING_NAMES, record.encoding, `graph file ${index} encoding`);
    const moduleFormat = enumName(MODULE_FORMAT_NAMES, record.moduleFormat, `graph file ${index} module format`);
    const side = enumName(FILE_SIDE_NAMES, record.side, `graph file ${index} side`);

    if (!name.startsWith(spec.prefix)) {
      fail(
        `graph file ${index} is keyed ${JSON.stringify(name)}, outside the ${JSON.stringify(spec.prefix)} prefix of ${target}; ` +
          `the graph was serialized for a different platform`,
      );
    }
    if (seen.has(name)) fail(`graph file ${index} duplicates the graph key ${JSON.stringify(name)}`);
    seen.add(name);

    if (record.bytecode.length > 0) {
      fail(
        `graph file ${JSON.stringify(name)} carries ${record.bytecode.length} bytes of JSC bytecode; ` +
          `the embedded runtime mutates bytecode in place, so it is packaged without precompiled bytecode`,
      );
    }
    if (record.moduleInfo.length > 0) {
      fail(`graph file ${JSON.stringify(name)} carries a bytecode module-info blob, which is only valid with bytecode`);
    }
    if (record.loader === LOADER_NAPI) {
      fail(
        `graph file ${JSON.stringify(name)} is a native addon (napi loader), which the runtime can only load by ` +
          `writing it to a temporary file; native dependencies must be integrated statically instead`,
      );
    }
    const lower = name.toLowerCase();
    for (const extension of NATIVE_EXTENSIONS) {
      if (lower.endsWith(extension)) {
        fail(
          `graph file ${JSON.stringify(name)} would require extracting a native module (${extension}) to disk at run time`,
        );
      }
    }

    if (index === layout.entryPointId) entry = name;
    files.push({
      name,
      isEntryPoint: index === layout.entryPointId,
      loader,
      encoding,
      moduleFormat,
      side,
      contentsLength: record.contents.length,
      sourcemapLength: record.sourcemap.length,
      bytecodeOriginPath: record.bytecodeOriginPath,
    });
  }

  return {
    target,
    payloadLength: payload.length,
    byteCount: layout.byteCount,
    flags: layout.flags,
    compileExecArgv: layout.compileExecArgv,
    entry,
    files,
  };
}

// --- serialization ----------------------------------------------------------

function requireAbsoluteFile(path: string, label: string): string {
  if (typeof path !== "string" || path.length === 0) fail(`${label} is required`);
  if (!isAbsolute(path)) fail(`${label} must be an absolute path, got ${JSON.stringify(path)}`);
  return path;
}

async function run(command: readonly string[], cwd: string): Promise<void> {
  const child = Bun.spawn([...command], { cwd, stdin: "ignore", stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  if (exitCode !== 0) {
    const details = [stdout.trim(), stderr.trim()].filter((part) => part.length > 0).join("\n");
    fail(`${command[0]} ${command[1]} failed with exit code ${exitCode}${details.length > 0 ? `:\n${details}` : ""}`);
  }
}

/**
 * Fails closed when the supplied serializer is not the pinned revision.
 *
 * `bun --revision` prints `<version>+<commit>`; the commit token must be a real
 * hexadecimal abbreviation of the pinned commit. There is deliberately no way
 * to disable this check: the graph payload carries no format version, so an
 * unpinned serializer can produce a payload the embedded runtime misreads.
 */
async function verifySerializerRevision(bun: string, revision: string): Promise<void> {
  const child = Bun.spawn([bun, "--revision"], { stdin: "ignore", stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  const reported = stdout.trim();
  if (exitCode !== 0) {
    fail(`serializer ${bun} failed to report its revision (exit ${exitCode}): ${stderr.trim()}`);
  }
  const match = /^(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)\+([0-9a-fA-F]+)$/.exec(reported);
  if (match === null) {
    fail(`serializer ${bun} reported ${JSON.stringify(reported)}, which is not a Bun version plus commit revision`);
  }
  const commit = match[2]!.toLowerCase();
  if (!revision.startsWith(commit)) {
    fail(
      `serializer ${bun} is ${reported} but the pinned revision is ${revision}; ` +
        `the embedded runtime must be serialized by a Bun built from the pin`,
    );
  }
}

function renderRustSource(
  graph: EmbeddedGraph,
  inner: TargetSpec,
  sectionLength: number,
  sectionBytesName: string,
): string {
  const header = `// @generated by scripts/bun-embedded-bundle.ts -- do not edit.
//
// Embedded Bun standalone module graph (${graph.target}).
//
// ${graph.files.length} graph files, ${graph.payloadLength} payload bytes, entry ${graph.entry}.
//
// ${JSON.stringify(sectionBytesName)} (${sectionLength} bytes) sits next to this file and is read at compile
// time by \`include_bytes!\`. Those bytes are the exact content of the runtime's
// graph section: an 8-byte little-endian payload length followed by the
// serialized module graph. The payload is mapped from the executable image and
// is never unpacked to disk or rewritten, so the section must not be
// dead-stripped, merged with another input, or preceded by any other data. The
// packager rejects bytecode and native addons precisely so that nothing has to
// be mutable or extracted at run time.
//
${
  inner.container === "pe"
    ? `// Windows: the runtime locates the section by its exact NUL-padded name
// (\`.bun\`) in the section table of the main image and reads the length at
// VirtualAddress + 8, so no symbol contract is involved. The static is
// naturally aligned to the u64 header and must stay at section offset 0.`
    : `// macOS: the runtime reads the payload through \`BUN_COMPILED\` in the
// \`__BUN,__bun\` section (the Bun C graph declares that placeholder weak and
// retained), so this file defines the same symbol strongly at the section's
// 16 KiB alignment, and the payload lives at symbol + 8.`
}
//
// The static is included by the final application crate, so it is emitted into
// that crate's object file and cannot be dropped by archive extraction;
// \`#[used]\` keeps it through dead stripping.
`;

  const body = `/// Graph identity of the packaged application entry point.
///
/// This is a key in the embedded module graph, not a filesystem path: pass it
/// unchanged to the embedded runtime, which resolves it without touching the
/// filesystem.
pub const BUN_EMBEDDED_ENTRY: &str = ${JSON.stringify(graph.entry)};

${
  inner.container === "pe"
    ? `/// Serialized graph section content, referenced by the section alone.
#[repr(C, align(8))]
struct SectionBlob {
    bytes: [u8; ${sectionLength}],
}

#[used]
#[allow(dead_code)]
#[unsafe(link_section = ${JSON.stringify(inner.linkSection)})]
static BUN_EMBEDDED_GRAPH: SectionBlob = SectionBlob {
    bytes: *include_bytes!(${JSON.stringify(sectionBytesName)}),
};
`
    : `/// Serialized graph section content. The runtime reads the payload length from
/// the first field through the \`BUN_COMPILED\` symbol, so the blob keeps the
/// section's 16 KiB alignment and its leading u64 intact.
#[repr(C, align(16384))]
pub struct SectionBlob {
    pub bytes: [u8; ${sectionLength}],
}

#[unsafe(no_mangle)]
#[used]
#[allow(dead_code)]
#[unsafe(link_section = ${JSON.stringify(inner.linkSection)})]
pub static BUN_COMPILED: SectionBlob = SectionBlob {
    bytes: *include_bytes!(${JSON.stringify(sectionBytesName)}),
};
`
}
`;

  return `${header}\n${body}`;
}

/**
 * Serializes the built application entry into Bun's standalone module graph and
 * writes the Rust include source that carries it in the final executable.
 *
 * The serializer is run as a build-time tool on the pinned Bun revision only.
 */
export async function packageEmbeddedGraph(options: PackageEmbeddedGraphOptions): Promise<PackagedEmbeddedGraph> {
  const bun = requireAbsoluteFile(options.bun, "bun");
  const entry = requireAbsoluteFile(options.entry, "entry");
  const outDir = requireAbsoluteFile(options.outDir, "outDir");
  const assets = (options.assets ?? []).map((asset, index) => requireAbsoluteFile(asset, `assets[${index}]`));
  const workers = (options.workers ?? []).map((worker, index) => requireAbsoluteFile(worker, `workers[${index}]`));
  const spec = targetSpec(options.target);

  let baseExecutable: string | undefined;
  if (options.baseExecutable !== undefined) {
    baseExecutable = requireAbsoluteFile(options.baseExecutable, "baseExecutable");
  } else if (spec.host !== `${process.platform}-${process.arch}`) {
    fail(
      `target ${options.target} does not match this host (${process.platform}-${process.arch}); pass the target-platform ` +
        `Bun executable as baseExecutable instead of letting the serializer download a base that the revision pin does not cover`,
    );
  }

  await verifySerializerRevision(bun, await pinnedBunRevision());
  await mkdir(outDir, { recursive: true });

  // The serializer keys the entry point after the output file's basename, with
  // a target-format suffix (`.exe`) appended for PE targets and then removed
  // again from the key, so the intermediate is named after the entry itself.
  // That keeps the packaged graph identity derived from the application rather
  // than from a temporary file name.
  const entryName = basename(entry);
  const work = await mkdtemp(join(outDir, ".bun-embedded-"));
  const intermediate = join(work, entryName);
  const executable = spec.container === "pe" && !intermediate.endsWith(".exe") ? `${intermediate}.exe` : intermediate;
  const command = [
    bun,
    "build",
    entry,
    ...workers,
    "--compile",
    "--target",
    options.target,
    "--conditions=browser",
    "--outfile",
    intermediate,
    ...assets.flatMap((asset) => ["--asset", asset]),
    ...(baseExecutable === undefined ? [] : ["--compile-executable-path", baseExecutable]),
  ];

  try {
    // The working directory is a fresh temporary directory: no `bunfig.toml`,
    // `tsconfig.json`, or `package.json` of the caller is picked up, and the
    // serializer cannot silently adopt a base executable named after the target
    // that happens to sit in the current directory.
    await run(command, work);
    const payload = extractGraphPayload(await readFile(executable), options.target);
    const graph = parseStandaloneGraph(payload, options.target);

    const expectedEntry = `${spec.prefix}${entryName}`;
    if (graph.entry !== expectedEntry) {
      fail(
        `graph entry point is ${JSON.stringify(graph.entry)} but ${JSON.stringify(expectedEntry)} was expected for ` +
          `${JSON.stringify(entry)}; refusing to return an identity that does not correspond to the packaged application entry`,
      );
    }

    return await writeEmbeddedGraphArtifacts(graph, payload, outDir);
  } finally {
    await rm(work, { recursive: true, force: true });
  }
}

/**
 * Writes the Rust include source and the section bytes for a validated graph.
 *
 * Separate from serialization so that an already-serialized payload can be
 * emitted without running the pinned serializer again.
 */
export async function writeEmbeddedGraphArtifacts(
  graph: EmbeddedGraph,
  payload: Uint8Array,
  outDir: string,
): Promise<PackagedEmbeddedGraph> {
  if (payload.length !== graph.payloadLength) {
    fail(`payload is ${payload.length} bytes but the parsed graph declares ${graph.payloadLength}`);
  }
  if (payload.length + 8 > 0xffff_ffff) {
    fail(
      `graph payload of ${payload.length} bytes plus its 8-byte header exceeds the 32-bit section size of the ` +
        `container format`,
    );
  }
  const section = new Uint8Array(payload.length + 8);
  new DataView(section.buffer).setBigUint64(0, BigInt(payload.length), true);
  section.set(payload, 8);

  const rustSource = join(outDir, RUST_SOURCE_NAME);
  await writeFile(join(outDir, SECTION_BYTES_NAME), section);
  await writeFile(rustSource, renderRustSource(graph, targetSpec(graph.target), section.length, SECTION_BYTES_NAME));
  return { rustSource, entry: graph.entry, graphSha256: createHash("sha256").update(payload).digest("hex") };
}

if (import.meta.main) {
  try {
    const { values } = parseArgs({
      options: {
        bun: { type: "string" },
        entry: { type: "string" },
        target: { type: "string" },
        "out-dir": { type: "string" },
        asset: { type: "string", multiple: true },
        worker: { type: "string", multiple: true },
        "base-executable": { type: "string" },
      },
      allowPositionals: false,
    });
    if (values.bun === undefined) fail("--bun is required");
    if (values.entry === undefined) fail("--entry is required");
    if (values.target === undefined) fail("--target is required");
    if (values["out-dir"] === undefined) fail("--out-dir is required");
    // CLI input is validated against the supported target table by packageEmbeddedGraph.
    const target = values.target as EmbeddedGraphTarget;
    const result = await packageEmbeddedGraph({
      bun: values.bun,
      entry: values.entry,
      target,
      outDir: values["out-dir"],
      assets: values.asset,
      workers: values.worker,
      baseExecutable: values["base-executable"],
    });
    console.log(JSON.stringify(result));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
