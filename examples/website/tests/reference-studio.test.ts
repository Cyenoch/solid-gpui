import { expect, test } from "bun:test";
import { createStudioProject, editClip, moveClip, reorderTrack } from "../src/showcase/reference/model";

test("timeline edits keep clip identity, clamp placement, and reorder tracks without losing clips", () => {
  const project = createStudioProject(240);
  const moved = moveClip(project, "clip-0-a", "track-2", 118);
  expect(moved.tracks[0]!.clips.map((clip) => clip.id)).toEqual(["clip-0-b"]);
  expect(moved.tracks[2]!.clips.find((clip) => clip.id === "clip-0-a")!.start).toBe(108);
  const edited = editClip(moved, "clip-0-a", { label: "字幕 😀 é", start: 8, duration: 6 });
  const reordered = reorderTrack(edited, "track-2", "track-0");
  expect(reordered.tracks.slice(0, 3).map((track) => track.id)).toEqual(["track-2", "track-0", "track-1"]);
  expect(reordered.tracks[0]!.clips.find((clip) => clip.id === "clip-0-a")).toEqual({
    id: "clip-0-a",
    label: "字幕 😀 é",
    start: 8,
    duration: 6,
  });
  expect(reordered.tracks.flatMap((track) => track.clips)).toHaveLength(480);
  expect(() => editClip(project, "clip-0-a", { label: "", start: NaN, duration: 1 })).toThrow();
});
