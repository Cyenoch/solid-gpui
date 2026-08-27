# TextInput maxLength and marked IME text

Status: resolved-no-gap

## Evidence

- `crates/react-gpui/src/renderer/input.rs:325-340` truncates replacement text
  at a UTF-16 boundary.
- Both ordinary and marked replacement paths subtract the replaced UTF-16
  range from the available budget and call that helper
  (`crates/react-gpui/src/renderer/input.rs:361-403`).
- Native initialization/reconciliation clamps controlled text and positions to
  the same UTF-16 max (`crates/react-gpui/src/renderer/input.rs:343-358,420-433`).

## Decision

No asymmetry was found in the bounded audit. Keep one truncation path and add a
focused test proving a marked replacement is truncated without splitting an
astral character and that its marked/selection ranges remain bounded.
