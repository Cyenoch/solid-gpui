# Headless window-control limitations

Status: resolved
Type: task

## Answer

The pinned TestWindow exposes readable bounds and reports `is_maximized() == false` and `is_active() == false`. Its platform `minimize()` is `unimplemented!()`. Therefore host tests assert bounds/state and activation routing, while minimize coverage stops before invoking the headless native method; visible minimize and activation require a display-backed host.
