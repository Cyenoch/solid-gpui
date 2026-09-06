//! Iconify JSON resolve used by `build.rs` and integration tests.
//! Each crate uses a subset of this module, so unused items are expected.
#![allow(dead_code)]

mod error;
mod model;
pub mod resolve;
pub mod svg;
mod vendor;
pub use error::CodegenError;
pub use model::{EmbeddedIcon, PaintKind};
#[cfg(test)]
pub use vendor::ensure_preset_known;
pub use vendor::{
    embed_preset, ensure_allowlist_known, ensure_catalog, ensure_preset_enabled, load_allowlist,
    load_palette_flag, preset_feature_enabled,
};
