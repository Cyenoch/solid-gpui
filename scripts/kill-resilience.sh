#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
run_parent="$repo_root/.scratch/kill-resilience"
mkdir -p "$run_parent"
run_root="$(mktemp -d "$run_parent/runtime.XXXXXX")"
trap 'rm -rf -- "$run_root"' EXIT

startup_timeout="${KILL_RESILIENCE_STARTUP_TIMEOUT_SECONDS:-10}"
kill_timeout="${KILL_RESILIENCE_KILL_TIMEOUT_SECONDS:-5}"
build_log="$run_root/build.log"

if ! (
  cd "$repo_root"
  cargo build -p react-gpui-host --release --locked
) >"$build_log" 2>&1; then
  printf '%s\n' 'kill-resilience: release host build failed' >&2
  cat "$build_log" >&2
  exit 1
fi

binary="$repo_root/target/$(rustc -vV | python3 -c 'import sys; print(next(line.split(": ", 1)[1] for line in sys.stdin if line.startswith("host: ")))')/release/react-gpui-host"
[[ -x "$binary" ]] || {
  printf 'kill-resilience: release host is missing or not executable: %s\n' "$binary" >&2
  exit 1
}

entries="${KILL_RESILIENCE_ENTRIES:-counter.tsx,gallery.tsx}"
python3 - "$binary" "$repo_root" "$run_root" "$startup_timeout" "$kill_timeout" "$entries" <<'PY'
import os
import re
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

binary_text, repo_root_text, run_root_text, startup_text, kill_text, entries_text = sys.argv[1:]
binary = Path(binary_text).resolve()
repo_root = Path(repo_root_text).resolve()
run_root = Path(run_root_text).resolve()
examples_dir = repo_root / "packages" / "react-gpui" / "examples"
startup_timeout = float(startup_text)
kill_timeout = float(kill_text)
if startup_timeout <= 0 or kill_timeout <= 0:
    raise SystemExit("kill-resilience: timeout values must be positive")

shutil_which = shutil.which("lsof")
if not shutil_which:
    raise SystemExit("kill-resilience: lsof is required for cwd verification before kills")

entry_paths = []
for entry_text in entries_text.split(","):
    name = entry_text.strip()
    if not name:
        continue
    entry = (examples_dir / name).resolve()
    if entry.parent != examples_dir or entry.suffix != ".tsx" or not entry.is_file():
        raise SystemExit(f"kill-resilience: invalid example entry: {name}")
    entry_paths.append(entry)
if not entry_paths:
    raise SystemExit("kill-resilience: no example entries selected")

startup_pattern = re.compile(rb"react-gpui-host: starting mode=Process protocol=v3 entry=bun pid=(\d+)")


def read_tail(path: Path, lines: int = 80) -> str:
    try:
        content = path.read_text(encoding="utf-8", errors="replace")
    except FileNotFoundError:
        return ""
    return "\n".join(content.splitlines()[-lines:])


def parse_ps_line(line: str):
    fields = line.strip().split(None, 4)
    if len(fields) < 5:
        return None
    try:
        return {
            "pid": int(fields[0]),
            "ppid": int(fields[1]),
            "pgid": int(fields[2]),
            "stat": fields[3],
            "command": fields[4],
        }
    except ValueError:
        return None


def ps_process(pid: int):
    result = subprocess.run(
        ["ps", "-p", str(pid), "-o", "pid=,ppid=,pgid=,stat=,command="],
        check=False,
        capture_output=True,
        text=True,
        timeout=1.0,
    )
    for line in result.stdout.splitlines():
        process = parse_ps_line(line)
        if process is not None and process["pid"] == pid:
            return process
    return None


def ps_all():
    result = subprocess.run(
        ["ps", "-axo", "pid=,ppid=,pgid=,stat=,command="],
        check=False,
        capture_output=True,
        text=True,
        timeout=1.0,
    )
    return [process for line in result.stdout.splitlines() if (process := parse_ps_line(line)) is not None]


def ps_group(pgid: int):
    return [process for process in ps_all() if process["pgid"] == pgid]


