#!/usr/bin/env python3
"""Summarize one or more REACT_GPUI_TAP JSONL files."""

from __future__ import annotations

import argparse
import json
import math
import statistics
import sys
from collections import Counter, defaultdict, deque
from pathlib import Path
from typing import Any


def percentile(values: list[float], probability: float) -> float | None:
    if not values:
        return None
    if len(values) == 1:
        return values[0]
    position = (len(values) - 1) * probability
    lower = math.floor(position)
    upper = math.ceil(position)
    if lower == upper:
        return values[lower]
    weight = position - lower
    return values[lower] + (values[upper] - values[lower]) * weight


def load_records(paths: list[Path]) -> tuple[list[dict[str, Any]], list[str]]:
    records: list[tuple[int, int, dict[str, Any]]] = []
    warnings: list[str] = []
    for path_index, path in enumerate(paths):
        try:
            handle = path.open(encoding="utf-8")
        except OSError as error:
            warnings.append(f"{path}: {error}")
            continue
        with handle:
            for line_number, raw_line in enumerate(handle, 1):
                if not raw_line.strip():
                    continue
                try:
                    record = json.loads(raw_line)
                except json.JSONDecodeError as error:
                    warnings.append(f"{path}:{line_number}: invalid JSON ({error.msg})")
                    continue
                if not isinstance(record, dict):
                    warnings.append(f"{path}:{line_number}: record is not an object")
                    continue
                timestamp = record.get("t")
                if not isinstance(timestamp, (int, float)) or isinstance(timestamp, bool):
                    warnings.append(f"{path}:{line_number}: missing numeric t")
                    continue
                records.append((path_index, line_number, record))
    records.sort(key=lambda item: (float(item[2]["t"]), item[0], item[1]))
    return [record for _, _, record in records], warnings


def numeric_bytes(records: list[dict[str, Any]]) -> list[float]:
    return [float(record["bytes"]) for record in records if isinstance(record.get("bytes"), (int, float))]


def byte_summary(records: list[dict[str, Any]]) -> dict[str, int | float | None]:
    values = numeric_bytes(records)
    values.sort()
    if not values:
        return {"count": 0, "min": None, "median": None, "max": None, "total": 0}
    median = statistics.median(values)
    return {
        "count": len(values),
        "min": int(values[0]) if values[0].is_integer() else values[0],
        "median": int(median) if float(median).is_integer() else median,
        "max": int(values[-1]) if values[-1].is_integer() else values[-1],
        "total": int(sum(values)) if sum(values).is_integer() else sum(values),
    }


def command_success(records: list[dict[str, Any]]) -> dict[str, int | float | None]:
    pending: dict[int, deque[dict[str, Any]]] = defaultdict(deque)
    command_count = 0
    for record in records:
        if record.get("kind") != "command":
            continue
        request_id = record.get("request_id")
        if isinstance(request_id, int) and not isinstance(request_id, bool):
            pending[request_id].append(record)
            command_count += 1

    matched = successes = failures = 0
    for record in records:
        if record.get("kind") != "event" or record.get("event_type") != 6:
            continue
        request_id = record.get("request_id")
        if not isinstance(request_id, int) or isinstance(request_id, bool) or not pending[request_id]:
            continue
        pending[request_id].popleft()
        success = record.get("success")
        if not isinstance(success, bool):
            continue
        matched += 1
        if success:
            successes += 1
        else:
            failures += 1
    unmatched = command_count - matched
    rate = successes / matched if matched else None
    return {
        "commands": command_count,
        "matched": matched,
        "successes": successes,
        "failures": failures,
        "unmatched": unmatched,
        "rate": rate,
    }


def summarize(records: list[dict[str, Any]], file_count: int, warnings: list[str]) -> dict[str, Any]:
    timestamps = [float(record["t"]) for record in records]
    frame_records = [record for record in records if record.get("kind") != "tap_stopped"]
    frame_times = [float(record["t"]) for record in frame_records]
    duration = max(timestamps) - min(timestamps) if timestamps else 0
    frame_rate = len(frame_records) / (duration / 1000) if duration > 0 else 0
    intervals = [max(0.0, later - earlier) for earlier, later in zip(frame_times, frame_times[1:])]
    kinds = Counter(str(record.get("kind", "unknown")) for record in records)
    event_types = Counter(
        str(record["event_type"])
        for record in records
        if record.get("kind") == "event" and isinstance(record.get("event_type"), (int, float))
    )
    bytes_by_kind = {
        kind: byte_summary([record for record in records if record.get("kind") == kind])
        for kind in sorted(kinds)
    }
    return {
        "files": file_count,
        "records": len(records),
        "frames": len(frame_records),
        "duration_ms": duration,
        "frame_rate_hz": frame_rate,
        "kinds": dict(sorted(kinds.items())),
        "bytes": byte_summary(records),
        "bytes_by_kind": bytes_by_kind,
        "event_types": dict(sorted(event_types.items(), key=lambda item: (float(item[0]), item[0]))),
        "command_success": command_success(records),
        "frame_interval_ms": {
            "count": len(intervals),
            "p50": percentile(intervals, 0.50),
            "p95": percentile(intervals, 0.95),
        },
        "warnings": warnings,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="+", type=Path, help="tap JSONL files")
    args = parser.parse_args()
    records, warnings = load_records(args.paths)
    json.dump(summarize(records, len(args.paths), warnings), sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
