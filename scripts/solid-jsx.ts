import { transformSync } from "@babel/core";
import typescript from "@babel/preset-typescript";
import { plugin } from "bun";
import solid from "babel-preset-solid";

plugin({
  name: "solid-gpui-jsx",
  setup(build) {
    build.onLoad({ filter: /\.tsx$/ }, async (args) => {
      if (args.path.includes("node_modules")) return undefined;
      const source = await Bun.file(args.path).text();
      const result = transformSync(source, {
        filename: args.path,
        babelrc: false,
        configFile: false,
        presets: [typescript, [solid, { generate: "universal", moduleName: "@solid-gpui/core/runtime" }]],
        sourceMaps: "inline",
      });
      if (!result?.code) {
        throw new Error(`solid-gpui jsx transform produced no code for ${args.path}`);
      }
      return {
        contents: result.code,
        loader: "js",
      };
    });
  },
});
