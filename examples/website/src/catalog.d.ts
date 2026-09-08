declare module "virtual:component-catalog" {
  const catalog: ReturnType<typeof import("../component-catalog").componentCatalog>;
  export default catalog;
  export const highlights: readonly import("@solid-gpui/shiki").HighlightResult[];
}

declare module "virtual:component-previews" {
  const previews: Record<string, (() => import("@solid-gpui/core").SolidChild) | undefined>;
  export default previews;
}
