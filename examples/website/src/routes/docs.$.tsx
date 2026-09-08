import { createFileRoute } from "@solid-gpui/router";
import { Docs } from "../Docs";
import { usePage } from "../route-page";

export const Route = createFileRoute("/docs/$")({
  component: () => <Docs {...usePage()} />,
});
