import { createEffect, getOwner, on, onCleanup, runWithOwner } from "solid-js";
import { createHostElement, mountPopupSurface, type SolidElement } from "./renderer";
import { resolveTree } from "./renderer/host-config";

const placements = [
  "bottom-start",
  "bottom",
  "bottom-end",
  "top-start",
  "top",
  "top-end",
  "left",
  "left-start",
  "left-end",
  "right",
  "right-start",
  "right-end",
] as const;
export type SystemPopoverPlacement = (typeof placements)[number];

export interface SystemPopoverProps {
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
  readonly width: number;
  readonly height: number;
  readonly placement?: SystemPopoverPlacement;
  readonly gap?: number;
  readonly accessibilityLabel?: string;
  readonly slots: { readonly trigger: SolidElement };
  /** Construct content in the popup Surface. Keep persistent application state above this factory. */
  readonly content: () => SolidElement;
  readonly onError?: (error: unknown) => void;
}

/** An owner-attached native popup. Unsupported hosts reject creation explicitly. */
export function SystemPopover(props: SystemPopoverProps): SolidElement {
  const owner = getOwner();
  if (!owner) throw new Error("SystemPopover must be created inside a Solid owner");
  const tree = resolveTree();
  const anchor = createHostElement("Pressable", {
    get children() {
      return props.slots.trigger;
    },
    onPress: () => props.onOpenChange(!props.open),
    focusable: true,
    accessibilityRole: "button",
    get accessibilityLabel() {
      return props.accessibilityLabel;
    },
    get accessibilityExpanded() {
      return props.open;
    },
    style: { alignSelf: "flex-start" },
  });
  const fail = (error: unknown) => {
    props.onOpenChange(false);
    if (props.onError) props.onError(error);
    else
      runWithOwner(owner, () => {
        throw error;
      });
  };
  createEffect(
    on(
      () => props.open,
      (open) => {
        if (!open) return;
        const placement = placements.indexOf(props.placement ?? "bottom-start");
        const dispose = mountPopupSurface(
          tree,
          {
            type: "open-popup",
            anchorNodeId: anchor.id,
            width: props.width,
            height: props.height,
            placement,
            gap: props.gap ?? 8,
          },
          owner,
          () => props.content(),
          () => props.onOpenChange(false),
          fail,
        );
        onCleanup(dispose);
      },
    ),
  );
  return anchor;
}
