pub mod embedded;
#[cfg(feature = "quickjs")]
pub mod quickjs;
#[cfg(not(target_family = "wasm"))]
pub mod vite;

#[cfg(feature = "quickjs")]
pub mod reload;
