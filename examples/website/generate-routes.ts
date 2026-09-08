import { generateRoutes } from "../../packages/solid-gpui-router/src/generator";

await generateRoutes({ root: import.meta.dirname });
