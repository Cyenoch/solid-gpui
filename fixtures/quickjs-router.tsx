import { MemoryTransport, Text, createRoot, mountApplication } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import {
  Outlet,
  RouterProvider,
  createRootRoute,
  createRoute,
  createRouter,
  isRedirect,
  redirect,
} from "@solid-gpui/router";
import { createGalleryState } from "../examples/gallery/src/gallery/context";
import { createGalleryRouter } from "../examples/gallery/src/gallery/routes";
import { PAGES, pagePath } from "../examples/gallery/src/gallery/types";

function assert(condition: unknown, message: string): void {
  if (!condition) throw new Error(message);
}

assert(self === globalThis && typeof document === "undefined", "worker-style global without a DOM");

const query = new URLSearchParams("q=%E4%B8%AD%E6%96%87+%F0%9F%8C%8D&tag=a&tag=b");
assert(query.get("q") === "中文 🌍" && query.getAll("tag").join() === "a,b", "Unicode search parsing");
assert(new URL("../search", "native://solid-gpui/nested/page").href === "native://solid-gpui/search", "relative URL");

const controller = new AbortController();
let abortEvents = 0;
controller.signal.addEventListener("abort", () => abortEvents++, { once: true });
controller.signal.onabort = () => {
  throw new Error("removed abort callback ran");
};
controller.signal.onabort = null;
controller.abort(null);
controller.abort("ignored second reason");
assert(controller.signal.aborted && controller.signal.reason === null && abortEvents === 1, "abort identity and once");
assert(controller.signal.onabort === null, "cleared abort handler");

const redirected = redirect({ to: "/search", search: { q: "redirected" } });
assert(isRedirect(redirected) && redirected.status === 307, "redirect Response identity");
assert(!isRedirect(new Error("loader failed")), "ordinary errors are not redirects");
const headers = new Headers([
  ["Location", " /search "],
  ["__proto__", "safe"],
]);
assert(headers.get("location") === "/search" && headers.get("__proto__") === "safe", "header normalization");
let invalidNameRejected = false;
try {
  headers.append("K", "not an ASCII token");
} catch (error) {
  invalidNameRejected = error instanceof TypeError;
}
assert(invalidNameRejected && !headers.has("k"), "Unicode header names cannot normalize into ASCII tokens");
for (let attempt = 0; attempt < 2; attempt++) {
  let rejected = false;
  try {
    headers.append("x-invalid", "one\ntwo");
  } catch (error) {
    rejected = error instanceof TypeError;
  }
  assert(rejected, "invalid header rejection must not depend on previous validation");
}
let invalidSetRejected = false;
try {
  headers.set("location", "one\ntwo");
} catch (error) {
  invalidSetRejected = error instanceof TypeError;
}
assert(invalidSetRejected && headers.get("location") === "/search", "invalid set preserves existing header");
let bodyRejected = false;
try {
  new Response("unavailable body");
} catch (error) {
  bodyRejected = error instanceof TypeError;
}
assert(bodyRejected, "body-bearing Responses must be rejected");
for (const init of [{ status: 199 }, { status: 600 }, { statusText: "bad\r\nphrase" }]) {
  let rejected = false;
  try {
    new Response(null, init);
  } catch {
    rejected = true;
  }
  assert(rejected, "invalid Response metadata");
}
assert(typeof fetch === "undefined", "QuickJS must not advertise an unavailable network service");

const transport = new MemoryTransport();
const root = createRoot(transport);
let loaderStarted!: () => void;
const started = new Promise<void>((resolve) => {
  loaderStarted = resolve;
});
let loaderAborted = false;
let renderedError = "";
const layout = createRootRoute({ component: Outlet });
const home = createRoute({ getParentRoute: () => layout, path: "/", component: () => <Text>Home</Text> });
const search = createRoute({
  getParentRoute: () => layout,
  path: "search",
  validateSearch: (value: Record<string, unknown>) => ({ q: String(value.q ?? "") }),
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => deps.q,
  component: () => <Text>Search</Text>,
});
const slow = createRoute({
  getParentRoute: () => layout,
  path: "slow",
  loader: ({ abortController }) =>
    new Promise<string>((_resolve, reject) => {
      abortController.signal.addEventListener(
        "abort",
        () => {
          loaderAborted = true;
          reject(abortController.signal.reason);
        },
        { once: true },
      );
      loaderStarted();
    }),
});
const redirectRoute = createRoute({
  getParentRoute: () => layout,
  path: "redirect",
  beforeLoad: () => {
    throw redirected;
  },
});
const broken = createRoute({
  getParentRoute: () => layout,
  path: "broken",
  loader: () => {
    throw new Error("expected loader failure");
  },
  errorComponent: ({ error }) => {
    renderedError = error instanceof Error ? error.message : String(error);
    return <Text>{renderedError}</Text>;
  },
});
const router = createRouter({ routeTree: layout.addChildren([home, search, slow, redirectRoute, broken]) });
try {
  root.render(() => <RouterProvider router={router} />);
  await router.load();
  await router.navigate({ to: "/search", search: { q: "中文 🌍" } });
  assert(router.latestLocation.search.q === "中文 🌍", "search navigation");
  assert(router.stores.getMatchStore("/search").get()?.loaderData === "中文 🌍", "search loader data");

  const loading = router.navigate({ to: "/slow" });
  await started;
  await router.navigate({ to: "/search", search: { q: "after cancellation" } });
  await loading;
  assert(loaderAborted && router.latestLocation.pathname === "/search", "superseded loader cancellation");
  assert(
    router.stores.getMatchStore("/search").get()?.loaderData === "after cancellation",
    "cancelled loader isolation",
  );

  await router.navigate({ to: "/redirect" });
  assert(
    router.latestLocation.pathname === "/search" && router.latestLocation.search.q === "redirected",
    "internal redirect",
  );
  await router.navigate({ to: "/broken" });
  await Promise.resolve();
  assert(renderedError === "expected loader failure", "loader error rendering");
} finally {
  root.unmount();
}

const galleryTransport = new MemoryTransport();
const galleryRoot = createRoot(galleryTransport);
const gallery = createGalleryState("dark");
gallery.setRoot(galleryRoot);
const galleryRouter = createGalleryRouter();
try {
  galleryRoot.render(() => <RouterProvider router={galleryRouter} />);
  await galleryRouter.load();
  for (const page of PAGES) {
    const path = pagePath(page.id);
    await galleryRouter.navigate({ to: path });
    await Promise.resolve();
    assert(galleryRouter.latestLocation.pathname === path, `Gallery navigation: ${path}`);
    galleryTransport.submitted.length = 0;
  }
} finally {
  galleryRoot.unmount();
}

// The real VM test receives this snapshot only after all asynchronous assertions.
mountApplication({
  transport: () => new EmbeddedTransport(),
  setup: () => ({ render: () => <Text>Router: passed</Text> }),
});
