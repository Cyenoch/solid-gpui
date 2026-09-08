import { createFileRoute } from "@solid-gpui/router";
import { Components } from "../Components";
import { usePage } from "../route-page";

export const Route = createFileRoute("/components/$")({
  component: () => <Components {...usePage()} />,
});
