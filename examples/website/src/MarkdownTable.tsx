import { View, type SolidChild } from "@solid-gpui/core";
import { MarkdownText } from "./MarkdownText";
import { Copy, colors } from "./ui";
import type { MarkdownBlock } from "./markdown";
export function MarkdownTable(props: {
  block: Extract<MarkdownBlock, { kind: "table" }>;
  width: number;
  open: (href: string) => void;
  renderCell?: (cell: string, column: number, row: number) => SolidChild;
}) {
  const columnWidth = (index: number) =>
    props.block.header.length === 2
      ? props.width * (index === 0 ? 0.36 : 0.64)
      : props.block.header.length === 3
        ? props.width * [0.3, 0.55, 0.15][index]
        : props.width / props.block.header.length;
  const stacked = () => props.width < 480 && props.block.header.length > 2;
  return (
    <View style={{ flexShrink: 0, width: props.width }}>
      {() =>
        stacked() ? null : (
          <View style={{ flexDirection: "row", backgroundColor: colors.panel }}>
            {() =>
              props.block.header.map((cell, index) => (
                <View style={{ width: columnWidth(index), padding: 10 }}>
                  <Copy size={13} style={{ fontWeight: "semibold", textAlign: props.block.align[index] }}>
                    {cell}
                  </Copy>
                </View>
              ))
            }
          </View>
        )
      }
      {() =>
        props.block.rows.map((row, rowIndex) => (
          <View>
            <View style={{ height: 1, backgroundColor: colors.line }} />
            <View style={{ flexDirection: stacked() ? "column" : "row", gap: stacked() ? 4 : 0 }}>
              {row.map((cell, index) => (
                <View
                  style={{
                    width: stacked() ? props.width : columnWidth(index),
                    padding: 10,
                    gap: 8,
                    flexDirection: stacked() ? "row" : "column",
                  }}
                >
                  {() =>
                    stacked() ? (
                      <Copy size={11} color={colors.muted} style={{ width: 60 }}>
                        {props.block.header[index]}
                      </Copy>
                    ) : null
                  }
                  <View style={{ width: stacked() ? props.width - 88 : columnWidth(index) - 20 }}>
                    {() =>
                      props.renderCell ? (
                        props.renderCell(cell, index, rowIndex)
                      ) : (
                        <MarkdownText source={cell} open={props.open} size={13} align={props.block.align[index]} />
                      )
                    }
                  </View>
                </View>
              ))}
            </View>
          </View>
        ))
      }
      <View style={{ height: 1, backgroundColor: colors.line }} />
    </View>
  );
}
