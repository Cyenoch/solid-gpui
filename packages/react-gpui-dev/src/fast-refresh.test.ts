import { expect, test } from "bun:test";
import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { createElement, useState, type FunctionComponent, type ReactElement } from "react";
import { FastRefreshSession, type RefreshModule, type RefreshRoot } from "./fast-refresh";

import { transformRefreshSource } from "./transform";
class TestRoot {
  renderer: ReactTestRenderer | undefined;
  renders = 0;

  render(node: ReactElement): void {
    act(() => {
      if (this.renderer === undefined) this.renderer = create(node);
      else this.renderer.update(node);
    });
    this.renders += 1;
  }

  text(): string {
    const button = this.renderer?.root.findByType("button");
    return button?.children.join("") ?? "";
  }
}

let childImplementation: (props: {}) => ReactElement = function ChildV1() {
  const [count, setCount] = useState(0);
  return createElement("button", { onClick: () => setCount((value) => value + 1) }, `v1:${count}`);
};

function StableChild(): ReactElement {
  return childImplementation({});
}

function ParentV1(): ReactElement {
  return createElement(StableChild);
}

function ParentV2(): ReactElement {
  return createElement(StableChild);
}

const asRoot = (root: TestRoot): RefreshRoot => root;

function moduleOf(component: FunctionComponent, signature?: string): RefreshModule<{}> {
  return signature === undefined ? { default: component } : { default: component, signature };
}

test("compatible nested family edit preserves hook state and native root identity", async () => {
  const root = new TestRoot();
  const session = new FastRefreshSession(asRoot(root), moduleOf(ParentV1, "parent"), {});
  const button = root.renderer?.root.findByType("button");
  act(() => button?.props.onClick());
  expect(root.text()).toBe("v1:1");

  childImplementation = function ChildV2() {
    const [count, setCount] = useState(0);
    return createElement("button", { onClick: () => setCount((value) => value + 1) }, `v2:${count}`);
  };
  const result = await session.refresh(async () => moduleOf(ParentV2, "parent"));
  expect(result.kind).toBe("preserved");
  expect(root.text()).toBe("v2:1");
  expect(root.renders).toBe(2);
});

test("incompatible signature remounts only the changed family", async () => {
  const root = new TestRoot();
  const session = new FastRefreshSession(asRoot(root), moduleOf(ParentV1, "one"), {});
  const result = await session.refresh(async () => moduleOf(ParentV2, "two"));
  expect(result.kind).toBe("remounted");
  expect(root.renders).toBe(2);
});

test("syntax/evaluation failure leaves last-good tree and root identity", async () => {
  childImplementation = function ChildV1() {
    const [count, setCount] = useState(0);
    return createElement("button", { onClick: () => setCount((value) => value + 1) }, `v1:${count}`);
  };
  const root = new TestRoot();
  const session = new FastRefreshSession(asRoot(root), moduleOf(ParentV1, "stable"), {});
  const renders = root.renders;
  const result = await session.refresh(async () => {
    throw new SyntaxError("unterminated JSX");
  });
  expect(result.kind).toBe("failed");
  expect(root.renders).toBe(renders);
  expect(root.text()).toBe("v1:0");
  expect(session.lastGoodModule.default).toBe(ParentV1);
});

test("Babel transform registers every component family and hook signature", async () => {
  const code = await transformRefreshSource(
    "export function Parent() { return <Child />; } function Child() { const [value] = useState(1); return <View>{value}</View>; }",
    "/workspace/entry.tsx",
  );
  expect(code).toContain("$RefreshReg$");
  expect(code).toContain("$RefreshSig$");
});
