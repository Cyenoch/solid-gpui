#!/usr/bin/env bun
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { resolve } from "node:path";
import { parseArgs } from "node:util";

// Reviewed main-image imports, not an OS-version or dynamic dependency guarantee.
// New imports require review; in particular, no Bun/JSC or dynamic CRT is allowed.
const SYSTEM_IMPORTS: Record<string, true> = {
  "advapi32.dll": true,
  "api-ms-win-core-synch-l1-2-0.dll": true,
  "api-ms-win-core-winrt-error-l1-1-0.dll": true,
  "api-ms-win-core-winrt-l1-1-0.dll": true,
  "api-ms-win-shcore-scaling-l1-1-1.dll": true,
  "bcryptprimitives.dll": true,
  "combase.dll": true,
  "comctl32.dll": true,
  "crypt32.dll": true,
  "d3d11.dll": true,
  "d3dcompiler_47.dll": true,
  "dbghelp.dll": true,
  "dcomp.dll": true,
  "dwmapi.dll": true,
  "dwrite.dll": true,
  "dxgi.dll": true,
  "gdi32.dll": true,
  "icuuc.dll": true,
  "imm32.dll": true,
  "iphlpapi.dll": true,
  "kernel32.dll": true,
  "ntdll.dll": true,
  "ole32.dll": true,
  "oleaut32.dll": true,
  "shell32.dll": true,
  "uiautomationcore.dll": true,
  "user32.dll": true,
  "userenv.dll": true,
  "winmm.dll": true,
  "ws2_32.dll": true,
};

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      exe: { type: "string" },
      arch: { type: "string" },
      readobj: { type: "string", default: "llvm-readobj" },
      help: { type: "boolean" },
    },
  });
  if (values.help) {
    console.log(
      "Usage: bun scripts/check-embedded-dependencies.ts --exe <application.exe> --arch x64|arm64 [--readobj <llvm-readobj>]",
    );
    return;
  }
  if (!values.exe || (values.arch !== "x64" && values.arch !== "arm64"))
    throw new Error("--exe and --arch x64|arm64 are required");
  const image = resolve(values.exe);
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(image)) hash.update(chunk);
  const sha256 = hash.digest("hex");
  const child = Bun.spawn([values.readobj, "--file-headers", "--coff-imports", image], {
    stdin: "ignore",
    stdout: "pipe",
    stderr: "pipe",
    timeout: 120_000,
  });
  const [output, stderr, status] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  if (status !== 0 || stderr.trim()) throw new Error(`llvm-readobj failed (${status}): ${stderr.trim()}`);
  const stdout = output.replace(/\r\n/g, "\n");
  const machine = /^  Machine: IMAGE_FILE_MACHINE_(AMD64|ARM64) \((0x[0-9A-Fa-f]+)\)$/m.exec(stdout);
  if (!/^Format: COFF-(x86-64|ARM64)$/m.test(stdout) || !/^AddressSize: 64bit$/m.test(stdout) || !machine) {
    throw new Error("Expected a 64-bit Windows executable and complete LLVM COFF headers");
  }
  const expectedMachine = values.arch === "x64" ? 0x8664 : 0xaa64;
  const violations: string[] = [];
  if (Number(machine[2]) !== expectedMachine) violations.push(`Expected ${values.arch}, found ${machine[1]}`);
  const imports: { normal: string[]; delay: string[] } = { normal: [], delay: [] };
  let active: "normal" | "delay" | undefined;
  let name: string | undefined;
  // LLVM's top-level scopes start at column zero. DelayImport has nested Import
  // scopes, which must not be mistaken for main-image normal imports.
  for (const line of stdout.split(/\r?\n/)) {
    if (line === "Import {" || line === "DelayImport {") {
      if (active) throw new Error("Unclosed LLVM import record");
      active = line === "Import {" ? "normal" : "delay";
    } else if (active && line.startsWith("  Name:")) {
      if (name) throw new Error("Duplicate LLVM import name");
      name = /^  Name: ([a-z0-9_.-]+\.dll)$/i.exec(line)?.[1]?.toLowerCase();
      if (!name) throw new Error("Invalid LLVM import name");
    } else if (active && line === "}") {
      if (!name) throw new Error("Missing LLVM import name");
      imports[active].push(name);
      if (SYSTEM_IMPORTS[name] !== true)
        violations.push(`${active} import ${name} is outside the reviewed system dependency set`);
      active = undefined;
      name = undefined;
    }
  }
  if (active) throw new Error("Truncated LLVM import record");
  for (const [kind, field] of [
    ["normal", "ImportTable"],
    ["delay", "DelayImportDescriptor"],
  ] as const) {
    const rva = new RegExp(`^    ${field}RVA: (0x[0-9A-Fa-f]+)$`, "m").exec(stdout)?.[1];
    const size = new RegExp(`^    ${field}Size: (0x[0-9A-Fa-f]+)$`, "m").exec(stdout)?.[1];
    if (
      !rva ||
      !size ||
      Boolean(Number(rva)) !== Boolean(Number(size)) ||
      Boolean(Number(rva)) !== Boolean(imports[kind].length)
    ) {
      throw new Error(`Missing or inconsistent ${kind} import directory evidence`);
    }
  }
  if (imports.normal.length + imports.delay.length === 0)
    throw new Error("No imports: this is not a packaged Windows application");
  const after = createHash("sha256");
  for await (const chunk of createReadStream(image)) after.update(chunk);
  if (after.digest("hex") !== sha256) throw new Error("Image changed during inspection");
  console.log(
    JSON.stringify(
      {
        image,
        sha256,
        machine: machine[1],
        imports,
        violations,
        scope:
          "Main-image imports only. Runtime loads, OS/API-set resolution, graphics, fonts and resource accesses remain separate qualification gates.",
      },
      null,
      2,
    ),
  );
  if (violations.length) process.exitCode = 1;
}

if (import.meta.main)
  await main().catch((error: Error) => {
    console.error(error.message);
    process.exitCode = 2;
  });
