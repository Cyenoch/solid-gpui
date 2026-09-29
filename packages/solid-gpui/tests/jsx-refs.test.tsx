import { test } from "bun:test";

test("compiled JSX refs receive native handles and preserve Solid lifecycle", async () => {
  await import("../../solid-gpui-vite/fixtures/jsx-refs");
});
