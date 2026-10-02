// Immutable application data and bounded timeline edits.
export interface Clip {
  readonly id: string;
  readonly label: string;
  readonly start: number;
  readonly duration: number;
}
export interface Track {
  readonly id: string;
  readonly name: string;
  readonly clips: readonly Clip[];
}
export interface StudioProject {
  readonly duration: number;
  readonly tracks: readonly Track[];
}
export interface HistoryEntry {
  readonly id: string;
  readonly author: string;
  readonly subject: string;
  readonly paragraphs: readonly string[];
}
export function createStudioProject(count = 240): StudioProject {
  return {
    duration: 120,
    tracks: Array.from({ length: count }, (_, index) => ({
      id: `track-${index}`,
      name: `${["Picture", "Dialogue", "Music", "Captions"][index % 4]} ${index + 1}`,
      clips: [
        { id: `clip-${index}-a`, label: `Opening ${index + 1}`, start: 4, duration: 12 },
        { id: `clip-${index}-b`, label: `Take ${index + 1}`, start: 24 + (index % 6) * 4, duration: 10 },
      ],
    })),
  };
}
function locate(project: StudioProject, id: string) {
  for (const track of project.tracks) {
    const clip = track.clips.find((item) => item.id === id);
    if (clip) return { track, clip };
  }
  throw new Error(`Unknown clip: ${id}`);
}
export function editClip(project: StudioProject, id: string, edit: Omit<Clip, "id">): StudioProject {
  locate(project, id);
  if (!edit.label.trim() || !Number.isFinite(edit.start) || !Number.isFinite(edit.duration) || edit.duration < 1)
    throw new Error("Enter a title, a finite start, and a duration of at least one second.");
  const duration = Math.min(project.duration, edit.duration);
  const clip = {
    id,
    label: edit.label,
    start: Math.max(0, Math.min(project.duration - duration, edit.start)),
    duration,
  };
  return {
    ...project,
    tracks: project.tracks.map((track) =>
      track.clips.some((item) => item.id === id)
        ? { ...track, clips: track.clips.map((item) => (item.id === id ? clip : item)) }
        : track,
    ),
  };
}
export function moveClip(project: StudioProject, id: string, destination: string, start: number): StudioProject {
  const { clip } = locate(project, id);
  if (!project.tracks.some((track) => track.id === destination)) throw new Error(`Unknown track: ${destination}`);
  const edited = editClip(project, id, { ...clip, start });
  const next = locate(edited, id).clip;
  return {
    ...edited,
    tracks: edited.tracks.map((track) => {
      if (track.id === destination) return { ...track, clips: [...track.clips.filter((item) => item.id !== id), next] };
      return track.clips.some((item) => item.id === id)
        ? { ...track, clips: track.clips.filter((item) => item.id !== id) }
        : track;
    }),
  };
}
export function reorderTrack(project: StudioProject, id: string, before: string): StudioProject {
  if (id === before) return project;
  const source = project.tracks.find((track) => track.id === id);
  if (!source || !project.tracks.some((track) => track.id === before)) throw new Error("Unknown reorder target");
  const tracks = project.tracks.filter((track) => track.id !== id);
  tracks.splice(
    tracks.findIndex((track) => track.id === before),
    0,
    source,
  );
  return { ...project, tracks };
}
export function createHistory(count = 10_000): readonly HistoryEntry[] {
  return Array.from({ length: count }, (_, index) => ({
    id: `history-${index}`,
    author: ["Maya", "Noah", "Ari", "Lin"][index % 4]!,
    subject: `Review ${index + 1}: ${["Opening cut", "Sound mix", "Caption pass", "Delivery notes"][index % 4]}`,
    paragraphs: [
      `Review ${index + 1} — The opening cut is ready for another look.`,
      ...Array.from(
        { length: (index % 3) + 1 },
        (_, paragraph) => `Note ${paragraph + 1}: Keep the dialogue clear and the transition gentle. 字幕 😀 é`,
      ),
    ],
  }));
}
