# Protocol additions

Add command codes 25/26, absolute path and bounded UTF-8 payload validation, a file-text result value tag, and command-result allowlists in both Rust and TypeScript. Keep the file payload cap at `MAX_FRAME_SIZE - 1024` so command frames have a single honest boundary.
