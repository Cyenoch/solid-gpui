#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
smoke_root="$(mktemp -d "${TMPDIR:-/tmp}/react-gpui-host-candidate-smoke.XXXXXX")"
trap 'rm -rf -- "$smoke_root"' EXIT

(
  cd "$repo_root"
  make host-release-check
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
bundle_name="react-gpui-host-${metadata}-${target}"
archive="$repo_root/dist/${bundle_name}.tar.gz"
user_entry="$repo_root/packages/react-gpui/examples/counter.tsx"
extract_dir="$smoke_root/extracted"
run_dir="$smoke_root/user-run"
mkdir -p "$extract_dir" "$run_dir"
tar -xzf "$archive" -C "$extract_dir"
binary="$extract_dir/$bundle_name/react-gpui-host"
[[ -f "$extract_dir/$bundle_name/THIRD-PARTY-NOTICES.md" ]] || { printf 'third-party notices inventory is missing\n' >&2; exit 1; }
[[ -f "$extract_dir/$bundle_name/SHA256SUMS" ]] || { printf 'candidate checksums are missing\n' >&2; exit 1; }
[[ -x "$binary" ]] || { printf 'candidate binary is not executable\n' >&2; exit 1; }
[[ -f "$user_entry" ]] || { printf 'user renderer entry is missing: %s\n' "$user_entry" >&2; exit 1; }

run_process_smoke() {
  local level="$1"
  local stderr_file="$smoke_root/${level}.stderr"
  local stdout_file="$smoke_root/${level}.stdout"
  local frame_file="$smoke_root/${level}.frames"
  set +e
  python3 - "$binary" "$user_entry" "$run_dir" "$level" "$frame_file" > "$stdout_file" 2> "$stderr_file" <<'PY'
import os
import signal
import subprocess
import sys
import time

binary, entry, cwd, level, frame_file = sys.argv[1:]
environment = os.environ.copy()
environment["REACT_GPUI_LOG"] = level
process = subprocess.Popen(
    [binary, "--runtime", "process", "sh", "-c", 'bun run "$1" | tee "$2"', "candidate-renderer", entry, frame_file],
    cwd=cwd,
    env=environment,
    start_new_session=True,
)
started = time.monotonic()
try:
    process.communicate(timeout=5.0)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGTERM)
    try:
        process.communicate(timeout=1.0)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()
    print(f"candidate run level={level} timed out after {time.monotonic() - started:.3f}s", file=sys.stderr)
    sys.exit(124)
print(f"candidate exited early level={level} code={process.returncode} after {time.monotonic() - started:.3f}s", file=sys.stderr)
sys.exit(process.returncode if process.returncode is not None else 1)
PY
  local status=$?
  set -e
  if [[ "$status" -ne 124 ]]; then
    printf 'candidate smoke level=%s expected timeout exit 124, got %s\n' "$level" "$status" >&2
    cat "$stderr_file" >&2 || true
    cat "$stdout_file" >&2 || true
    exit 1
  fi
  python3 - "$frame_file" <<'PY'
import struct
import sys

data = open(sys.argv[1], "rb").read()
if len(data) < 7:
    raise SystemExit("renderer emitted no complete frame")
length = struct.unpack("<I", data[:4])[0]
payload = data[4 : 4 + length]
if len(payload) != length or payload[:3] != bytes((0x97, 0x03, 0x01)):
    raise SystemExit("renderer first frame was not a Snapshot Commit Batch")
print(f"snapshot commit observed: {length} bytes")
PY
  printf '\n--- candidate smoke level=%s stderr ---\n' "$level"
  cat "$stderr_file"
  printf '\n--- candidate smoke level=%s stdout ---\n' "$level"
  cat "$stdout_file"
}

run_process_smoke error
run_process_smoke info
python3 - "$smoke_root/info.stderr" <<'PY'
import re
import sys
text = open(sys.argv[1], encoding="utf-8").read()
if re.search(r"react-gpui-host: starting mode=Process protocol=v3 entry=sh pid=\d+", text) is None:
    raise SystemExit("missing info startup diagnostic with mode/entry/pid")
if "react-gpui-host: renderer" in text and "fatal" in text:
    raise SystemExit("fatal renderer diagnostic appeared during startup smoke")
PY

version_output="$($binary --version)"
help_output="$($binary --help)"
[[ "$version_output" == "react-gpui-host $metadata protocol=v3" ]] || { printf 'unexpected candidate version: %s\n' "$version_output" >&2; exit 1; }
case "$help_output" in
  *"--version"*"--runtime process"*) ;;
  *) printf 'candidate help output is incomplete\n' >&2; exit 1 ;;
esac
printf '\n--- candidate --version ---\n%s\n' "$version_output"
printf '\n--- candidate --help (validated) ---\n'
printf '%s\n' "$help_output"
printf '\nhost candidate smoke passed: extracted binary, process timeout exit 124, info startup diagnostic, help/version\n'
