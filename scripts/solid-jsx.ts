import { plugin } from "bun";

// Test-only loader; applications compile JSX/TSX through @solid-gpui/vite.
plugin({
  name: "solid-gpui-jsx",
  setup(build) {
    build.onLoad({ filter: /\.[jt]sx$/ }, async (args) => {
      if (args.path.includes("node_modules")) return undefined;
      const source = await Bun.file(args.path).text();
      // Compiler bindings are platform-specific; non-JSX commands must not load them.
      const { transformJsx } = await import("../packages/solid-gpui-vite/src/transform");
      const result = transformJsx(source, args.path);
      return {
        contents: `${result.code}\n//# sourceMappingURL=data:application/json;base64,${Buffer.from(result.map).toString("base64")}`,
        loader: "js",
      };
    });
  },
});
