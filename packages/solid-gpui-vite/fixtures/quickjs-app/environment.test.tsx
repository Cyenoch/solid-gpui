import { expect, test } from "bun:test";
import { createSignal } from "solid-js";
import { createRoot } from "@solid-gpui/core";
import { TestHost } from "@solid-gpui/core/testing";
import { Probe } from "./src/probe";

test("reads and mutates the real environment and passes it to a PATH-resolved subprocess", () => {
  expect(process.env.SOLID_GPUI_FIXTURE_SENTINEL).toBe("inherited-sentinel");
  const previous = process.env.SOLID_GPUI_FIXTURE_MUTATED;
  process.env.SOLID_GPUI_FIXTURE_MUTATED = "mutated-value";
  try {
    expect(process.env.SOLID_GPUI_FIXTURE_MUTATED).toBe("mutated-value");
    // Print only fixture variables, never the caller's full environment.
    const child = Bun.spawnSync({
      cmd: [
        "bun",
        "-e",
        `process.stdout.write(JSON.stringify({
        sentinel: process.env.SOLID_GPUI_FIXTURE_SENTINEL,
        mutated: process.env.SOLID_GPUI_FIXTURE_MUTATED,
        hasPath: Boolean(process.env.PATH)
      }))`,
      ],
      env: { ...process.env },
      stdout: "pipe",
      stderr: "pipe",
    });
    expect(child.exitCode).toBe(0);
    expect(JSON.parse(child.stdout.toString())).toEqual({
      sentinel: "inherited-sentinel",
      mutated: "mutated-value",
      hasPath: true,
    });
  } finally {
    if (previous === undefined) delete process.env.SOLID_GPUI_FIXTURE_MUTATED;
    else process.env.SOLID_GPUI_FIXTURE_MUTATED = previous;
  }
});

test("still renders application TSX against one reactive graph", async () => {
  const [label, setLabel] = createSignal("first");
  const host = new TestHost();
  const root = createRoot(host.transport, { surfaceId: 11 });
  try {
    root.render(() => <Probe label={label()} />);
    expect(
      host
        .surface(11)!
        .nodes.filter((node) => node.text !== null)
        .map((node) => node.text),
    ).toEqual(["first"]);
    setLabel("second");
    await Promise.resolve();
    expect(
      host
        .surface(11)!
        .nodes.filter((node) => node.text !== null)
        .map((node) => node.text),
    ).toEqual(["second"]);
  } finally {
    root.unmount();
  }
});
