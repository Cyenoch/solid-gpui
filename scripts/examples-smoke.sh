#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
examples_dir="$repo_root/fixtures"
smoke_root="$(mktemp -d "${TMPDIR:-/tmp}/solid-gpui-examples-smoke.XXXXXX")"
trap 'rm -rf -- "$smoke_root"' EXIT

startup_timeout="${EXAMPLES_SMOKE_STARTUP_TIMEOUT_SECONDS:-15}"
term_timeout="${EXAMPLES_SMOKE_TERM_TIMEOUT_SECONDS:-1}"
kill_timeout="${EXAMPLES_SMOKE_KILL_TIMEOUT_SECONDS:-5}"
build_log="$smoke_root/build.log"
target_started="$(python3 -c 'import time; print(time.monotonic())')"

if ! (
  cd "$repo_root"
  cargo build -p solid-gpui --bin solid-gpui-host --release --locked
) >"$build_log" 2>&1; then
  printf '%s\n' 'examples-smoke: release host build failed' >&2
  cat "$build_log" >&2
  exit 1
fi

binary="$repo_root/target/release/solid-gpui-host"
[[ -x "$binary" ]] || {
  printf 'examples-smoke: release host is missing or not executable: %s\n' "$binary" >&2
  exit 1
}

python3 - "$binary" "$examples_dir" "$smoke_root" "$startup_timeout" "$term_timeout" "$kill_timeout" "$target_started" <<'PY'
import os
import re
import signal
import subprocess
import sys
import time
from pathlib import Path

binary, examples_dir_text, smoke_root_text, startup_text, term_text, kill_text, target_started_text = sys.argv[1:]
examples_dir = Path(examples_dir_text)
smoke_root = Path(smoke_root_text)
preload = str(examples_dir.parent / "scripts/solid-jsx.ts")
startup_timeout = float(startup_text)
term_timeout = float(term_text)
kill_timeout = float(kill_text)
target_started = float(target_started_text)
if startup_timeout <= 0 or term_timeout <= 0 or kill_timeout <= 0:
    raise SystemExit("examples-smoke: timeout values must be positive")

entries = [examples_dir / "press-roundtrip.ts"]
if not entries:
    raise SystemExit(f"examples-smoke: no .ts/.tsx entries found in {examples_dir}")

startup_pattern = re.compile(rb"solid-gpui-host: starting mode=Process protocol=v5 entry=bun pid=\d+")
allowed_exit_codes = {0, -signal.SIGTERM, 128 + signal.SIGTERM}

def read_tail(path: Path, lines: int = 80) -> str:
    try:
        content = path.read_text(encoding="utf-8", errors="replace")
    except FileNotFoundError:
        return ""
    return "\n".join(content.splitlines()[-lines:])

def process_group_members(pgid: int):
    result = subprocess.run(
        ["ps", "-axo", "pid=,pgid=,stat=,command="],
        check=False,
        capture_output=True,
        text=True,
    )
    members = []
    for line in result.stdout.splitlines():
        fields = line.strip().split(None, 3)
        if len(fields) >= 2 and fields[1] == str(pgid):
            members.append(line.strip())
    return members

def terminate(process: subprocess.Popen, pgid: int):
    forced_kill = False
    if process.poll() is None:
        try:
            os.killpg(pgid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=term_timeout)
        except subprocess.TimeoutExpired:
            forced_kill = True
            try:
                os.killpg(pgid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=kill_timeout)
            except subprocess.TimeoutExpired:
                return forced_kill, process.poll(), ["host process did not exit after SIGKILL"]

    deadline = time.monotonic() + kill_timeout
    members = process_group_members(pgid)
    if members:
        forced_kill = True
        try:
            os.killpg(pgid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        while members and time.monotonic() < deadline:
            time.sleep(0.05)
            members = process_group_members(pgid)
    return forced_kill, process.poll(), members

def report_failure(name: str, reason: str, stderr_path: Path, stdout_path: Path):
    print(f"examples-smoke: {name}: FAIL: {reason}", file=sys.stderr)
    stderr_tail = read_tail(stderr_path)
    if stderr_tail:
        print(f"--- {name} stderr tail ---", file=sys.stderr)
        print(stderr_tail, file=sys.stderr)
    stdout_tail = read_tail(stdout_path)
    if stdout_tail:
        print(f"--- {name} stdout tail ---", file=sys.stderr)
        print(stdout_tail, file=sys.stderr)

print("examples-smoke: entries=" + ",".join(entry.stem for entry in entries))
print("examples-smoke: startup watchdog=" + f"{startup_timeout:.3f}s")
results = []
for entry in entries:
    name = entry.stem
    stderr_path = smoke_root / f"{name}.stderr"
    stdout_path = smoke_root / f"{name}.stdout"
    environment = os.environ.copy()
    environment["SOLID_GPUI_LOG"] = "info"
    stderr_file = stderr_path.open("w", encoding="utf-8")
    stdout_file = stdout_path.open("w", encoding="utf-8")
    started = time.monotonic()
    process = None
    pgid = None
    startup_elapsed = None
    failure = None
    forced_kill = False
    exit_code = None
    members = []
    try:
        process = subprocess.Popen(
            [binary, "--runtime", "process", "--", "bun", "run", "--conditions=browser", "--preload", preload, entry.name],
            cwd=examples_dir,
            env=environment,
            stdout=stdout_file,
            stderr=stderr_file,
            start_new_session=True,
            text=True,
        )
        pgid = process.pid
        deadline = started + startup_timeout
        while True:
            if startup_pattern.search(stderr_path.read_bytes()):
                startup_elapsed = time.monotonic() - started
                break
            exit_code = process.poll()
            if exit_code is not None:
                failure = f"exited before startup diagnostic (code={exit_code})"
                break
            if time.monotonic() >= deadline:
                failure = f"timed out after {startup_timeout:.3f}s waiting for protocol=v5 startup diagnostic"
                break
            time.sleep(0.01)
    except OSError as error:
        failure = f"failed to spawn host: {error}"
    finally:
        if process is not None:
            forced_kill, exit_code, members = terminate(process, pgid)
        stderr_file.close()
        stdout_file.close()

    elapsed = time.monotonic() - started
    if failure is None and startup_elapsed is None:
        failure = "startup diagnostic was not observed"
    if failure is None and forced_kill:
        failure = "host or renderer group required SIGKILL after SIGTERM"
    if failure is None and exit_code not in allowed_exit_codes:
        failure = f"unexpected exit code after SIGTERM: {exit_code}"
    if failure is None and members:
        failure = "process-group members remained after teardown: " + "; ".join(members)
    if failure is not None:
        report_failure(name, failure, stderr_path, stdout_path)
        verdict = "FAIL"
    else:
        verdict = "PASS"
    results.append((name, elapsed, startup_elapsed, verdict, exit_code, forced_kill))
    startup_display = "n/a" if startup_elapsed is None else f"{startup_elapsed:.3f}s"
    print(
        f"examples-smoke: {name}: startup={startup_display} elapsed={elapsed:.3f}s "
        f"exit={exit_code} ps=clean verdict={verdict}"
    )

passed = sum(result[3] == "PASS" for result in results)
failed = len(results) - passed
total_elapsed = time.monotonic() - target_started
print(
    f"examples-smoke: total={total_elapsed:.3f}s launch_loop={sum(result[1] for result in results):.3f}s "
    f"passed={passed} failed={failed}"
)
if failed:
    print("examples-smoke: failed examples=" + ",".join(result[0] for result in results if result[3] == "FAIL"), file=sys.stderr)
    raise SystemExit(1)
PY
