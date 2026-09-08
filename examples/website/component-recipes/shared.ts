import type { ComponentVariant } from "../component-variants";

export type Recipe = { id: string; title: string; description: string; jsx: string; setup?: string; preamble?: string };

/** Compound parts share executable compositions rather than isolated, invalid previews. */
export function recipes(names: string, examples: Recipe[]): ComponentVariant[] {
  const components = names.split(" ");
  return examples.flatMap(({ id, title, description, jsx, setup = "", preamble = "" }) => {
    const source = `import * as N from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
${setup.includes("createSignal") ? 'import { createSignal } from "@solid-gpui/core/runtime";' : ""}

${preamble}

export default function Example() {
${setup}
  return <View style={{ gap: 16 }}>${jsx}</View>;
}`;
    return components.map((component) => ({ component, id: `${components[0]}--${id}`, title, description, source }));
  });
}
