# ADR-0007: Keep Image sources host-resolved

- **Status:** Accepted (revised 2026-09-08)
- **Date:** 2026-08-25

Image sources cross the runtime boundary as bounded strings. The host resolves
local paths, native `file:` URLs, HTTP(S) URLs, and inline `data:image/...` URLs;
GPUI owns asynchronous loading, decoding, and caching. This replaces the original
local-path-only decision: remote images are a supported host capability, and the
standard native host installs an HTTP client before opening windows.

Relative paths resolve from the host working directory. URL images need no
JavaScript fetch or temporary file. Inline images share the asset cache and are
subject to the 1 MiB UTF-8 source limit and total commit-frame budget. See
[Images](../native-composition.md#images) for usage and platform requirements.

A failed primary image displays `fallbackSource` when supplied, otherwise blank
output. A pending request does not display the fallback. Failure remains local
to the image and does not discard the surface or emit a JavaScript error event.
An implicit JavaScript-runtime asset resolver remains excluded because resolution
must be consistent across external and embedded runtimes.
