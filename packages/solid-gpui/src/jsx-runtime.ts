import type { HostNodeInternal } from "./renderer/types";

export namespace JSX {
  export type Element =
    HostNodeInternal | (() => Element) | string | number | boolean | null | undefined | readonly Element[];

  export interface ElementChildrenAttribute {
    children: unknown;
  }
}
