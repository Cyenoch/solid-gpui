import { View, type SolidChild } from "@solid-gpui/core";
import { ScrollShadow } from "@solid-gpui/core/components";
import { MarkdownText } from "./MarkdownText";
import { Copy, colors } from "./ui";
import type { MarkdownBlock } from "./markdown";
export function MarkdownTable(props: {
  block: Extract<MarkdownBlock, { kind: "table" }>;
  width: number;
  open: (href: string) => void;
  renderCell?: (cell: string, column: number, row: number) => SolidChild;
}) {
  const tableWidth = () => Math.max(props.width, props.block.header.length * 220);
  const columnWidth = () => tableWidth() / props.block.header.length;
  return (
    <ScrollShadow
      axis="horizontal"
      scrollbarVisibility="always"
      color={colors.bg}
      style={{ flexShrink: 0, width: props.width, minWidth: 0 }}
    >
      <View style={{ width: tableWidth(), flexShrink: 0, paddingBottom: tableWidth() > props.width ? 14 : 0 }}>
        <View style={{ flexDirection: "row", backgroundColor: colors.panel }}>
          {() =>
            props.block.header.map((cell, index) => (
              <View style={{ width: columnWidth(), flexShrink: 0, padding: 10 }}>
                <Copy size={13} style={{ fontWeight: "semibold", textAlign: props.block.align[index] }}>
                  {cell}
                </Copy>
              </View>
            ))
          }
        </View>
        {() =>
          props.block.rows.map((row, rowIndex) => (
            <View>
              <View style={{ height: 1, backgroundColor: colors.line }} />
              <View style={{ flexDirection: "row" }}>
                {row.map((cell, index) => (
                  <View
                    style={{
                      width: columnWidth(),
                      flexShrink: 0,
                      padding: 10,
                      gap: 8,
                      flexDirection: "column",
                    }}
                  >
                    <View style={{ width: columnWidth() - 20 }}>
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
    </ScrollShadow>
  );
}
