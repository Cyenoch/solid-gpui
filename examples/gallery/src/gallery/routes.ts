import type { RouteComponent } from "@solid-gpui/router";
import { createRootRoute, createRoute, createRouter } from "@solid-gpui/router";
import { App } from "./App";
import { BenchmarkMiniApp } from "./pages/BenchmarkMiniApp";
import { CursorsShowcase } from "./pages/CursorsShowcase";
import { DragDropShowcase } from "./pages/DragDropShowcase";
import { EventsShowcase } from "./pages/EventsShowcase";
import { FlexboxShowcase } from "./pages/FlexboxShowcase";
import { FormMiniApp } from "./pages/FormMiniApp";
import { ImageShowcase } from "./pages/ImageShowcase";
import {
  NativeControlsShowcase,
  NativeOverlaysShowcase,
  NativeSettingsShowcase,
  NativeDockShowcase,
  NativeChartsShowcase,
} from "./pages/NativeComponentsShowcase";
import { NativePlatformShowcase } from "./pages/NativePlatformShowcase";
import { OverviewPage } from "./pages/OverviewPage";
import { PaletteMiniApp } from "./pages/PaletteMiniApp";
import { PressableShowcase } from "./pages/PressableShowcase";
import { StylingShowcase } from "./pages/StylingShowcase";
import { TextInputShowcase } from "./pages/TextInputShowcase";
import { TextShowcase } from "./pages/TextShowcase";
import { TodoMiniApp } from "./pages/TodoMiniApp";
import { TransitionsShowcase } from "./pages/TransitionsShowcase";
import { ViewShowcase } from "./pages/ViewShowcase";
import { VirtualListShowcase } from "./pages/VirtualListShowcase";
import { PAGES, pagePath, type PageId } from "./types";

const PAGE_COMPONENTS: Record<PageId, RouteComponent> = {
  overview: OverviewPage as RouteComponent,
  view: ViewShowcase as RouteComponent,
  text: TextShowcase as RouteComponent,
  pressable: PressableShowcase as RouteComponent,
  "text-input": TextInputShowcase as RouteComponent,
  image: ImageShowcase as RouteComponent,
  "virtual-list": VirtualListShowcase as RouteComponent,
  flexbox: FlexboxShowcase as RouteComponent,
  styling: StylingShowcase as RouteComponent,
  events: EventsShowcase as RouteComponent,
  "drag-drop": DragDropShowcase as RouteComponent,
  transitions: TransitionsShowcase as RouteComponent,
  cursors: CursorsShowcase as RouteComponent,
  "native-platform": NativePlatformShowcase as RouteComponent,
  "native-controls": NativeControlsShowcase as RouteComponent,
  "native-overlays": NativeOverlaysShowcase as RouteComponent,
  "native-settings": NativeSettingsShowcase as RouteComponent,
  "native-dock": NativeDockShowcase as RouteComponent,
  "native-charts": NativeChartsShowcase as RouteComponent,

  "todo-app": TodoMiniApp as RouteComponent,
  "form-builder": FormMiniApp as RouteComponent,
  benchmark: BenchmarkMiniApp as RouteComponent,
  palette: PaletteMiniApp as RouteComponent,
};

const rootRoute = createRootRoute({
  component: App as RouteComponent,
});

const routeTree = rootRoute.addChildren(
  PAGES.map((page) =>
    createRoute({
      getParentRoute: () => rootRoute,
      path: page.id === "overview" ? "/" : page.id,
      component: PAGE_COMPONENTS[page.id],
    }),
  ),
);

export function createGalleryRouter(initialEntries: readonly string[] = [pagePath("overview")]) {
  return createRouter({
    routeTree,
    initialEntries,
  });
}