def process_cwd(pid: int):
    result = subprocess.run(
        [shutil_which, "-a", "-p", str(pid), "-d", "cwd", "-Fn"],
        check=False,
        capture_output=True,
        text=True,
        timeout=5.0,
    )
    paths = [line[1:] for line in result.stdout.splitlines() if line.startswith("n")]
    return os.path.realpath(paths[-1]) if paths else None


def verify_process(pid: int, run_dir: Path, role: str, pgid: int | None = None):
    process = ps_process(pid)
    if process is None:
        return None
    cwd = process_cwd(pid)
    expected_cwd = os.path.realpath(run_dir)
    if cwd != expected_cwd:
        raise RuntimeError(
            f"refusing to signal {role} pid={pid}: cwd={cwd!r}, expected scratch cwd={expected_cwd!r}"
        )
    if pgid is not None and process["pgid"] != pgid:
        raise RuntimeError(
            f"refusing to signal {role} pid={pid}: pgid={process['pgid']}, expected {pgid}"
        )
    if role == "host" and binary.name not in process["command"]:
        raise RuntimeError(f"refusing to signal pid={pid}: command is not the spawned host: {process['command']}")
    if role == "renderer" and "bun" not in process["command"]:
        raise RuntimeError(f"refusing to signal pid={pid}: command is not the spawned Bun renderer: {process['command']}")
    return process


def safe_kill_pid(pid: int, run_dir: Path, role: str, sig: signal.Signals, pgid: int | None = None):
    process = verify_process(pid, run_dir, role, pgid)
    if process is None:
        return False
    try:
        os.kill(pid, sig)
    except ProcessLookupError:
        return False
    return True


def safe_kill_group(pgid: int, run_dir: Path, sig: signal.Signals):
    members = ps_group(pgid)
    expected_cwd = os.path.realpath(run_dir)
    for member in members:
        cwd = process_cwd(member["pid"])
        if cwd != expected_cwd:
            raise RuntimeError(
                f"refusing to signal process group {pgid}: member pid={member['pid']} cwd={cwd!r}, "
                f"expected scratch cwd={expected_cwd!r}"
            )
    if members:
        try:
            os.killpg(pgid, sig)
        except ProcessLookupError:
            pass
    return members


def wait_for(predicate, timeout: float, interval: float = 0.02):
    deadline = time.monotonic() + timeout
    while True:
        if predicate():
            return True
        if time.monotonic() >= deadline:
            return False
        time.sleep(interval)


def wait_gone(pid: int, timeout: float):
    return wait_for(lambda: ps_process(pid) is None, timeout)


def wait_host_exit(host: subprocess.Popen, timeout: float):
    try:
        host.wait(timeout=timeout)
        return True
    except subprocess.TimeoutExpired:
        return False


def find_renderer(host_pid: int, pgid: int, entry: Path, run_dir: Path, timeout: float):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        for process in ps_all():
            if process["ppid"] != host_pid or process["pgid"] != pgid or "bun" not in process["command"]:
                continue
            if entry.name not in process["command"]:
                continue
            if process_cwd(process["pid"]) == os.path.realpath(run_dir):
                return process["pid"]
        time.sleep(0.02)
    return None


def wait_startup(host: subprocess.Popen, stderr_path: Path, timeout: float):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            data = stderr_path.read_bytes()
        except FileNotFoundError:
            data = b""
        match = startup_pattern.search(data)
        if match:
            return int(match.group(1))
        if host.poll() is not None:
            return None
        time.sleep(0.02)
    return None


def cleanup(host: subprocess.Popen | None, renderer_pid: int | None, run_dir: Path, pgid: int):
    errors = []
    if renderer_pid is not None and ps_process(renderer_pid) is not None:
        try:
            safe_kill_pid(renderer_pid, run_dir, "renderer", signal.SIGKILL, pgid)
        except (OSError, RuntimeError) as error:
            errors.append(str(error))
    if host is not None and host.poll() is None:
        try:
            safe_kill_pid(host.pid, run_dir, "host", signal.SIGKILL, pgid)
        except (OSError, RuntimeError) as error:
            errors.append(str(error))
    if host is not None:
        try:
            host.wait(timeout=kill_timeout)
        except subprocess.TimeoutExpired:
            errors.append(f"host pid={host.pid} did not become waitable during cleanup")
    if renderer_pid is not None and ps_process(renderer_pid) is not None:
        errors.append(f"renderer pid={renderer_pid} remained after cleanup")
    return errors


