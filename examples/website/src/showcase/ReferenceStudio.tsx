import {
  Pressable,
  Text,
  TextInput,
  View,
  VirtualList,
  type SolidChild,
  type Style,
  type VirtualListHandle,
} from "@solid-gpui/core";
import { For, onCleanup } from "@solid-gpui/core/runtime";
import { createReferenceStudioState, type ReferenceStudioState } from "./reference/state";
import type { HistoryEntry, Track } from "./reference/model";

export { createReferenceStudioState };
export interface StudioPalette {
  background: string;
  panel: string;
  raised: string;
  border: string;
  text: string;
  muted: string;
  accent: string;
}
export const studioPalette: StudioPalette = {
  background: "#131217",
  panel: "#1B1A20",
  raised: "#222127",
  border: "#3C3944",
  text: "#ECEAF1",
  muted: "#B5B1BF",
  accent: "#D4688C",
};
export interface ReferenceStudioProps {
  state: ReferenceStudioState;
  width: number;
  height?: number;
  view: "timeline" | "history";
  navigate: (view: "timeline" | "history") => void;
  copyText: (text: string) => Promise<void>;
  palette?: StudioPalette;
  /** A generated NativeView consumer owns its resource lifetime inside this slot. */
  preview?: () => SolidChild;
  /** Acceptance observes row owner lifetime, never substitutes a virtual range. */
  onRowLifetime?: (kind: "track" | "history", id: string, mounted: boolean, owner: object) => void;
  onListHandle?: (kind: "track" | "history", handle: VirtualListHandle | undefined) => void;
}

function Action(props: {
  label: string;
  text: string;
  onPress: () => void;
  palette: StudioPalette;
  active?: boolean;
  disabled?: boolean;
}) {
  return (
    <Pressable
      accessibilityLabel={props.label}
      accessibilityRole="button"
      focusable
      disabled={props.disabled}
      onPress={props.onPress}
      style={{
        padding: 7,
        minHeight: 32,
        flexShrink: 0,
        borderRadius: 5,
        borderWidth: 1,
        borderColor: props.active ? props.palette.accent : props.palette.border,
        backgroundColor: props.palette.raised,
      }}
    >
      <Text style={{ fontSize: 12, lineHeight: 18, color: props.palette.text }}>{props.text}</Text>
    </Pressable>
  );
}

