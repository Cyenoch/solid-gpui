import { createFileRoute } from "@solid-gpui/router";
import { Showcase } from "../Showcase";
import { usePage } from "../route-page";

export const Route = createFileRoute("/showcase/$")({
  component: () => <Showcase {...usePage()} />,
});
