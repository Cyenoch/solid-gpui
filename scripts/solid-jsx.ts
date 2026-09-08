import { plugin } from "bun";
import { transformJsx } from "../packages/solid-gpui-vite/src/transform";

// Test-only loader; applications compile JSX/TSX through @solid-gpui/vite.
plugin({
  name: "solid-gpui-jsx",
  setup(build) {
    build.onLoad({ filter: /\.[jt]sx$/ }, async (args) => {
      if (args.path.includes("node_modules")) return undefined;
      const source = await Bun.file(args.path).text();
      const result = transformJsx(source, args.path);
      return {
        contents: `${result.code}\n//# sourceMappingURL=data:application/json;base64,${Buffer.from(result.map).toString("base64")}`,
        loader: "js",
      };
    });
  },
});
