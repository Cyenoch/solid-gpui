#!/usr/bin/env bun

import { doctor, formatDoctorReport } from "./doctor.ts";
import { EMBEDDED_USAGE, runEmbeddedCommand } from "./embedded.ts";
import { checkProject, prepareProject, previewApplication, type PreparedProject } from "./project.ts";
import { runTests } from "./test.ts";

const USAGE = `solid-gpui <command> [options]

Commands
  prepare              Build the native host, publish its bindings and write the prepared
                       TypeScript config and artifact record under .solid-gpui/
  doctor               Report package, Cargo and runtime prerequisites of this application
  test [-- <args...>]  Run this application's tests with its own Vite configuration
  preview [-- <args...>]  Run the built host against the production bundle
  embedded <args...>   ${EMBEDDED_USAGE}

Options
  --check          prepare: verify generated files without writing (exit 1 when stale)
  --json           prepare: print the artifact record to stdout
  --root <dir>     project root (default: the working directory)
  --config <file>  Vite config file (default: Vite's own discovery)
  --mode <mode>    Vite mode (default: production)
  -h, --help       show this message

Every command reads the same vite.config.ts the application uses, so no path is maintained twice.`;

/** Flags that take a value; everything else is a boolean switch. */
const VALUED_OPTIONS = ["root", "config", "mode"];

interface ParsedArguments {
  readonly command?: string;
  readonly options: Readonly<Record<string, string | true>>;
  /** Positional arguments and everything after `--`, forwarded verbatim. */
  readonly rest: readonly string[];
  /** Index of the command token, so delegation can take everything after it untouched. */
  readonly commandIndex: number;
}

function parseArguments(argv: readonly string[]): ParsedArguments {
  const options: Record<string, string | true> = {};
  const rest: string[] = [];
  let command: string | undefined;
  let commandIndex = -1;
  for (let index = 0; index < argv.length; index++) {
    const argument = argv[index]!;
    if (argument === "--") {
      rest.push(...argv.slice(index + 1));
      break;
    }
    if (argument.startsWith("--")) {
      const name = argument.slice(2);
      const equals = name.indexOf("=");
      if (equals >= 0) options[name.slice(0, equals)] = name.slice(equals + 1);
      else if (VALUED_OPTIONS.includes(name)) {
        const value = argv[++index];
        if (value === undefined) throw new Error(`--${name} requires a value`);
        options[name] = value;
      } else options[name] = true;
      continue;
    }
    if (argument === "-h") {
      options.help = true;
      continue;
    }
    if (!command) {
      command = argument;
      commandIndex = index;
    } else rest.push(argument);
  }
  return { command, options, rest, commandIndex };
}

/**
 * Everything after the command, minus this tool's own options. A delegated command keeps its own
 * flag grammar, so `test --coverage` or `--test-name-pattern x` reach it rather than being consumed
 * here; `--` is still accepted as an explicit separator.
 */
function forwardedArguments(argv: readonly string[], commandIndex: number): string[] {
  const forwarded: string[] = [];
  for (let index = commandIndex + 1; index < argv.length; index++) {
    const argument = argv[index]!;
    if (argument === "--") {
      forwarded.push(...argv.slice(index + 1));
      break;
    }
    const name = argument.startsWith("--") ? argument.slice(2).split("=", 1)[0]! : "";
    if (VALUED_OPTIONS.includes(name)) {
      if (!argument.includes("=")) index++;
      continue;
    }
    forwarded.push(argument);
  }
  return forwarded;
}

function write(line: string): void {
  process.stderr.write(`${line}\n`);
}

function option(arguments_: ParsedArguments, name: string): string | undefined {
  const value = arguments_.options[name];
  return typeof value === "string" ? value : undefined;
}

function reportPrepared(prepared: PreparedProject, check: boolean): void {
  if (check) {
    write(`Generated files match their hosts (${prepared.files.length} checked).`);
    return;
  }
  for (const file of prepared.files) write(`${file.status === "written" ? "wrote" : "unchanged"} ${file.path}`);
  const { artifacts } = prepared;
  if (artifacts.host) write(`host ${artifacts.host.command}`);
  if (artifacts.bundle)
    write(`bundle ${artifacts.bundle} (produced by \`vite build\`; preview runs it with ${artifacts.runtime})`);
}

async function runPrepare(arguments_: ParsedArguments): Promise<number> {
  const project = {
    root: option(arguments_, "root"),
    configFile: option(arguments_, "config"),
    mode: option(arguments_, "mode"),
  };
  const check = arguments_.options.check === true;
  const prepared = await (check ? checkProject : prepareProject)({ ...project, log: write });
  if (arguments_.options.json === true) process.stdout.write(`${JSON.stringify(prepared.artifacts, null, 2)}\n`);
  else reportPrepared(prepared, check);
  return 0;
}

async function runDoctor(arguments_: ParsedArguments): Promise<number> {
  const report = await doctor({
    root: option(arguments_, "root"),
    configFile: option(arguments_, "config"),
  });
  write(formatDoctorReport(report));
  return report.status === "fail" ? 1 : 0;
}

async function main(argv: readonly string[]): Promise<number> {
  // The embedded packager owns its own flag grammar, so its arguments pass through untouched.
  if (argv[0] === "embedded") return runEmbeddedCommand(argv.slice(1), { cwd: process.cwd() });
  const arguments_ = parseArguments(argv);
  if (arguments_.options.help === true) {
    write(USAGE);
    return 0;
  }
  if (!arguments_.command) {
    write(USAGE);
    return 2;
  }
  const project = {
    root: option(arguments_, "root"),
    configFile: option(arguments_, "config"),
    mode: option(arguments_, "mode"),
  };
  switch (arguments_.command) {
    case "prepare":
      return runPrepare(arguments_);
    case "doctor":
      return runDoctor(arguments_);
    case "test":
      return runTests({
        root: project.root,
        configFile: project.configFile,
        args: forwardedArguments(argv, arguments_.commandIndex),
      });
    case "preview":
      return previewApplication({
        root: project.root,
        configFile: project.configFile,
        args: forwardedArguments(argv, arguments_.commandIndex),
      });
    default:
      write(`Unknown command: ${arguments_.command}\n\n${USAGE}`);
      return 2;
  }
}

if (import.meta.main) {
  try {
    process.exitCode = await main(process.argv.slice(2));
  } catch (error) {
    write(`solid-gpui: ${error instanceof Error ? error.message : String(error)}`);
    process.exitCode = 1;
  }
}