def report_failure(label: str, reason: str, stderr_path: Path, stdout_path: Path):
    print(f"kill-resilience: {label}: FAIL: {reason}", file=sys.stderr)
    stderr_tail = read_tail(stderr_path)
    if stderr_tail:
        print(f"--- {label} stderr tail ---", file=sys.stderr)
        print(stderr_tail, file=sys.stderr)
    stdout_tail = read_tail(stdout_path)
    if stdout_tail:
        print(f"--- {label} stdout tail ---", file=sys.stderr)
        print(stdout_tail, file=sys.stderr)


def run_case(entry: Path, scenario: str, case_number: int):
    label = f"{entry.stem}/{scenario}"
    run_dir = run_root / f"case-{case_number}-{entry.stem}-{scenario}"
    run_dir.mkdir()
    stdout_path = run_dir / "host.stdout"
    stderr_path = run_dir / "host.stderr"
    stdout_file = stdout_path.open("w", encoding="utf-8")
    stderr_file = stderr_path.open("w", encoding="utf-8")
    environment = os.environ.copy()
    environment["REACT_GPUI_LOG"] = "info"
    host = None
    renderer_pid = None
    pgid = None
    result = {
        "label": label,
        "entry": entry.name,
        "scenario": scenario,
        "verdict": "FAIL",
        "latency": None,
        "host_exit": None,
        "renderer": "unknown",
        "cause": "unknown",
        "reason": "not started",
    }
    try:
        host = subprocess.Popen(
            [str(binary), "--runtime", "process", "--", "bun", "run", str(entry)],
            cwd=run_dir,
            env=environment,
            stdout=stdout_file,
            stderr=stderr_file,
            start_new_session=True,
            text=True,
        )
        pgid = host.pid
        host_process = verify_process(host.pid, run_dir, "host", pgid)
        if host_process is None:
            raise RuntimeError(f"spawned host pid={host.pid} disappeared before startup")
        startup_host_pid = wait_startup(host, stderr_path, startup_timeout)
        if startup_host_pid != host.pid:
            raise RuntimeError(
                f"startup diagnostic host pid mismatch: diagnostic={startup_host_pid}, process={host.pid}"
            )
        renderer_pid = find_renderer(host.pid, pgid, entry, run_dir, startup_timeout)
        if renderer_pid is None:
            raise RuntimeError("spawned Bun renderer was not found as a scratch-cwd child")
        verify_process(renderer_pid, run_dir, "renderer", pgid)

        if scenario == "host-pid":
            killed_at = time.monotonic()
            if not safe_kill_pid(host.pid, run_dir, "host", signal.SIGKILL, pgid):
                raise RuntimeError(f"host pid={host.pid} disappeared before SIGKILL")
            host.wait(timeout=kill_timeout)
            gone = wait_gone(renderer_pid, kill_timeout)
            result["latency"] = time.monotonic() - killed_at
            result["host_exit"] = host.returncode
            result["renderer"] = "gone" if gone else "present"
            stderr = read_tail(stderr_path, 200)
            if "StdioTransport input ended" in stderr:
                result["cause"] = "stdin-eof"
            elif "StdioTransport input closed" in stderr:
                result["cause"] = "stdin-eof-close"
            elif gone:
                result["cause"] = "renderer-exited-without-eof-diagnostic"
            else:
                result["cause"] = "renderer-still-running"
            if not gone:
                raise RuntimeError(
                    f"renderer pid={renderer_pid} remained {kill_timeout:.3f}s after host SIGKILL"
                )
            if result["cause"] not in {"stdin-eof", "stdin-eof-close"}:
                raise RuntimeError(f"renderer exited, but stdin EOF diagnostic was absent ({result['cause']})")
            result["verdict"] = "PASS"
            result["reason"] = "host SIGKILL; renderer observed stdin EOF"
        elif scenario == "renderer-pid":
            killed_at = time.monotonic()
            if not safe_kill_pid(renderer_pid, run_dir, "renderer", signal.SIGKILL, pgid):
                raise RuntimeError(f"renderer pid={renderer_pid} disappeared before SIGKILL")
            host_exited = wait_host_exit(host, kill_timeout)
            result["latency"] = time.monotonic() - killed_at
            result["host_exit"] = host.returncode
            result["renderer"] = "killed"
            stderr = read_tail(stderr_path, 200)
            typed = (
                "renderer runtime terminated unexpectedly" in stderr
                and "terminated by signal 9" in stderr
            )
            result["cause"] = "typed-child-death" if typed else "untyped-child-death"
            if not host_exited:
                raise RuntimeError(f"host pid={host.pid} remained {kill_timeout:.3f}s after renderer SIGKILL")
            if host.returncode != 1:
                raise RuntimeError(f"host exited with {host.returncode}, expected typed fatal exit 1")
            if not typed:
                raise RuntimeError("host stderr lacked typed child-death diagnostic")
            result["verdict"] = "PASS"
            result["reason"] = "renderer SIGKILL; host reported typed child death and exited 1"
        elif scenario == "group":
            verify_process(host.pid, run_dir, "host", pgid)
            verify_process(renderer_pid, run_dir, "renderer", pgid)
            members = ps_group(pgid)
            member_pids = {member["pid"] for member in members}
            if host.pid not in member_pids or renderer_pid not in member_pids:
                raise RuntimeError(
                    f"verified process group {pgid} changed before SIGKILL: members={sorted(member_pids)}"
                )
            members = safe_kill_group(pgid, run_dir, signal.SIGKILL)
            killed_at = time.monotonic()
            host_gone = wait_host_exit(host, kill_timeout)
            renderer_gone = wait_gone(renderer_pid, kill_timeout)
            result["latency"] = time.monotonic() - killed_at
            result["host_exit"] = host.returncode
            result["renderer"] = "gone" if renderer_gone else "present"
            result["cause"] = f"group-kill ({len(members)} verified scratch members)"
            if not host_gone or not renderer_gone:
                raise RuntimeError(
                    f"group SIGKILL left host={not host_gone} renderer={not renderer_gone}"
                )
            result["verdict"] = "PASS"
            result["reason"] = "verified scratch process group SIGKILL removed host and renderer together"
        else:
            raise RuntimeError(f"unknown scenario {scenario}")
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        result["reason"] = str(error)
    finally:
        stdout_file.close()
        stderr_file.close()
        cleanup_errors = cleanup(host, renderer_pid, run_dir, pgid or -1)
        if cleanup_errors:
            result["reason"] = f"{result['reason']}; cleanup: {'; '.join(cleanup_errors)}"
            result["verdict"] = "FAIL"
    latency = "n/a" if result["latency"] is None else f"{result['latency']:.3f}s"
    print(
        f"kill-resilience: {label}: latency={latency} host_exit={result['host_exit']} "
        f"renderer={result['renderer']} cause={result['cause']} verdict={result['verdict']}"
    )
    if result["verdict"] != "PASS":
        report_failure(label, result["reason"], stderr_path, stdout_path)
    return result


started = time.monotonic()
results = []
case_number = 0
for entry in entry_paths:
    for scenario in ("host-pid", "renderer-pid", "group"):
        case_number += 1
        results.append(run_case(entry, scenario, case_number))

failed = [result for result in results if result["verdict"] != "PASS"]
elapsed = time.monotonic() - started
print(f"kill-resilience: total={elapsed:.3f}s passed={len(results) - len(failed)} failed={len(failed)}")
if failed:
    print("kill-resilience: failed=" + ",".join(result["label"] for result in failed), file=sys.stderr)
    raise SystemExit(1)
PY
