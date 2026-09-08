import { CodeCollapse } from "./CodeCollapse";
import { View, Icon, type SolidChild } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { Button, Copy, colors } from "./ui";
import { CodeBlock } from "./CodeBlock";
import { CodeExcerpt } from "./CodeExcerpt";

export function ComponentPreview(props: {
  source: string;
  preview?: () => SolidChild;
  width: number;
  compact?: boolean;
  unavailable?: string;
  previewWidth?: number;
  minPreviewWidth?: number;
  centered?: boolean;
}) {
  const previewWidth = () => Math.min(props.previewWidth ?? 360, props.width - 58);
  const [expanded, setExpanded] = createSignal(false);
  return (
    <View style={{ borderRadius: 14, borderWidth: 1, borderColor: colors.line, overflow: "hidden" }}>
      <View
        style={{
          minHeight: props.preview ? (props.compact ? 180 : 230) : 80,
          padding: props.width < 440 ? 20 : 28,
          gap: 12,
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        {() => {
          const Preview = props.preview;
          return Preview ? (
            <View
              style={{
                width: previewWidth(),
                overflow: props.minPreviewWidth ? "scroll" : "visible",
                gap: 12,
                alignItems: props.centered ? "center" : "stretch",
              }}
            >
              <View
                style={{
                  width: Math.max(previewWidth(), props.minPreviewWidth ?? 0),
                  alignItems: props.centered ? "center" : "stretch",
                  gap: 12,
                }}
              >
                {Preview()}
              </View>
            </View>
          ) : (
            <Copy size={13} color={colors.muted}>
              {props.unavailable ?? "Interactive preview is not available for this example yet."}
            </Copy>
          );
        }}
      </View>
      {() =>
        (props.minPreviewWidth ?? 0) > previewWidth() ? (
          <View style={{ flexDirection: "row", gap: 8, padding: 12, alignItems: "center", justifyContent: "center" }}>
            <Icon name="lucide:chevron-right" size={14} color={colors.muted} />
            <Copy size={12} color={colors.muted}>
              Scroll horizontally to explore
            </Copy>
          </View>
        ) : null
      }
      <View style={{ height: 1, backgroundColor: colors.line }} />
      {() =>
        expanded() ? (
          <CodeBlock source={props.source} label="Example.tsx" />
        ) : (
          <CodeExcerpt source={props.source} width={props.width} onExpand={() => setExpanded(true)} />
        )
      }
      {() => (expanded() ? <CodeCollapse onPress={() => setExpanded(false)} /> : null)}
    </View>
  );
}
