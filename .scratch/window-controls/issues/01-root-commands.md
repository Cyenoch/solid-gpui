# Root window commands

Status: resolved
Type: task

Implement root-only window commands 30–33 with MessagePack validation, native GPUI dispatch, TypeScript Root methods, and cross-language golden vectors.

## Answer

Commands 30 (`minimizeWindow`), 31 (`getWindowBounds`), 32 (`getWindowState`), and 33 (`activateWindow`) are implemented. Bounds use value tag 8 and state uses value tag 9. All commands require `nodeId=1` and a null payload. Headless coverage records the pinned TestWindow limitation: minimize is unimplemented and active-state reporting is false; display-backed verification is required for visible miniaturization/activation.
