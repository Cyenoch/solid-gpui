import type { AccessibilityProps, HostKind } from "../renderer/types";
import type { Style } from "../style";

/** A detached view of one committed node, in native child order. */
export interface TestNode {
  readonly id: number;
  readonly kind: HostKind;
  readonly parentId: number;
  readonly children: readonly number[];
  readonly text: string | null;
  readonly inputValue: string | null;
  readonly placeholder: string | null;
  readonly accessibilityLabel: string | null;
  readonly tooltip: string | null;
  /** Submitted values, not measured GPUI geometry. Colors use #rrggbbaa. */
  readonly style: Readonly<Style> | null;
  readonly accessibility: Readonly<AccessibilityProps>;
  readonly disabled: boolean;
  readonly virtualList: Readonly<{
    itemCount: number;
    rangeStart: number;
    rangeEnd: number;
    estimatedItemSize: number;
    overscan: number;
    dataRevision: number;
  }> | null;
}

/** The latest committed tree for a Surface; previous views are not mutated. */
export interface TestSurface {
  readonly surfaceId: number;
  readonly epoch: number;
  readonly revision: number;
  /** Preorder traversal, including the synthetic root. */
  readonly nodes: readonly TestNode[];
}

/** Commit history without exposing wire tags, masks or generated message types. */
export interface TestCommit {
  readonly type: "snapshot" | "patch";
  readonly surfaceId: number;
  readonly epoch: number;
  readonly revision: number;
}

export type TestEvent =
  | { readonly type: "visible-range"; readonly start: number; readonly end: number }
  | { readonly type: "layout"; readonly x: number; readonly y: number; readonly width: number; readonly height: number }
  | {
      readonly type: "pointer";
      readonly action: "down" | "up";
      readonly button?: "left" | "right" | "middle" | "back" | "forward";
      readonly x: number;
      readonly y: number;
      readonly clickCount?: number;
      readonly modifiers?: readonly string[];
    }
  | { readonly type: "pointer-move"; readonly x: number; readonly y: number; readonly modifiers?: readonly string[] }
  | { readonly type: "press" | "focus" | "blur" }
  | {
      readonly type: "input";
      readonly text: string;
      /** UTF-8 byte offsets; omitted selection places the caret at the end. */
      readonly selectionStart?: number;
      readonly selectionEnd?: number;
    }
  | { readonly type: "native"; readonly eventId: number; readonly value: unknown };

/** An InvokeNative request, for either a module function or a component method. */
export interface TestNativeCall {
  readonly surfaceId: number;
  readonly epoch: number;
  readonly nodeId: number;
  readonly requestId: number;
  readonly moduleId: readonly number[];
  readonly moduleDigest: readonly number[];
  readonly functionId: number;
  /** Opaque bytes, matching Root.invokeNative; use decodeJson for generated DTO calls. */
  readonly args: Uint8Array;
}

interface TestRequest {
  readonly surfaceId: number;
  readonly epoch: number;
  readonly nodeId: number;
  readonly requestId: number;
}

/** Core scrolling requests remain pending until explicitly replied to or rejected. */
export type TestScrollCommand = TestRequest &
  (
    | { readonly type: "get-scroll-offset" }
    | { readonly type: "scroll-to-end" }
    | { readonly type: "scroll-to-index"; readonly index: number }
    | { readonly type: "scroll-to-offset"; readonly offset: number }
  );
