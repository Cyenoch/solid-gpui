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

TIMELINE_BUCKET_MS = 1_000
BYTE_HISTOGRAM_BUCKETS: tuple[tuple[str, int, int | None], ...] = (
    ("0-255", 0, 256),
    ("256-1023", 256, 1_024),
    ("1024-4095", 1_024, 4_096),
    ("4096-16383", 4_096, 16_384),
    ("16384+", 16_384, None),
)


def is_finite_number(value: object) -> bool:
    return (
        isinstance(value, (int, float))
        and not isinstance(value, bool)
        and math.isfinite(float(value))
    )


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
                if not is_finite_number(timestamp):
                    warnings.append(f"{path}:{line_number}: missing numeric t")
                    continue
                records.append((path_index, line_number, record))
    records.sort(key=lambda item: (float(item[2]["t"]), item[0], item[1]))
    return [record for _, _, record in records], warnings


def numeric_bytes(records: list[dict[str, Any]]) -> list[float]:
    return [float(record["bytes"]) for record in records if is_finite_number(record.get("bytes"))]


def display_number(value: float) -> int | float:
    return int(value) if value.is_integer() else value


def byte_histogram(records: list[dict[str, Any]]) -> dict[str, int]:
    histogram = {label: 0 for label, _, _ in BYTE_HISTOGRAM_BUCKETS}
    for value in numeric_bytes(records):
        for label, lower, upper in BYTE_HISTOGRAM_BUCKETS:
            if value >= lower and (upper is None or value < upper):
                histogram[label] += 1
                break
    return histogram


def per_second(total: float, duration_ms: float) -> int | float:
    if duration_ms <= 0:
        return 0
    return display_number(total * 1000 / duration_ms)


def frame_timeline(records: list[dict[str, Any]]) -> list[dict[str, int | float]]:
    buckets: dict[int, dict[str, float]] = {}
    for record in records:
        timestamp = float(record["t"])
        start_ms = math.floor(timestamp / TIMELINE_BUCKET_MS) * TIMELINE_BUCKET_MS
        bucket = buckets.setdefault(start_ms, {"frames": 0, "bytes": 0})
        bucket["frames"] += 1
        bytes_value = record.get("bytes")
        if is_finite_number(bytes_value):
            bucket["bytes"] += float(bytes_value)
    return [
        {
            "start_ms": start_ms,
            "end_ms": start_ms + TIMELINE_BUCKET_MS,
            "frames": int(bucket["frames"]),
            "frame_rate_hz": int(bucket["frames"]),
            "bytes": display_number(bucket["bytes"]),
            "bytes_per_second": per_second(bucket["bytes"], TIMELINE_BUCKET_MS),
        }
        for start_ms, bucket in sorted(buckets.items())
    ]


def malformed_record_count(warnings: list[str]) -> int:
    markers = ("invalid JSON (", "record is not an object", "missing numeric t")
    return sum(1 for warning in warnings if any(marker in warning for marker in markers))


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
    frame_duration = max(frame_times) - min(frame_times) if frame_times else 0
    frame_rate = len(frame_records) / (frame_duration / 1000) if frame_duration > 0 else 0
    frame_byte_total = sum(numeric_bytes(frame_records))
    patch_byte_total = sum(
        numeric_bytes([record for record in frame_records if record.get("kind") == "patch"])
    )
    intervals = [max(0.0, later - earlier) for earlier, later in zip(frame_times, frame_times[1:])]
    kinds = Counter(str(record.get("kind", "unknown")) for record in records)
    frames_by_kind = Counter(str(record.get("kind", "unknown")) for record in frame_records)
    event_types = Counter(
        str(record["event_type"])
        for record in records
        if record.get("kind") == "event" and is_finite_number(record.get("event_type"))
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
        "frame_duration_ms": frame_duration,
        "frame_rate_hz": frame_rate,
        "frames_by_kind": dict(sorted(frames_by_kind.items())),
        "kinds": dict(sorted(kinds.items())),
        "bytes": byte_summary(records),
        "frame_bytes": byte_summary(frame_records),
        "bytes_per_second": per_second(frame_byte_total, frame_duration),
        "patch_bytes_per_second": per_second(patch_byte_total, frame_duration),
        "byte_histogram": byte_histogram(frame_records),
        "bytes_by_kind": bytes_by_kind,
        "timeline": frame_timeline(frame_records),
        "event_types": dict(sorted(event_types.items(), key=lambda item: (float(item[0]), item[0]))),
        "malformed_frames": sum(1 for record in frame_records if record.get("kind") == "unknown"),
        "malformed_records": malformed_record_count(warnings),
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
