import { useLocation, useNavigate } from "@solid-gpui/router";
import { docsQuery, setDocsQuery, size, headerHeight, setSiteError } from "./site";

export function usePage() {
  const route = useLocation({ select: (location) => location.pathname });
  const go = useNavigate();
  return {
    get route() {
      return route();
    },
    get width() {
      return size().width;
    },
    get height() {
      return size().height - headerHeight();
    },
    get query() {
      return docsQuery();
    },
    setQuery: setDocsQuery,
    navigate: (to: string) => {
      void go({ to }).catch((error) => setSiteError(String(error)));
    },
  };
}
