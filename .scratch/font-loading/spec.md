# Runtime custom-font loading

## Goal

Let a React GPUI application register a bundled TrueType/OpenType font before
using its family in `fontFamily`. The host reads the bounded absolute path,
registers the bytes with the pinned GPUI `TextSystem`, and returns the family
name parsed from the font metadata.

## Evidence

The pinned GPUI revision is `6805d952f9f3d702f760aa11b1547df8a625fa16`.

- `references/zed/crates/gpui/src/text_system.rs:101-104` exposes
  `TextSystem::add_fonts(Vec<Cow<'static, [u8]>>) -> Result<()>`, delegating to
  `PlatformTextSystem::add_fonts` (`platform.rs:1071-1074`).
- macOS `gpui_platform::current_platform` constructs `MacPlatform`, and with
  the enabled `font-kit` feature `MacPlatform::new` constructs
  `MacTextSystem` (`gpui_platform/src/gpui_platform.rs:56-61`,
  `gpui_macos/src/platform.rs:196-211`). The macOS system is CoreText-backed
  (`gpui_macos/src/text_system.rs:56-57`) and stores runtime fonts in a
  `font_kit::sources::mem::MemSource`.
- `MacTextSystemState::add_fonts` converts bytes to memory handles and calls
  `MemSource::add_fonts` (`gpui_macos/src/text_system.rs:255-273`); family
  lookup prefers that memory source (`:276-290`).
- Linux/FreeBSD's pinned WGPU backend uses `CosmicTextSystem`; its
  `add_fonts` calls `fontdb::Database::load_font_source(Source::Binary(...))`
  (`gpui_wgpu/src/cosmic_text_system.rs:95-98,212-217`).
- Zed preloads embedded fonts before normal rendering: `zed/src/main.rs:733-735`
  calls `load_embedded_fonts`, which reads assets and invokes
  `cx.text_system().add_fonts` (`:1834-1855`). The reusable asset helper does
  the same (`crates/assets/src/assets.rs:40-56`).
- GPUI caches both successful and failed `font_id` resolutions in its private
  `font_ids_by_font` map (`gpui/src/text_system.rs:52-54,115-127`). There is no
  cache-invalidation seam. Therefore this feature's contract is registration
  before the family's first layout/use on a given `TextSystem`; a later or
  duplicate registration is accepted by the platform but an already-cached
  failed lookup continues to resolve through GPUI's fallback stack rather than
  erroring.

## Decisions

- Add root-only `COMMAND_LOAD_FONT = 29`.
- `Root.loadFont(path: string): Promise<string>` sends the existing command
  tuple with the absolute path as its string payload and resolves with the
  existing `CommandValue::Text`/tag-4 family result. No alias is accepted: the
  font's metadata family is the only honest name usable by `fontFamily`.
- Reuse file-command path validation and the frame-safe file cap
  (`MAX_FILE_READ_BYTES = MAX_FRAME_SIZE - 1024`) for the font bytes.
- Read bytes on the background executor, then register on the GPUI foreground
  context and parse/return the family name. A successful registration causes a
  redraw so later frames can shape the family.
- Parse the first face's OpenType `name` table using `ttf-parser`; prefer
  typographic family name (`nameID 16`) and fall back to family name (`nameID
  1`). Reject malformed/unnamed fonts. TTF/OTF are required; TTC/OTC are
  accepted where the pinned backend can load them but the returned family is
  the first face's family. WOFF/WOFF2 are not supported by the pinned GPUI
  text backends and are rejected as malformed/unsupported.
- No preload option is added to `RootOptions`: callers can invoke `loadFont`
  before their first render, matching Zed's startup preload pattern.

## Surface

```ts
Root.loadFont(path: string): Promise<string>
```

Local argument errors reject before framing. Host read, parse, and registration
errors are unsuccessful command results and reject the Promise. Repeated calls
are allowed; callers should retain the returned family name. Calling after the
family has already been laid out does not retroactively invalidate GPUI's
cached fallback choice.

## Verification contract

- Rust wire tests cover command 29's root-only path shape, malformed payloads,
  and command-result allowlisting.
- Host headless coverage registers a committed deterministic font fixture,
  checks the returned family, and proves a subsequent custom-family glyph can
  resolve/rasterize without fallback.
- TypeScript tests cover command framing, result-family decoding, and path/size
  validation.
- Golden vectors include command 29 and its tag-4 family result in both
  directions.
