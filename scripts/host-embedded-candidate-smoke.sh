#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
smoke_root="$(mktemp -d "${TMPDIR:-/tmp}/react-gpui-host-embedded-smoke.XXXXXX")"
trap 'rm -rf -- "$smoke_root"' EXIT

(
  cd "$repo_root"
  cargo build -p react-gpui-host --features embedded-bun --release --locked --target "$(rustc -vV | python3 -c 'import sys; print(next(line.split(": ", 1)[1] for line in sys.stdin if line.startswith("host: ")))')"
)
metadata="$(
  cd "$repo_root"
  cargo metadata --format-version 1 --no-deps |
    python3 -c 'import json, sys; package = next(p for p in json.load(sys.stdin)["packages"] if p["name"] == "react-gpui-host"); print(package["version"])'
)"
target="$(
  cd "$repo_root"
  rustc -vV | python3 -c 'import sys; print(next(line.split(": ", 1)[1] for line in sys.stdin if line.startswith("host: ")))'
)"
binary="$repo_root/target/$target/release/react-gpui-host"
user_entry="$repo_root/packages/react-gpui/examples/counter.tsx"
run_dir="$smoke_root/user-run"
stderr_file="$smoke_root/embedded.stderr"
stdout_file="$smoke_root/embedded.stdout"
mkdir -p "$run_dir"
[[ -x "$binary" ]] || { printf 'embedded candidate binary is not executable\n' >&2; exit 1; }
[[ -f "$user_entry" ]] || { printf 'user renderer entry is missing: %s\n' "$user_entry" >&2; exit 1; }

set +e
python3 - "$binary" "$user_entry" "$run_dir" "$stderr_file" > "$stdout_file" 2> "$stderr_file" <<'PY'
import os
import re
import signal
import subprocess
import sys
import time

binary, entry, cwd, stderr_path = sys.argv[1:]
environment = os.environ.copy()
environment["REACT_GPUI_LOG"] = "info"
child_stderr_path = stderr_path + ".child"
stderr_sink = open(child_stderr_path, "w", encoding="utf-8")
process = subprocess.Popen(
    [binary, "--runtime", "embedded", entry, "--smoke-press"],
    cwd=cwd,
    env=environment,
    stdout=subprocess.PIPE,
    stderr=stderr_sink,
    start_new_session=True,
)
started = time.monotonic()
commit_deadline = started + 10.0

def diagnostics() -> str:
    stderr_sink.flush()
    try:
        return open(child_stderr_path, encoding="utf-8").read()
    except FileNotFoundError:
        return ""
def finish(message: str) -> None:
    text = diagnostics()
    stderr_sink.close()
    print(text, end="", file=sys.stderr)
    print(message, file=sys.stderr)

try:
    while True:
        if process.poll() is not None:
            finish(
                f"embedded candidate exited early code={process.returncode} after {time.monotonic() - started:.3f}s"
            )
            sys.exit(process.returncode if process.returncode is not None else 1)
        text = diagnostics()
        match = re.search(r"embedded smoke press sent=true, commits=(\d+), status=", text)
        if match is not None and int(match.group(1)) >= 1:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.communicate(timeout=1.0)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.communicate()
            finish(f"embedded candidate timed out after {time.monotonic() - started:.3f}s")
            sys.exit(124)
        if time.monotonic() >= commit_deadline:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.communicate(timeout=1.0)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.communicate()
            finish("embedded candidate timed out waiting for a committed Snapshot")
            sys.exit(124)
        time.sleep(0.02)
finally:
    if stderr_sink is not None and not stderr_sink.closed:
        stderr_sink.close()
PY
status=$?
set -e
if [[ "$status" -ne 124 ]]; then
  printf 'embedded candidate expected timeout exit 124, got %s\n' "$status" >&2
  cat "$stderr_file" >&2 || true
  cat "$stdout_file" >&2 || true
  exit 1
fi

python3 - "$stderr_file" <<'PY'
import re
import sys
text = open(sys.argv[1], encoding="utf-8").read()
if re.search(r"react-gpui-host: starting mode=Embedded protocol=v3 entry=.* pid=\d+", text) is None:
    print(text, file=sys.stderr)
    raise SystemExit("missing embedded info startup diagnostic with entry/pid")
match = re.search(r"embedded smoke press sent=true, commits=(\d+), status=", text)
if match is None or int(match.group(1)) < 1:
    print(text, file=sys.stderr)
    raise SystemExit("embedded smoke did not report a committed Snapshot")
PY

version_output="$($binary --version)"
help_output="$($binary --help)"
[[ "$version_output" == "react-gpui-host $metadata protocol=v3" ]] || { printf 'unexpected embedded candidate version: %s\n' "$version_output" >&2; exit 1; }
case "$help_output" in
  *"--runtime embedded"*"--version"*) ;;
  *) printf 'embedded candidate help output is incomplete\n' >&2; exit 1 ;;
esac
printf '\n--- embedded candidate stderr ---\n'
cat "$stderr_file"
printf '\n--- embedded candidate stdout ---\n'
cat "$stdout_file"
printf '\n--- embedded candidate --version ---\n%s\n' "$version_output"
printf '\n--- embedded candidate --help (validated) ---\n%s\n' "$help_output"
printf '\nembedded candidate smoke passed: release binary, embedded Snapshot commit count, info diagnostic, timeout exit 124, help/version\n'
