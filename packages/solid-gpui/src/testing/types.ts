import type { HostKind } from "../renderer/types";

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
