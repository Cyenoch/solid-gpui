import { createFileRoute } from "@solid-gpui/router";
import { Landing } from "../Landing";
import { usePage } from "../route-page";

export const Route = createFileRoute("/")({
  component: () => <Landing {...usePage()} />,
});
