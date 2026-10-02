import { readdir, stat } from "node:fs/promises";
import { join } from "node:path";

const decoder = new TextDecoder();
const encoder = new TextEncoder();

/** Bun owns archive encoding/compression; restore executable modes it currently defaults to 0644. */
export async function writePortableArchive(root: string, prefix: string, output: string): Promise<void> {
  const files: Record<string, Uint8Array<ArrayBuffer>> = {};
  const modes = new Map<string, number>();
  const collect = async (directory: string, relative: string): Promise<void> => {
    files[relative + "/"] = new Uint8Array();
    modes.set(relative + "/", 0o755);
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      const name = `${relative}/${entry.name}`;
      if (entry.isDirectory()) await collect(path, name);
      else if (entry.isFile()) {
        files[name] = await Bun.file(path).bytes();
        modes.set(name, (await stat(path)).mode & 0o111 ? 0o755 : 0o644);
      } else throw new Error(`Portable archives require regular files: ${path}`);
    }
  };
  await collect(root, prefix);
  const bytes = await new Bun.Archive(files).bytes();
  let extendedPath: string | undefined;
  for (let offset = 0; offset + 512 <= bytes.length;) {
    const header = bytes.subarray(offset, offset + 512);
    if (header.every((byte) => byte === 0)) break;
    const text = (start: number, end: number) => decoder.decode(header.subarray(start, end)).split("\0")[0]!;
    const size = Number.parseInt(text(124, 136), 8);
    if (!Number.isSafeInteger(size) || size < 0 || offset + 512 + size > bytes.length)
      throw new Error("Bun produced an invalid archive size");
    const kind = text(156, 157);
    if (kind === "x") {
      const records = bytes.subarray(offset + 512, offset + 512 + size);
      for (let index = 0; index < records.length;) {
        const space = records.indexOf(32, index);
        const length = Number.parseInt(decoder.decode(records.subarray(index, space)), 10);
        if (
          space < index ||
          !Number.isSafeInteger(length) ||
          length <= space - index ||
          index + length > records.length
        )
          throw new Error("Bun produced an invalid extended archive header");
        const record = decoder.decode(records.subarray(space + 1, index + length - 1));
        if (record.startsWith("path=")) extendedPath = record.slice(5);
        index += length;
      }
    } else if (kind === "0" || kind === "") {
      const prefix = text(345, 500);
      const path = extendedPath ?? `${prefix ? prefix + "/" : ""}${text(0, 100)}`;
      extendedPath = undefined;
      const mode = modes.get(path);
      if (mode === undefined) throw new Error(`Bun produced an unknown archive entry: ${path}`);
      header.set(encoder.encode(mode.toString(8).padStart(7, "0") + "\0"), 100);
      if (path.endsWith("/")) header[156] = 53;
      header.fill(32, 148, 156);
      const checksum = header.reduce((sum, byte) => sum + byte, 0);
      header.set(encoder.encode(checksum.toString(8).padStart(6, "0") + "\0 "), 148);
    } else if (kind !== "5") throw new Error(`Bun produced an unsupported archive entry: ${kind}`);
    offset += 512 + Math.ceil(size / 512) * 512;
  }
  await Bun.Archive.write(output, new Bun.Archive(bytes, { compress: "gzip" }));
}
