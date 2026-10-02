//! Explicit application-owned signed updates. No host enables this module by default.

use crate::native::{CommandDefinition, ModuleDefinition, NativeCallContext};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use ts_rs::TS;

/// Native configuration is trusted application code, never a JavaScript request.
#[derive(Clone)]
pub struct UpdaterConfig {
    pub app_id: String,
    pub channel: String,
    pub feed_url: String,
    pub public_key: [u8; 32],
    pub current_sequence: u32,
    pub install_path: PathBuf,
    pub executable: String,
    pub max_archive_bytes: u32,
    pub max_unpacked_bytes: u32,
    pub max_entries: u32,
    pub timeout: Duration,
    /// Allows literal loopback HTTP only. Intended for temporary local fixtures.
    pub allow_loopback_http: bool,
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateOffer {
    pub token: String,
    pub version: String,
    pub sequence: u32,
    pub archive_bytes: u32,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateRequest {
    pub token: String,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateStatus {
    pub enabled: bool,
    pub platform: String,
    pub pending_restart: bool,
    pub rollback_available: bool,
    pub relaunch_policy: String,
}

mod acquisition;
mod installation;
#[cfg(all(test, target_os = "macos"))]
mod recovery_tests;
#[cfg(all(test, target_os = "macos"))]
mod test_io;
use acquisition::{Manifest, SignedFeed};

struct State {
    offer: Option<(UpdateOffer, Manifest, SignedFeed)>,
    sequence: u32,
    disk_changed: bool,
}

pub struct SignedUpdater {
    config: UpdaterConfig,
    client: Arc<dyn gpui::http_client::HttpClient>,
    state: Mutex<State>,
    _lock: std::fs::File,
}

impl SignedUpdater {
    pub fn new(
        config: UpdaterConfig,
        client: Arc<dyn gpui::http_client::HttpClient>,
    ) -> Result<Self, String> {
        if !cfg!(target_os = "macos") {
            return Err("signed update installation is supported only on macOS".into());
        }
        acquisition::validate_config(&config)?;
        let lock = installation::lock(&config)?;
        installation::recover(&config)?;
        let sequence = config.current_sequence;
        Ok(Self {
            config,
            client,
            state: Mutex::new(State {
                offer: None,
                sequence,
                disk_changed: false,
            }),
            _lock: lock,
        })
    }

    pub fn status(&self) -> Result<UpdateStatus, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "signed updater state is poisoned")?;
        self.status_locked(&state)
    }

    fn status_locked(&self, state: &State) -> Result<UpdateStatus, String> {
        let pending = installation::pending(&self.config)?;
        Ok(UpdateStatus {
            enabled: true,
            platform: acquisition::platform(),
            pending_restart: pending || state.disk_changed,
            rollback_available: pending,
            relaunch_policy: "application-managed".into(),
        })
    }

    pub fn check(&self, context: NativeCallContext) -> Result<Option<UpdateOffer>, String> {
        let mut state = self
            .state
            .try_lock()
            .map_err(|_| "signed updater is busy")?;
        state.offer = None;
        if installation::pending(&self.config)? {
            return Err("confirm or roll back the pending update first".into());
        }
        let bytes = acquisition::download(
            &self.config,
            &self.client,
            &self.config.feed_url,
            32768,
            &context,
        )?;
        let signed: SignedFeed =
            serde_json::from_slice(&bytes).map_err(|e| format!("invalid signed feed: {e}"))?;
        let manifest = signed.verify(&self.config)?;
        if manifest.sequence <= state.sequence {
            return Ok(None);
        }
        let token = acquisition::hex(&sha2::Sha256::digest(signed.payload.as_bytes()));
        let offer = UpdateOffer {
            token,
            version: manifest.version.clone(),
            sequence: manifest.sequence,
            archive_bytes: manifest.archive_bytes,
        };
        context.check_cancelled()?;
        state.offer = Some((offer.clone(), manifest, signed));
        Ok(Some(offer))
    }

    pub fn install(
        &self,
        request: UpdateRequest,
        context: NativeCallContext,
    ) -> Result<UpdateStatus, String> {
        let mut state = self
            .state
            .try_lock()
            .map_err(|_| "signed updater is busy")?;
        let (offer, manifest, signed) = state.offer.as_ref().ok_or("check for an update first")?;
        if request.token != offer.token {
            return Err("stale update token".into());
        }
        if installation::pending(&self.config)? {
            return Err("an update is already pending confirmation".into());
        }
        let bytes = acquisition::download(
            &self.config,
            &self.client,
            &manifest.url,
            manifest.archive_bytes,
            &context,
        )?;
        if bytes.len() != manifest.archive_bytes as usize
            || acquisition::hex(&sha2::Sha256::digest(&bytes)) != manifest.sha256
        {
            return Err("signed artifact size or SHA-256 mismatch".into());
        }
        // The signature binds identity, monotonic sequence, archive format, URL,
        // size and artifact hash. Verify again immediately before staging.
        signed.verify(&self.config)?;
        context.check_cancelled()?;
        installation::install(&self.config, manifest, signed, &bytes, &context)?;
        state.disk_changed = true;
        state.offer = None;
        self.status_locked(&state)
    }

    pub fn rollback(&self, context: NativeCallContext) -> Result<UpdateStatus, String> {
        let mut state = self
            .state
            .try_lock()
            .map_err(|_| "signed updater is busy")?;
        context.check_cancelled()?;
        if installation::pending(&self.config)? {
            state.disk_changed = true;
        }
        installation::rollback(&self.config)?;
        state.offer = None;
        self.status_locked(&state)
    }

    /// Call only after the application has restarted and completed its health check.
    /// Until then the previous bundle stays available, even across host lifetimes.
    pub fn confirm(&self, context: NativeCallContext) -> Result<UpdateStatus, String> {
        let mut state = self
            .state
            .try_lock()
            .map_err(|_| "signed updater is busy")?;
        context.check_cancelled()?;
        state.sequence = installation::confirm(&self.config)?;
        state.disk_changed = false;
        self.status_locked(&state)
    }
}

use sha2::Digest;

/// The registered instance exports its exact DTOs through the normal native generator.
/// Passing `None` exports a disabled service, with no feed or installation authority.
pub fn native_module(service: Option<Arc<SignedUpdater>>) -> ModuleDefinition {
    let owner = service.clone();
    let status = CommandDefinition::blocking("updateStatus", move |_: (), _| match &owner {
        Some(owner) => owner.status(),
        None => Ok(UpdateStatus {
            enabled: false,
            platform: acquisition::platform(),
            pending_restart: false,
            rollback_available: false,
            relaunch_policy: "application-managed".into(),
        }),
    });
    let check = service.clone();
    let install = service.clone();
    let rollback = service.clone();
    ModuleDefinition::new(
        "signed-updater",
        "1.0.0",
        vec![],
        vec![
            status,
            CommandDefinition::blocking("checkUpdate", move |_: (), context| {
                check
                    .as_ref()
                    .ok_or("signed updater is disabled")?
                    .check(context)
            }),
            CommandDefinition::blocking("installUpdate", move |request: UpdateRequest, context| {
                install
                    .as_ref()
                    .ok_or("signed updater is disabled")?
                    .install(request, context)
            }),
            CommandDefinition::blocking("rollbackUpdate", move |_: (), context| {
                rollback
                    .as_ref()
                    .ok_or("signed updater is disabled")?
                    .rollback(context)
            }),
            CommandDefinition::blocking("confirmUpdate", move |_: (), context| {
                service
                    .as_ref()
                    .ok_or("signed updater is disabled")?
                    .confirm(context)
            }),
        ],
    )
    .with_implementation(concat!(
        include_str!("mod.rs"),
        include_str!("acquisition.rs"),
        include_str!("installation.rs")
    ))
}
