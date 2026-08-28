# Text file commands

## Goal

Give a root-only React GPUI surface a bounded, asynchronous text-file persistence
interface. A renderer app can read a picked path and write UTF-8 content without
reimplementing native filesystem access, while every malformed request and OS
failure remains observable as a rejected Promise.

## Decisions

- `COMMAND_READ_TEXT_FILE = 25` accepts one absolute path string. `COMMAND_WRITE_TEXT_FILE = 26` accepts `[path, content]`. Both require `nodeId=1` and preserve the existing Command tuple/header.
- Paths are non-empty, absolute on the host platform, no control characters, and at most 1024 UTF-8 bytes. Paths are accepted by ordinary host filesystem semantics: symlinks are neither prohibited nor resolved by the protocol.
- Writes and reads use one frame-safe text cap, `MAX_FILE_*_BYTES = MAX_FRAME_SIZE - 1024` (16 MiB minus a 1 KiB envelope allowance). The allowance covers the complete MessagePack command/result envelope and avoids a nominal 16 MiB text payload later failing frame encoding. This is deliberately a single honest boundary rather than a nominal cap with an unexpected transport rejection.
- CommandResult value tag `6` is `FileText`, separate from tag `4` clipboard/path text, so the clipboard's 1 MiB invariant remains unchanged. Write acknowledgements use the existing finite numeric tag `1` and return UTF-8 bytes written.

## Surface

```ts
Root.readTextFile(path: string): Promise<string>
Root.writeTextFile(path: string, content: string): Promise<number>
```

Host failures reject with the existing command failure `Error` surface. Local
argument failures reject before a frame is submitted.

## Verification contract

- Rust wire tests cover both command shapes, value tag 6, and command allowlists.
- Host headless tests write/read/delete a real temporary file, cover missing and directory errors, and prove oversize/relative requests are rejected.
- TypeScript tests cover command encoding, typed value decoding, Promise resolution/rejection, path/content validation, and byte acknowledgements.
- Cross-language golden vectors cover both new command directions and read/write CommandResult values.
- `examples/notes.tsx` demonstrates Open/Save, multiline editing, dirty state, and asynchronous close confirmation.
