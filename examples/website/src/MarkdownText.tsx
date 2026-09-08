import { Text } from "@solid-gpui/core";
import { createMemo } from "@solid-gpui/core/runtime";
import { colors } from "./ui";
type Part = { text: string; kind: "plain" | "code" | "bold" | "link"; href?: string };
function parts(source: string): Part[] {
  const result: Part[] = [];
  let offset = 0;
  for (const match of source.matchAll(/`([^`]+)`|\*\*([^*]+)\*\*|\[([^\]]+)\]\(([^)]+)\)/g)) {
    if (match.index! > offset) result.push({ kind: "plain", text: source.slice(offset, match.index) });
    result.push(
      match[1]
        ? { kind: "code", text: match[1] }
        : match[2]
          ? { kind: "bold", text: match[2] }
          : { kind: "link", text: match[3], href: match[4] },
    );
    offset = match.index! + match[0].length;
  }
  if (offset < source.length) result.push({ kind: "plain", text: source.slice(offset) });
  return result;
}
export function MarkdownText(props: {
  source: string;
  open: (href: string) => void;
  size?: number;
  align?: "left" | "center" | "right";
}) {
  const content = createMemo(() => parts(props.source));
  return (
    <Text
      style={{
        fontFamily: "Inter Variable",
        fontSize: props.size ?? 15,
        lineHeight: props.size ? 21 : 25,
        textAlign: props.align,
        color: colors.muted,
        flexShrink: 0,
      }}
    >
      {() =>
        content().map((part) => (
          <Text
            onPress={part.href ? () => props.open(part.href!) : undefined}
            style={{
              fontFamily: part.kind === "code" ? "Maple Mono" : "Inter Variable",
              fontWeight: part.kind === "bold" ? "semibold" : "normal",
              color: part.kind === "plain" ? colors.muted : colors.text,
              textDecoration: part.kind === "link" ? "underline" : "none",
            }}
          >
            {part.text}
          </Text>
        ))
      }
    </Text>
  );
}
