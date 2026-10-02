import { batch, createMemo, createSignal } from "@solid-gpui/core/runtime";
import { createHistory, createStudioProject, editClip, moveClip, reorderTrack, type HistoryEntry } from "./model";

export function createReferenceStudioState() {
  const [project, setProject] = createSignal(createStudioProject());
  const [history, setHistory] = createSignal(createHistory());
  const [selectedClipId, setSelectedClipId] = createSignal("clip-0-a");
  const [selectedHistoryId, setSelectedHistoryId] = createSignal("history-0");
  const [title, setTitle] = createSignal("Opening 1");
  const [start, setStart] = createSignal("4");
  const [duration, setDuration] = createSignal("12");
  const [draft, setDraft] = createSignal("Review 字幕 😀 é");
  const [query, setQuery] = createSignal("");
  const [status, setStatus] = createSignal("Ready for review");
  const [inspectorOpen, setInspectorOpen] = createSignal(false);
  const [zoom, setZoom] = createSignal(1);
  const [pan, setPan] = createSignal(0);
  const selectedClip = createMemo(() =>
    project()
      .tracks.flatMap((track) => track.clips)
      .find((clip) => clip.id === selectedClipId())!,
  );
  const selectedHistory = createMemo(() => history().find((entry) => entry.id === selectedHistoryId())!);
  const visibleHistory = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return needle
      ? history().filter((entry) =>
          `${entry.author} ${entry.subject} ${entry.paragraphs.join(" ")}`.toLowerCase().includes(needle),
        )
      : history();
  });
  const selectClip = (id: string) => {
    const clip = project()
      .tracks.flatMap((track) => track.clips)
      .find((item) => item.id === id);
    if (!clip) throw new Error(`Unknown clip: ${id}`);
    batch(() => {
      setSelectedClipId(id);
      setTitle(clip.label);
      setStart(String(clip.start));
      setDuration(String(clip.duration));
    });
  };
  const apply = () => {
    try {
      setProject(
        editClip(project(), selectedClipId(), { label: title(), start: Number(start()), duration: Number(duration()) }),
      );
      setStatus(`Saved ${selectedClip().label}`);
    } catch (error) {
      setStatus(String(error));
    }
  };
  const drop = (type: string, target: string) => {
    if (type.startsWith("studio-track:")) {
      setProject(reorderTrack(project(), type.slice("studio-track:".length), target));
      setStatus(`Moved ${type.slice("studio-track:".length)} before ${target}`);
    } else if (type.startsWith("studio-clip:")) {
      const id = type.slice("studio-clip:".length);
      const clip = project()
        .tracks.flatMap((track) => track.clips)
        .find((item) => item.id === id);
      if (!clip) throw new Error(`Unknown clip: ${id}`);
      batch(() => {
        setProject(moveClip(project(), id, target, clip.start));
        selectClip(id);
        setStatus(`Moved ${id} to ${target}`);
      });
    }
  };
  let nextNote = 0;
  const post = () => {
    if (!draft().trim()) return;
    const entry: HistoryEntry = {
      id: `note-${nextNote++}`,
      author: "You",
      subject: "New review note",
      paragraphs: [draft()],
    };
    batch(() => {
      setHistory((entries) => [entry, ...entries]);
      setSelectedHistoryId(entry.id);
      setDraft("");
      setQuery("");
      setStatus("Review note posted");
    });
  };
  return {
    project,
    history,
    selectedClip,
    selectedHistory,
    selectedClipId,
    selectedHistoryId,
    setSelectedHistoryId,
    title,
    setTitle,
    start,
    setStart,
    duration,
    setDuration,
    draft,
    setDraft,
    query,
    setQuery,
    visibleHistory,
    status,
    setStatus,
    inspectorOpen,
    setInspectorOpen,
    zoom,
    setZoom,
    pan,
    setPan,
    selectClip,
    apply,
    drop,
    post,
  };
}
export type ReferenceStudioState = ReturnType<typeof createReferenceStudioState>;
