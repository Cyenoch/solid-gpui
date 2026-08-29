# VirtualList scroll offset

Implementation complete. `COMMAND_GET_SCROLL_OFFSET=34` reads the per-node GPUI ListState's logical pixel offset and returns value tag `10`; `COMMAND_SCROLL_TO_OFFSET=35` restores through the native pixel setter. Wire, renderer, host, TypeScript handles, focused tests, golden vectors, and documentation are updated. Native clamping and pre-layout no-op semantics remain authoritative.
