#!/usr/bin/env python3
"""Focused regression test for protocol-tap-report.py's aggregate output."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "protocol-tap-report.py"
FIXTURE = ROOT / "scripts" / "fixtures" / "protocol-tap-report.jsonl"


class ProtocolTapReportTest(unittest.TestCase):
    def test_aggregates_frames_bytes_events_timeline_and_malformed_input(self) -> None:
        result = subprocess.run(
            [sys.executable, str(SCRIPT), str(FIXTURE)],
            check=True,
            capture_output=True,
            text=True,
        )
        report = json.loads(result.stdout)

        self.assertEqual(report["frames"], 6)
        self.assertEqual(report["frame_bytes"]["total"], 520)
        self.assertEqual(report["frame_duration_ms"], 2250.0)
        self.assertAlmostEqual(report["bytes_per_second"], 520_000 / 2250)
        self.assertAlmostEqual(report["patch_bytes_per_second"], 300_000 / 2250)
        self.assertEqual(
            report["byte_histogram"],
            {"0-255": 5, "256-1023": 1, "1024-4095": 0, "4096-16383": 0, "16384+": 0},
        )
        self.assertEqual(report["event_types"], {"6": 1, "12": 1})
        self.assertEqual(report["malformed_frames"], 1)
        self.assertEqual(report["malformed_records"], 1)
        self.assertEqual(
            report["timeline"],
            [
                {
                    "start_ms": 0,
                    "end_ms": 1000,
                    "frames": 3,
                    "frame_rate_hz": 3,
                    "bytes": 420,
                    "bytes_per_second": 420,
                },
                {
                    "start_ms": 1000,
                    "end_ms": 2000,
                    "frames": 2,
                    "frame_rate_hz": 2,
                    "bytes": 90,
                    "bytes_per_second": 90,
                },
                {
                    "start_ms": 2000,
                    "end_ms": 3000,
                    "frames": 1,
                    "frame_rate_hz": 1,
                    "bytes": 10,
                    "bytes_per_second": 10,
                },
            ],
        )
        self.assertEqual(len(report["warnings"]), 1)


if __name__ == "__main__":
    unittest.main()
