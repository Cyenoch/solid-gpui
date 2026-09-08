import { createRouter } from "@solid-gpui/router";
import { routeTree } from "./routeTree.gen";

export function createWebsiteRouter(initialPath = "/") {
  return createRouter({ routeTree, initialEntries: [initialPath] });
}
