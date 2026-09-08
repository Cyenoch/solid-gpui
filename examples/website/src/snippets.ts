export const landingCode = `const [count, setCount] = createSignal(0);

<Pressable onPress={() => setCount(count() + 1)}>
  <Text>Count: {count()}</Text>
</Pressable>`;

export const installCode = "bun install --frozen-lockfile\nbun run website:native";
export const importCode = (name: string) => `import { ${name} } from "@solid-gpui/core/components";`;
export const excerptCode = (source: string) => source.split("\n").slice(0, 3).join("\n");
export const snippetKey = (source: string, language: string) => JSON.stringify([language, source]);
