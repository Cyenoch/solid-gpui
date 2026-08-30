# Quartz has no active display for GUI host smoke tests

Status: needs-triage
Type: task

The current Quartz environment has no active display. Display-dependent GPUI
renderer/host paths were therefore not re-tested in this workflow. Non-GUI
coverage remains available through CLI parsing tests, extracted host
`--help`/`--version` checks, process/runtime tests, and package smoke checks.

Evidence: runtime-hardening's validation report and the host release-check
boundary in `scripts/host-release.sh`.

## Comments

A display-backed smoke test still requires an environment with an active
macOS display; this issue does not claim that the GUI path is broken.
