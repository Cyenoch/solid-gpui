import { afterAll, expect, test } from "bun:test";
import { rm } from "node:fs/promises";
import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { type ReactElement } from "react";
import { FastRefreshSession, type RefreshModule, type RefreshRoot } from "./fast-refresh";
import { installFastRefreshTransform, transformRefreshSource } from "./transform";

class RootHarness implements RefreshRoot {
  renderer: ReactTestRenderer | undefined;
  renders = 0;

  render(element: ReactElement): void {
    act(() => {
      if (this.renderer === undefined) this.renderer = create(element);
      else this.renderer.update(element);
    });
    this.renders += 1;
  }

  text(): string {
    return this.renderer?.root.findByType("button").children.join("") ?? "";
  }
}


afterAll(async () => {
  await rm(`${import.meta.dir}/.refresh-integration`, { recursive: true, force: true });
});
const childSource = (label: string, extraHook: boolean): string => `
import React, { useState } from "react";
export function NestedChild() {
  ${extraHook ? "useState(0);" : ""}
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount((value) => value + 1)}>${label}:{count}</button>;
}
export default NestedChild;
`;

const parentSource = (childUrl: string): string => `
import React from "react";
import Child from ${JSON.stringify(childUrl)};
export default function Parent() { return <Child />; }
`;

async function writeModule(path: string, source: string): Promise<void> {
  await Bun.write(path, source);
}
async function transformedDataUrl(source: string, path: string): Promise<string> {
  const code = await transformRefreshSource(source, path);
  return `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
}

async function importParent(
  childSourceText: string,
  parentSourceText: (childUrl: string) => string,
  childPath: string,
  parentPath: string,
  revision: number,
): Promise<RefreshModule<{}>> {
  void revision;
  const childUrl = await transformedDataUrl(childSourceText, childPath);
  const parentUrl = await transformedDataUrl(parentSourceText(childUrl), parentPath);
  const module = await import(parentUrl) as { default: () => ReactElement };
  return { default: module.default };
}

test("Babel-transformed nested families preserve and remount hook state", async () => {
  const directory = `${import.meta.dir}/.refresh-integration`;
  await installFastRefreshTransform(directory);
  const childPath = `${directory}/NestedChild.tsx`;
  const parentPath = `${directory}/Parent.tsx`;
  const root = new RootHarness();

  await writeModule(childPath, childSource("v1", false));
  await writeModule(parentPath, parentSource("./NestedChild.tsx"));
  const initial = await importParent(childSource("v1", false), parentSource, childPath, parentPath, 1);
  const session = new FastRefreshSession(root, initial, {});
  const button = root.renderer?.root.findByType("button");
  act(() => button?.props.onClick());
  expect(root.text()).toBe("v1:1");

  await writeModule(childPath, childSource("v2", false));
  await writeModule(parentPath, parentSource("./NestedChild.tsx"));
  const compatible = await session.refresh(() =>
    importParent(childSource("v2", false), parentSource, childPath, parentPath, 2),
  );
  expect(compatible.kind).toBe("preserved");
  expect(root.text()).toBe("v2:1");
  const rootRenders = root.renders;

  await writeModule(childPath, childSource("v3", true));
  await writeModule(parentPath, parentSource("./NestedChild.tsx"));
  const incompatible = await session.refresh(() =>
    importParent(childSource("v3", true), parentSource, childPath, parentPath, 3),
  );
  expect(incompatible.kind).toBe("preserved");
  expect(root.text()).toBe("v3:0");
  expect(root.renders).toBe(rootRenders + 1);

  const failed = await session.refresh(() =>
    importParent("export default function NestedChild() { return <button>", parentSource, childPath, parentPath, 4),
  );
  expect(failed.kind).toBe("failed");
  expect(root.text()).toBe("v3:0");
  expect(root.renderer).toBeDefined();
});