export function ReferenceStudio(props: ReferenceStudioProps) {
  const s = props.state;
  const p = () => props.palette ?? studioPalette;
  const height = () => props.height ?? 620;
  const narrow = () => props.width < 720;
  const inspectorWidth = () => (narrow() ? (s.inspectorOpen() ? Math.min(240, props.width - 120) : 0) : 220);
  const contentWidth = () => Math.max(120, props.width - inspectorWidth() - 2);
  const timelineWidth = () =>
    narrow()
      ? props.view === "timeline"
        ? contentWidth()
        : 0
      : Math.floor(contentWidth() * (props.view === "timeline" ? 0.62 : 0.38));
  const historyWidth = () => contentWidth() - timelineWidth();
  let tracks: VirtualListHandle | undefined;
  let history: VirtualListHandle | undefined;
  let active = true;
  onCleanup(() => {
    active = false;
    tracks = undefined;
    history = undefined;
    props.onListHandle?.("track", undefined);
    props.onListHandle?.("history", undefined);
  });
  const command = (pending: Promise<void>) => {
    void pending.catch((error) => {
      if (active) s.setStatus(String(error));
    });
  };
  const inputStyle = (): Style => ({
    height: 34,
    padding: 6,
    color: p().text,
    backgroundColor: p().background,
    borderWidth: 1,
    borderColor: p().border,
    fontSize: 13,
    lineHeight: 20,
  });
  const paneStyle = (width: number): Style => ({
    width,
    height: height() - 96,
    minWidth: 0,
    minHeight: 0,
    flexShrink: 0,
    overflow: "hidden",
    flexDirection: "column",
    backgroundColor: p().panel,
  });

  function TrackRow(row: { track: Track }) {
    const owner = {};
    props.onRowLifetime?.("track", row.track.id, true, owner);
    onCleanup(() => props.onRowLifetime?.("track", row.track.id, false, owner));
    const gridWidth = () => Math.max(80, timelineWidth() - 100);
    const scale = () => (gridWidth() / 120) * s.zoom();
    return (
      <View
        accessibilityLabel={`studio.track.${row.track.id}`}
        accessibilityRole="listitem"
        onDrop={(type) => s.drop(type, row.track.id)}
        style={{ height: 64, flexShrink: 0, flexDirection: "row", borderBottomWidth: 1, borderColor: p().border }}
      >
        <Pressable
          accessibilityLabel={`studio.reorder.${row.track.id}`}
          draggable={{ type: `studio-track:${row.track.id}` }}
          style={{ width: 100, flexShrink: 0, padding: 8, backgroundColor: p().raised, cursor: "grab" }}
        >
          <Text style={{ color: p().text, fontSize: 11, lineHeight: 16 }}>{row.track.name}</Text>
          <Text style={{ color: p().muted, fontSize: 10, lineHeight: 14 }}>Drag to reorder</Text>
        </Pressable>
        <View style={{ width: gridWidth(), height: 64, position: "relative", overflow: "hidden" }}>
          <For each={row.track.clips}>
            {(clip) => (
              <Pressable
                accessibilityLabel={`studio.clip.${clip.id}`}
                accessibilitySelected={s.selectedClipId() === clip.id}
                draggable={{ type: `studio-clip:${clip.id}` }}
                onPress={() => s.selectClip(clip.id)}
                style={{
                  position: "absolute",
                  left: (clip.start - s.pan()) * scale(),
                  top: 10,
                  width: Math.max(24, clip.duration * scale()),
                  height: 42,
                  padding: 4,
                  borderRadius: 4,
                  borderWidth: s.selectedClipId() === clip.id ? 2 : 1,
                  borderColor: s.selectedClipId() === clip.id ? p().accent : p().border,
                  backgroundColor: p().raised,
                  overflow: "hidden",
                  cursor: "grab",
                }}
              >
                <Text style={{ color: p().text, fontSize: 11, lineHeight: 15, lineClamp: 1, textOverflow: "ellipsis" }}>
                  {clip.label}
                </Text>
              </Pressable>
            )}
          </For>
        </View>
      </View>
    );
  }
  function HistoryRow(row: { entry: HistoryEntry }) {
    const owner = {};
    props.onRowLifetime?.("history", row.entry.id, true, owner);
    onCleanup(() => props.onRowLifetime?.("history", row.entry.id, false, owner));
    return (
      <View
        accessibilityLabel={`studio.history.${row.entry.id}`}
        accessibilityRole="listitem"
        style={{
          padding: 10,
          gap: 5,
          flexShrink: 0,
          borderBottomWidth: 1,
          borderColor: p().border,
          backgroundColor: s.selectedHistoryId() === row.entry.id ? p().raised : p().panel,
        }}
      >
        <Pressable
          accessibilityLabel={`studio.open.${row.entry.id}`}
          onPress={() => s.setSelectedHistoryId(row.entry.id)}
        >
          <Text style={{ color: p().accent, fontSize: 12, lineHeight: 18 }}>
            {row.entry.author} · {row.entry.subject}
          </Text>
        </Pressable>
        <For each={row.entry.paragraphs}>
          {(paragraph, index) => (
            <Text
              selectable
              accessibilityLabel={`studio.text.${row.entry.id}.${index()}`}
              style={{ color: p().text, fontSize: 12, lineHeight: 18 }}
            >
              {paragraph}
            </Text>
          )}
        </For>
      </View>
    );
  }
  return (
    <View
      accessibilityLabel="studio.shell"
      style={{
        width: props.width,
        height: height(),
        minWidth: 0,
        minHeight: 0,
        flexDirection: "column",
        backgroundColor: p().background,
        color: p().text,
      }}
    >
      <View
        accessibilityLabel="studio.navigation"
        style={{ height: 54, padding: 8, flexDirection: "row", alignItems: "center", gap: 8, flexShrink: 0 }}
      >
        <Action
          label="studio.nav.timeline"
          text="Timeline"
          active={props.view === "timeline"}
          onPress={() => props.navigate("timeline")}
          palette={p()}
        />
        <Action
          label="studio.nav.history"
          text="History"
          active={props.view === "history"}
          onPress={() => props.navigate("history")}
          palette={p()}
        />
        <Text style={{ flexGrow: 1, minWidth: 0, color: p().muted, fontSize: 12 }}>Cut review</Text>
        <Action
          label="studio.inspector.toggle"
          text="Inspector"
          onPress={() => s.setInspectorOpen(!s.inspectorOpen())}
          palette={p()}
        />
      </View>
      <View style={{ flexDirection: "row", height: height() - 96, minHeight: 0, gap: 1 }}>
        <View accessibilityLabel="studio.timeline.pane" style={paneStyle(timelineWidth())}>
          <View style={{ height: 44, padding: 6, flexDirection: "row", gap: 6, overflow: "hidden", flexShrink: 0 }}>
            <Action
              label="studio.tracks.first"
              text="First"
              onPress={() => tracks && command(tracks.scrollToIndex(0))}
              palette={p()}
            />
            <Action
              label="studio.tracks.last"
              text="Last"
              onPress={() => tracks && command(tracks.scrollToEnd())}
              palette={p()}
            />
            <Action
              label="studio.zoom"
              text={`${s.zoom()}×`}
              onPress={() => s.setZoom(s.zoom() === 1 ? 2 : 1)}
              palette={p()}
            />
            <Action
              label="studio.pan"
              text={s.pan() ? "0s" : "30s"}
              onPress={() => s.setPan(s.pan() ? 0 : 30)}
              palette={p()}
            />
          </View>
          <View
            style={{
              height: 28,
              flexShrink: 0,
              paddingLeft: 100,
              flexDirection: "row",
              justifyContent: "space-between",
              overflow: "hidden",
            }}
          >
            {[0, 30, 60, 90].map((second) => (
              <Text style={{ color: p().muted, fontSize: 10 }}>{second / s.zoom() + s.pan()}s</Text>
            ))}
          </View>
          <VirtualList
            accessibilityLabel="studio.tracks"
            ref={(handle) => {
              tracks = handle;
              props.onListHandle?.("track", handle);
            }}
            data={s.project().tracks}
            itemKey={(track) => track.id}
            estimatedItemSize={64}
            initialNumToRender={8}
            overscan={2}
            style={{ height: height() - 168, minHeight: 0, flexDirection: "column" }}
            renderItem={(track) => <TrackRow track={track} />}
          />
        </View>
        <View accessibilityLabel="studio.history.pane" style={paneStyle(historyWidth())}>
          <View style={{ padding: 6, gap: 6, flexShrink: 0 }}>
            <TextInput
              accessibilityLabel="studio.history.search"
              placeholder="Search reviews"
              value={s.query()}
              onChangeText={s.setQuery}
              style={inputStyle()}
            />
            <View style={{ flexDirection: "row", gap: 6 }}>
              <Action
                label="studio.history.first"
                text="First"
                onPress={() => history && command(history.scrollToIndex(0))}
                palette={p()}
              />
              <Action
                label="studio.history.last"
                text="Last"
                onPress={() => history && command(history.scrollToEnd())}
                palette={p()}
              />
              <Text accessibilityLabel="studio.history.count" style={{ color: p().muted, fontSize: 11 }}>
                {s.visibleHistory().length} reviews
              </Text>
            </View>
          </View>
          <VirtualList
            accessibilityLabel="studio.history.list"
            ref={(handle) => {
              history = handle;
              props.onListHandle?.("history", handle);
            }}
            data={s.visibleHistory()}
            itemKey={(entry) => entry.id}
            estimatedItemSize={140}
            initialNumToRender={5}
            overscan={2}
            style={{ height: height() - 300, minHeight: 0, flexDirection: "column" }}
            renderItem={(entry) => <HistoryRow entry={entry} />}
            emptyState={<Text style={{ color: p().muted, padding: 10 }}>No reviews match this search.</Text>}
          />
          <View style={{ padding: 6, gap: 6, flexShrink: 0 }}>
            <TextInput
              accessibilityLabel="studio.note.draft"
              multiline
              value={s.draft()}
              onChangeText={s.setDraft}
              style={{ ...inputStyle(), height: 70 }}
            />
            <Action
              label="studio.note.post"
              text="Post review"
              disabled={!s.draft().trim()}
              onPress={() => {
                s.post();
                if (history) command(history.scrollToIndex(0));
              }}
              palette={p()}
            />
          </View>
        </View>
        <View accessibilityLabel="studio.inspector" style={{ ...paneStyle(inspectorWidth()), overflow: "scroll" }}>
          <View style={{ padding: 10, gap: 8, flexShrink: 0 }}>
            <Text style={{ color: p().text, fontSize: 14 }}>Clip inspector</Text>
            <Text style={{ color: p().muted, fontSize: 11 }}>{s.selectedClipId()}</Text>
            <TextInput
              accessibilityLabel="studio.clip.title"
              value={s.title()}
              onChangeText={s.setTitle}
              style={inputStyle()}
            />
            <Text style={{ color: p().muted, fontSize: 11 }}>Start / duration (seconds)</Text>
            <TextInput
              accessibilityLabel="studio.clip.start"
              value={s.start()}
              onChangeText={s.setStart}
              style={inputStyle()}
            />
            <TextInput
              accessibilityLabel="studio.clip.duration"
              value={s.duration()}
              onChangeText={s.setDuration}
              style={inputStyle()}
            />
            <Action label="studio.clip.save" text="Save clip" onPress={s.apply} palette={p()} />
            {props.preview?.()}
            <Text style={{ color: p().text, fontSize: 14 }}>Selected review</Text>
            <Text selectable accessibilityLabel="studio.review.subject" style={{ color: p().text, fontSize: 12 }}>
              {s.selectedHistory().subject}
            </Text>
            <For each={s.selectedHistory().paragraphs}>
              {(paragraph, index) => (
                <Text
                  selectable
                  accessibilityLabel={`studio.review.text.${index()}`}
                  style={{ color: p().text, fontSize: 12, lineHeight: 18 }}
                >
                  {paragraph}
                </Text>
              )}
            </For>
            <Action
              label="studio.review.copy"
              text="Copy review"
              onPress={() => command(props.copyText(s.selectedHistory().paragraphs.join("\n")))}
              palette={p()}
            />
          </View>
        </View>
      </View>
      <View style={{ height: 42, padding: 8, flexShrink: 0, overflow: "hidden" }}>
        <Text accessibilityLabel="studio.status" accessibilityRole="status" style={{ color: p().muted, fontSize: 12 }}>
          {s.status()}
        </Text>
      </View>
    </View>
  );
}
