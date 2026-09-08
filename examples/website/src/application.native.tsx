import { createEffect } from "@solid-gpui/core/runtime";
import { setSiteError } from "./site";
import { mountApplication, type DisposableTransport, type Root } from "@solid-gpui/core";
import { useNative } from "@solid-gpui/core/components";
import { RouterProvider } from "@solid-gpui/router";
import { createWebsiteRouter } from "./router";
import { configureSite, size, setSize, type SiteState } from "./site";
import { locale, setLocale } from "./i18n";
function DesktopApp(props: { router: ReturnType<typeof createWebsiteRouter> }) {
  const native = useNative();
  createEffect(() => {
    void native.setTheme("dark").catch((error) => setSiteError(String(error)));
  });
  return <RouterProvider router={props.router} />;
}
export function mountWebsite(transport: () => DisposableTransport, hotKey?: string) {
  return mountApplication<SiteState>({
    hotKey,
    transport,
    setup(previous) {
      let root: Root;
      const router = createWebsiteRouter(previous?.route ?? "/showcase/workspace");
      setSize(previous ? { width: previous.width, height: previous.height } : { width: 1280, height: 800 });
      setLocale(previous?.language ?? "en");
      configureSite({
        copyText: (text) => root.setClipboardText(text),
        openUrl: (url) => root.openUrl(url),
        saveLanguage: () => {},
      });
      return {
        render: () => <DesktopApp router={router} />,
        rootOptions: { onWindowResize: (width, height) => setSize({ width, height }) },
        onMount(value) {
          root = value;
          if (!previous) void root.resize(1280, 860).catch((error) => setSiteError(String(error)));
          void root.setTitle("Solid GPUI").catch((error) => setSiteError(String(error)));
          void root
            .getWindowSize()
            .then(([width, height]) => setSize({ width, height }))
            .catch((error) => setSiteError(String(error)));
        },
        captureState: () => ({ route: router.state.location.href, ...size(), language: locale() }),
      };
    },
  });
}
