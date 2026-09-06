#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
duration_seconds="${SOAK_DURATION_SECONDS:-60}"
sample_seconds="${SOAK_SAMPLE_SECONDS:-5}"
smoke_root="$(mktemp -d "${TMPDIR:-/tmp}/solid-gpui-soak.XXXXXX")"
trap 'rm -rf -- "$smoke_root"' EXIT

binary="$repo_root/target/release/solid-gpui-host"
entry="$repo_root/fixtures/press-roundtrip.ts"
host_tap="$smoke_root/host.tap.jsonl"
renderer_tap="$smoke_root/renderer.tap.jsonl"
rss_samples="$smoke_root/rss.tsv"
stdout_file="$smoke_root/host.stdout"
stderr_file="$smoke_root/host.stderr"

cargo build -p solid-gpui --bin solid-gpui-host --release --locked

[[ -x "$binary" ]] || {
  printf 'soak-smoke: release build did not produce an executable host: %s\n' "$binary" >&2
  exit 1
}
[[ -f "$entry" ]] || { printf 'soak-smoke: renderer entry is missing: %s\n' "$entry" >&2; exit 1; }

set +e
python3 - "$binary" "$entry" "$duration_seconds" "$sample_seconds" "$host_tap" "$renderer_tap" "$rss_samples" >"$stdout_file" 2>"$stderr_file" <<'PY'
import json
import os
import signal
import subprocess
import sys
import time

binary, entry, duration_text, sample_text, host_tap, renderer_tap, rss_samples = sys.argv[1:]
duration = float(duration_text)
sample_interval = float(sample_text)
if duration <= 0 or sample_interval <= 0:
    raise SystemExit("soak-smoke: duration and sample interval must be positive")

environment = os.environ.copy()
environment["SOLID_GPUI_TAP"] = host_tap
renderer_command = 'SOLID_GPUI_TAP="$1" exec bun run "$2"'
process = subprocess.Popen(
    [binary, "--runtime", "process", "sh", "-c", renderer_command, "soak-renderer", renderer_tap, entry],
    cwd=os.path.dirname(entry),
    env=environment,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    start_new_session=True,
    text=True,
)
started = time.monotonic()
samples = []
try:
    while True:
        time.sleep(min(sample_interval, max(0.0, duration - (time.monotonic() - started))))
        elapsed = time.monotonic() - started
        if elapsed >= duration:
            break
        result = subprocess.run(
            ["ps", "-o", "rss=", "-p", str(process.pid)],
            check=False,
            capture_output=True,
            text=True,
        )
        value = result.stdout.strip()
        if value.isdigit():
            samples.append((elapsed, int(value)))
    os.killpg(process.pid, signal.SIGTERM)
    try:
        stdout, stderr = process.communicate(timeout=10)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        stdout, stderr = process.communicate()
        raise SystemExit("soak-smoke: host did not exit within 10 seconds after SIGTERM")
except BaseException:
    if process.poll() is None:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()
    raise

with open(rss_samples, "w", encoding="utf-8") as output:
    for elapsed, rss in samples:
        output.write(f"{elapsed:.3f}\t{rss}\n")

print(json.dumps({
    "duration_seconds": round(time.monotonic() - started, 3),
    "requested_seconds": duration,
    "exit_code": process.returncode,
    "rss_samples": len(samples),
    "stdout": stdout,
    "stderr": stderr,
}, indent=2))
if process.returncode not in (0, -signal.SIGTERM, 128 + signal.SIGTERM):
    raise SystemExit(f"soak-smoke: host exited with code {process.returncode}")
PY
driver_status=$?
set -e
if [[ "$driver_status" -ne 0 ]]; then
  printf '%s\n' '--- host stderr ---' >&2
  cat "$stderr_file" >&2 || true
  printf '%s\n' '--- host stdout ---' >&2
  cat "$stdout_file" >&2 || true
  exit "$driver_status"
fi

python3 - "$rss_samples" "$host_tap" "$renderer_tap" "$duration_seconds" "${SOAK_RSS_GROWTH_PERCENT:-30}" "${SOAK_RSS_GROWTH_KB:-262144}" <<'PY'
import json
import math
import pathlib
import sys

rss_path, host_tap, renderer_tap, duration_text, percent_limit_text, absolute_limit_text = sys.argv[1:]
duration = float(duration_text)
percent_limit = float(percent_limit_text)
absolute_limit_kb = int(absolute_limit_text)
samples = []
for line in pathlib.Path(rss_path).read_text().splitlines():
    elapsed, rss = line.split("\t")
    samples.append((float(elapsed), int(rss)))
if len(samples) < 2:
    raise SystemExit("soak-smoke: fewer than two host RSS samples")
first = samples[0][1]
last = samples[-1][1]
peak = max(rss for _, rss in samples)
delta = last - first
percent = (delta / first * 100) if first else math.inf
slope = delta / max(duration / 60, 1 / 60)
print(f"host RSS: samples={len(samples)} first={first}KiB last={last}KiB peak={peak}KiB delta={delta}KiB percent={percent:.2f}% slope={slope:.2f}KiB/min")
print(f"RSS leak-smoke limits: tail-first <= {percent_limit:.0f}% and <= {absolute_limit_kb}KiB")
if percent > percent_limit and delta > absolute_limit_kb:
    raise SystemExit("soak-smoke: RSS growth exceeded both leak-smoke thresholds")

def records(path):
    result = []
    for line in pathlib.Path(path).read_text().splitlines():
        record = json.loads(line)
        if record.get("kind") == "tap_stopped":
            raise SystemExit(f"soak-smoke: tap reached capacity: {path}")
        result.append(record)
    return result

host = records(host_tap)
renderer = records(renderer_tap)
def assert_monotonic(name, values):
    if any(later <= earlier for earlier, later in zip(values, values[1:])):
        raise SystemExit(f"soak-smoke: non-monotonic tap sequence: {name}")

for name, values in (("host", host), ("renderer", renderer)):
    assert_monotonic(name, [int(record["seq"]) for record in values])
renderer_out = [r for r in renderer if r.get("dir") == "out" and r.get("peer") == "host"]
host_in = [r for r in host if r.get("dir") == "in" and r.get("peer") == "renderer"]
host_out = [r for r in host if r.get("dir") == "out" and r.get("peer") == "renderer"]
renderer_in = [r for r in renderer if r.get("dir") == "in" and r.get("peer") == "host"]
renderer_commands = [r for r in renderer_out if r.get("kind") == "command"]
host_command_results = [r for r in host_out if r.get("kind") == "event" and r.get("event_type") == 6]
print(f"command frames: renderer_out_commands={len(renderer_commands)} host_command_results={len(host_command_results)}")
print(f"tap frames: renderer_out={len(renderer_out)} host_in={len(host_in)} host_out={len(host_out)} renderer_in={len(renderer_in)}")
if len(renderer_out) != len(host_in) or len(host_out) != len(renderer_in):
    raise SystemExit("soak-smoke: tap frame counts do not match across the process boundary")
print(f"soak-smoke passed: requested={duration:.0f}s, rss_samples={len(samples)}, clean_exit=true, frame_counts_match=true")
PY

printf '%s\n' '--- host stderr ---'
cat "$stderr_file"
printf '%s\n' '--- host stdout ---'
cat "$stdout_file"
