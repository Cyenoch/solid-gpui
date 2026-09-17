//! Application-owned embedded host.
//!
//! This is the library half of the custom-host fixture: the packager generates
//! the binary crate that owns the final link, depends on this package, and
//! `include!`s `embedded-main.rs` from it. Nothing here is SDK-owned, which is
//! the point — an application that needs its own Cargo features, patches and
//! profiles does not fork the packager.

use solid_gpui::runtime::embedded::{EmbeddedBunAdapter, EmbeddedResult};
use std::sync::Arc;

/// The host's own view of the application session.
pub struct Session {
    runtime: Arc<EmbeddedBunAdapter>,
}

impl Session {
    /// Starts the packaged application from its graph identity.
    pub fn start(entry: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            runtime: EmbeddedBunAdapter::start_packaged(entry)?,
        })
    }

    /// The runtime every application surface shares.
    pub fn runtime(&self) -> Arc<EmbeddedBunAdapter> {
        Arc::clone(&self.runtime)
    }

    /// The result the application declared, once its session has settled.
    pub fn declared_result(&self) -> Option<EmbeddedResult> {
        self.runtime.result()
    }
}
