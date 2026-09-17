import { Text, View } from "@solid-gpui/core";
import { createMemo } from "@solid-gpui/core/runtime";
import { HighlightedCode, type HighlightRun } from "@solid-gpui/shiki";
import { highlight } from "./highlight";

export function CodeLines(props: { source: string; language?: string; faded?: boolean }) {
  const highlighted = createMemo(() => ({
    ...highlight(props.source, props.language ?? "tsx"),
    background: "#161616",
  }));
  const lines = createMemo(() => {
    const result: HighlightRun[][] = [[]];
    for (const run of highlighted().runs) {
      run.text.split(/\r\n|\n|\r/).forEach((text, index) => {
        if (index) result.push([]);
        if (text) result[result.length - 1].push({ ...run, text });
      });
    }
    if (result.length > 1 && !result[result.length - 1].length) result.pop();
    return result;
  });
  const codeWidth = createMemo(
    () => Math.max(...lines().map((line) => line.reduce((width, run) => width + [...run.text].length, 0))) * 8 + 12,
  );
  return (
    <View style={{ minWidth: 0, overflow: props.faded ? "hidden" : "scroll" }}>
      {() =>
        props.faded ? (
          lines().map((line, index) => (
            <View
              style={{
                flexDirection: "row",
                width: codeWidth() + 44,
                height: 26,
                flexShrink: 0,
                opacity: [0.85, 0.5, 0.15][index] ?? 0,
              }}
            >
              <Text
                style={{
                  width: 28,
                  marginRight: 16,
                  flexShrink: 0,
                  fontFamily: "Maple Mono",
                  fontSize: 13,
                  lineHeight: 26,
                  textAlign: "right",
                  color: "#737373",
                }}
              >
                {index + 1}
              </Text>
              <Text
                style={{
                  width: codeWidth(),
                  flexShrink: 0,
                  fontFamily: "Maple Mono",
                  fontSize: 13,
                  lineHeight: 26,
                  lineClamp: 1,
                  textOverflow: "clip",
                }}
              >
                {line.map((run) => (
                  <Text
                    style={{
                      color: run.color,
                      fontStyle: run.fontStyle & 1 ? "italic" : "normal",
                      fontWeight: run.fontStyle & 2 ? "bold" : "normal",
                    }}
                  >
                    {run.text}
                  </Text>
                ))}
              </Text>
            </View>
          ))
        ) : (
          <View style={{ flexDirection: "row", width: codeWidth() + 44, flexShrink: 0 }}>
            <Text
              style={{
                width: 28,
                marginRight: 16,
                flexShrink: 0,
                fontFamily: "Maple Mono",
                fontSize: 13,
                lineHeight: 26,
                textAlign: "right",
                color: "#737373",
              }}
            >
              {lines()
                .map((_, index) => index + 1)
                .join("\n")}
            </Text>
            <HighlightedCode
              highlighted={highlighted()}
              style={{
                width: codeWidth(),
                flexShrink: 0,
                padding: 0,
                fontFamily: "Maple Mono",
                fontSize: 13,
                lineHeight: 26,
                textOverflow: "clip",
              }}
            />
          </View>
        )
      }
    </View>
  );
}
