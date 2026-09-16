import type { SolidChild } from "./renderer/types";

export namespace JSX {
  /**
   * A native JSX element is exactly a renderer child: host nodes, nested
   * children, raw text values, and the accessors Solid inserts.
   *
   * Solid types its control flow against its own DOM-shaped JSX namespace, so
   * `solid-js`'s `For`/`Show` cannot describe or return native children. The
   * native-typed control flow in `./runtime` states those signatures against
   * this element type instead.
   */
  export type Element = SolidChild;

  export interface ElementChildrenAttribute {
    children: unknown;
  }
}
