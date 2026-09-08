import { MemoryTransport, createRoot } from "@solid-gpui/core";
import { RouterProvider } from "@solid-gpui/router";
import { createWebsiteRouter } from "../src/router";
import { configureSite, setSize } from "../src/site";
import { Envelope } from "../../../packages/solid-gpui/src/protocol/generated/protocol";
import catalog from "virtual:component-catalog";
import { referenceDocs } from "../src/documentation";

export async function verifyRuntime() {
  configureSite({ copyText: async () => {}, openUrl: async () => {}, saveLanguage: () => {} });
  setSize({ width: 1280, height: 800 });
  const router = createWebsiteRouter("/components/button");
  await router.load();
  const transport = new MemoryTransport();
  const root = createRoot(transport);
  try {
    root.render(() => <RouterProvider router={router} />);
    const first = Envelope.decode(transport.submitted[0]!.subarray(4)).body!;
    if (first.tag !== 1) throw new Error("Website did not publish a snapshot");
    const sidebar = first.value.nodes!.find((node) => node.kind === 6)!;
    if (!sidebar) throw new Error("Component sidebar was not mounted");
    transport.submitted.length = 0;
    for (const { name, id } of catalog) {
      await router.navigate({ to: `/components/${id}` });
      await Promise.resolve();
      for (const frame of transport.submitted.splice(0)) {
        const body = Envelope.decode(frame.subarray(4)).body!;
        if (body.tag === 1) throw new Error("Navigation rebuilt the whole surface");
        if (body.tag !== 3) continue;
        for (const { operation } of body.value.operations ?? []) {
          if (operation?.tag === 4 && operation.value.id === sidebar.id)
            throw new Error(`Sidebar remounted on ${name}`);
        }
      }
    }
    await router.navigate({ to: "/showcase/workspace" });
    if (!transport.submitted.length) throw new Error("Showcase navigation did not publish content");
    const showcaseSidebar = transport.submitted.flatMap((frame) => {
      const body = Envelope.decode(frame.subarray(4)).body;
      return body?.tag === 3
        ? (body.value.operations ?? []).flatMap(({ operation }) =>
            operation?.tag === 1 && operation.value.node?.kind === 6 ? [operation.value.node] : [],
          )
        : [];
    })[0];
    if (!showcaseSidebar) throw new Error("Showcase sidebar was not mounted");
    for (const id of ["account", "collections", "workspace"]) {
      transport.submitted.length = 0;
      await router.navigate({ to: `/showcase/${id}` });
      if (!transport.submitted.length) throw new Error(`Showcase did not update: ${id}`);
      if (transport.submitted.some((frame) => Envelope.decode(frame.subarray(4)).body?.tag === 1))
        throw new Error("Showcase rebuilt the whole surface");
      for (const frame of transport.submitted) {
        const body = Envelope.decode(frame.subarray(4)).body;
        if (
          body?.tag === 3 &&
          body.value.operations?.some(
            ({ operation }) => operation?.tag === 4 && operation.value.id === showcaseSidebar.id,
          )
        )
          throw new Error(`Showcase sidebar remounted on ${id}`);
      }
    }
    const routerGuide = referenceDocs.find((doc) => doc.id === "router");
    if (
      !routerGuide?.source.includes("@solid-gpui/router/vite") ||
      !routerGuide.source.includes("https://tanstack.com/router/")
    )
      throw new Error("Router guide is missing setup or upstream documentation");
    transport.submitted.length = 0;
    await router.navigate({ to: "/docs/reference/router" });
    if (!transport.submitted.length || router.state.matches.at(-1)?.status !== "success")
      throw new Error("Router guide did not render");
  } finally {
    root.unmount();
  }
}
