# Window position persistence boundary

Status: resolved
Type: research

## Question

Can the protocol restore a saved global window position at runtime or during `openSurface`?

## Answer

No. The pinned public GPUI Window API exposes global bounds reads but no public runtime position setter, and the existing `openSurface` payload exposes size/options only. The supported persistence recipe is to save `getWindowBounds()`, restore width/height through `openSurface({ width, height })`, and accept centered creation. Exact position restoration remains upstream-blocked and is not represented as a fake or fallback command.
