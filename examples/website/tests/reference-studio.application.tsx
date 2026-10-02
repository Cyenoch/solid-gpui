import {
  createRoot,
  createWindowSizeStore,
  useWindowSize,
  type Root,
  type Transport,
  type VirtualListHandle,
} from "@solid-gpui/core";
import { createRootRoute, createRoute, createRouter, RouterProvider, useLocation, useRouter } from "@solid-gpui/router";
import { ReferenceStudio, createReferenceStudioState } from "../src/showcase/ReferenceStudio";
import { StudioPreview } from "../src/showcase/StudioPreview.native";
import type { LiveFrameRef } from "@solid-gpui/core/components";

/** Production controls and router, isolated from documentation virtualization for native qualification. */
export function mountReferenceStudio(transport: Transport, surfaceId = 1) {
  const size = createWindowSizeStore({ width: 1280, height: 720 });
  const live = new Set<object>();
  let peak = 0;
  let root: Root;
  let previewFrame: LiveFrameRef | undefined;
  const lists = new Map<"track" | "history", VirtualListHandle>();
  const route = createRootRoute({
    component: () => {
      const dimensions = useWindowSize(size);
      const location = useLocation();
      const router = useRouter();
      const state = createReferenceStudioState();
      return (
        <ReferenceStudio
          state={state}
          width={dimensions().width}
          height={dimensions().height}
          view={location().pathname === "/studio/history" ? "history" : "timeline"}
          navigate={(view) => void router.navigate({ to: `/studio/${view}` })}
          copyText={(text) => root.setClipboardText(text)}
          preview={() => (
            <StudioPreview
              title={state.selectedClip().label}
              onFrame={(frame) => {
                previewFrame = frame;
              }}
            />
          )}
          onListHandle={(kind, handle) => {
            if (handle) lists.set(kind, handle);
            else lists.delete(kind);
          }}
          onRowLifetime={(_kind, _id, mounted, owner) => {
            if (mounted) live.add(owner);
            else live.delete(owner);
            peak = Math.max(peak, live.size);
          }}
        />
      );
    },
  });
  const timeline = createRoute({ getParentRoute: () => route, path: "/studio/timeline", component: () => null });
  const history = createRoute({ getParentRoute: () => route, path: "/studio/history", component: () => null });
  const router = createRouter({
    routeTree: route.addChildren([timeline, history]),
    initialEntries: ["/studio/timeline"],
  });
  root = createRoot(transport, { surfaceId, onWindowResize: (width, height, scale) => size.set(width, height, scale) });
  root.render(() => <RouterProvider router={router} />);
  return {
    root,
    router,
    ready: router.load(),
    rowOwners: () => ({ live: live.size, peak }),
    preview: () => {
      if (!previewFrame) throw new Error("Studio preview is not mounted");
      return previewFrame;
    },
    list: (kind: "track" | "history") => {
      const handle = lists.get(kind);
      if (!handle) throw new Error(`Studio ${kind} list is not mounted`);
      return handle;
    },
    dispose: () => {
      root.unmount();
      router.history.destroy();
    },
  };
}
