# Menu accelerator audit

**Audit scope:** pinned GPUI under `references/zed`, with the static-menu protocol currently exposed by `COMMAND_SET_MENUS = 21`.

## Verdict

**No explicit accelerator seam exists in the pinned `MenuItem` data model.** `MenuItem::Action` carries only `name`, `action`, optional `os_action`, `checked`, and `disabled`; there is no keybinding, accelerator, shortcut, or key-equivalent field (`references/zed/crates/gpui/src/platform/app_menu.rs:86-103`). The constructors likewise accept only `(name, action)` or `(name, action, os_action)` (`.../app_menu.rs:125-149`). The owned form preserves the same fields (`.../app_menu.rs:260-277`), so ownership conversion cannot carry a caller-supplied accelerator either.

This is not an absence of all platform shortcut rendering. The upstream platform API passes the `Keymap` alongside menus (`references/zed/crates/gpui/src/platform.rs:230-248`), and the macOS renderer derives the first applicable keybinding for each action (`references/zed/crates/gpui_macos/src/platform.rs:308-347`). For a single keystroke it converts modifiers and calls `NSMenuItem::initWithTitle_action_keyEquivalent_`, then applies the modifier mask (`.../gpui_macos/src/platform.rs:362-399`); multi-stroke bindings and missing bindings receive an empty key equivalent (`.../gpui_macos/src/platform.rs:400-417`). Thus Zed's own macOS menu bar displays shortcuts by binding the same action in the app keymap, not by passing a shortcut to `MenuItem::action`. Zed's menu construction uses plain `MenuItem::action(...)` calls (`references/zed/crates/zed/src/zed/app_menus.rs:7-49`, `:61-163`), while the GPUI image-gallery example explicitly binds `cmd-q` separately before creating `MenuItem::action("Quit", Quit)` (`references/zed/crates/gpui/examples/image_gallery.rs:264-266`).

Other pinned platforms do not provide an equivalent renderer seam: Linux and Windows discard the `Keymap` parameter and only store owned menu data (`references/zed/crates/gpui_linux/src/linux/platform.rs:621-629`; `references/zed/crates/gpui_windows/src/platform.rs:742-748`). The client-side title-bar fallback also ignores any hypothetical accelerator and renders action/checked/disabled fields only (`references/zed/crates/title_bar/src/application_menu.rs:115-151`).

## Boundary for `COMMAND_SET_MENUS`

The local wire payload intentionally has compact menu tuples: separator `(0)`, action `(1, name)`, action-with-options `(1, name, (disabled, checked))`, and submenu `(2, menu)` (`crates/react-gpui/src/protocol/wire/command.rs:715-724`). The public decoded action has only `name`, `disabled`, and `checked` (`crates/react-gpui/src/protocol.rs:217-232`), and the renderer maps that action to `GpuiMenuItem::action(...).checked(...).disabled(...)` (`crates/react-gpui/src/renderer/commands.rs:97-117`). There is no accelerator payload field to pass through.

The protocol does have a separate `COMMAND_SET_KEYBINDINGS` path with `keystrokes` and `action_name` (`crates/react-gpui/src/protocol.rs:239-243`; `crates/react-gpui/src/protocol/wire/command.rs:152-170`). Therefore, on the macOS GPUI path, a menu action can acquire a displayed shortcut only indirectly when the caller supplies a matching keybinding for the generated menu action. This is not a caller-selected field on a menu item, and it is not portable: pinned Linux/Windows menu storage ignores the keymap, while the client-side fallback does not display keybindings.

## Future design note (not implemented)

If explicit static-menu accelerators are required, the protocol should extend action items with an optional accelerator string while retaining old tuple forms for compatibility, for example:

- `(1, name, (disabled, checked), accelerator)` when options and an accelerator are present;
- `(1, name, accelerator)` when only an accelerator is present; or, preferably, a versioned options record/tuple that avoids ambiguous untagged shapes.

Validation should reject empty accelerators, control characters, and an agreed maximum length (the existing menu text limit is 256 characters; `valid_menu_text` is at `crates/react-gpui/src/protocol/wire/command.rs:725-727`). The decoder must continue accepting the current two- and three-element forms, while the renderer must validate/parse the string into GPUI's platform keystroke representation rather than pass arbitrary display text to native APIs. The native contract must be specified per platform: macOS needs a key equivalent plus modifier mask; Linux/Windows need actual platform menu accelerator support (currently absent in the pinned implementations); and the non-native title-bar fallback must render the shortcut label itself if that UI is intended to show it.

That design is deliberately deferred: this audit records the upstream boundary only and makes no product-source changes.
