#![doc = r#"`gpui-iconify` embeds an allowlisted Iconify subset and paints it through GPUI.

Design consensus: [docs/design.md](../../../docs/design.md)
Architecture: [docs/architecture.md](../../../docs/architecture.md)
Crate boundaries: [docs/crates.md](../../../docs/crates.md)
Related decision: [ADR-0014](../../../docs/adr/0014-gpui-iconify-ui-library.md)

`icon_id!("prefix:name")` and `icon!("prefix:name")` accept string literals only.
A name that is not compiled in fails at compile time. Size and color are runtime
style. JSON is build input, never a runtime resource.

```rust
use gpui_iconify::{icon, icon_id, IconId, IconSize};

let _id = icon_id!("lucide:play");
let _view = icon!("lucide:play").with_size(IconSize::MEDIUM);
assert_eq!(IconId::from_name("lucide:play"), Some(IconId::LucidePlay));
```

```compile_fail
use gpui_iconify::icon_id;
let _ = icon_id!("lucide:not-a-real-icon");
```
"#]

mod assets;
mod icon_id;
mod icon_view;
mod palette;
mod size;

pub use assets::IconAssets;
pub use icon_id::{IconId, IconKind};
pub use icon_view::IconView;
pub use size::IconSize;

include!(concat!(env!("OUT_DIR"), "/icon_macro.rs"));
