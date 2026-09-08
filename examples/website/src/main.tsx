import { configureSite, setSize } from "./site";
import { setLocale } from "./i18n";
import { createRoot } from "@solid-gpui/core";
import { WebTransport } from "@solid-gpui/core/web";
import { RouterProvider } from "@solid-gpui/router";
import { createWebsiteRouter } from "./router";
import { loadHost } from "./wasm";
import { showStartupError as showError } from "./startup";
try {
  const router = createWebsiteRouter(location.hash.slice(1) || "/");
  const updateRoute = () => {
    void router.navigate({ to: location.hash.slice(1) || "/", replace: true }).catch(showError);
  };
  const unsubscribeHistory = router.history.subscribe(({ location: next }) => {
    if (location.hash.slice(1) !== next.href) history.pushState(null, "", `#${next.href}`);
  });
  const updateSize = () => setSize({ width: innerWidth, height: innerHeight });
  configureSite({
    copyText: (text) => navigator.clipboard.writeText(text),
    openUrl: async (url) => {
      window.open(url, "_blank", "noopener");
    },
    saveLanguage: (language) => {
      localStorage.setItem("solid-gpui-language", language);
      document.documentElement.lang = language;
    },
  });
  const language = localStorage.getItem("solid-gpui-language") === "zh-CN" ? "zh-CN" : "en";
  setLocale(language);
  document.documentElement.lang = language;
  updateSize();
  window.addEventListener("hashchange", updateRoute);
  window.addEventListener("resize", updateSize);
  const transport = new WebTransport(await loadHost());
  let failed = false;
  transport.onTermination((error) => {
    if (error.cause?.kind !== "shutdown") {
      failed = true;
      showError(error);
    }
  });
  const root = createRoot(transport);
  await router.load();
  root.render(() => <RouterProvider router={router} />);
  if (!failed) document.getElementById("status")?.remove();
  window.addEventListener(
    "pagehide",
    () => {
      unsubscribeHistory();
      window.removeEventListener("hashchange", updateRoute);
      window.removeEventListener("resize", updateSize);
      root.unmount();
      transport.dispose();
    },
    { once: true },
  );
} catch (error) {
  showError(error);
}
