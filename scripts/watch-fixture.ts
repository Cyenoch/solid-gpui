import { writeFile } from "node:fs/promises";

/** Vite coalesces change events for 50ms; distinct fixture edits must cross that window. */
export async function saveWatchedFile(path: string, contents: string): Promise<void> {
  await Bun.sleep(100);
  await writeFile(path, contents);
}
