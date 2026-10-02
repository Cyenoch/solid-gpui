use super::*;
use ed25519_dalek::{Signature, VerifyingKey};
use futures::AsyncReadExt;
use gpui::http_client::{AsyncBody, HttpRequestExt, RedirectPolicy, Request, Url};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SignedFeed {
    pub payload: String,
    signature: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Manifest {
    format: String,
    app_id: String,
    channel: String,
    platform: String,
    pub sequence: u32,
    pub version: String,
    archive_format: String,
    pub url: String,
    pub archive_bytes: u32,
    pub sha256: String,
    pub bundle: String,
    pub executable: String,
}

pub(super) fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}
pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(text: &str) -> Result<Vec<u8>, String> {
    if !text.len().is_multiple_of(2)
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid lowercase hex encoding".into());
    }
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).map_err(|e| e.to_string()))
        .collect()
}

impl SignedFeed {
    pub fn verify(&self, config: &UpdaterConfig) -> Result<Manifest, String> {
        if self.payload.len() > 24576 || self.signature.len() != 128 {
            return Err("signed feed exceeds limits".into());
        }
        let payload = unhex(&self.payload)?;
        let signature = Signature::from_slice(&unhex(&self.signature)?)
            .map_err(|_| "invalid Ed25519 signature")?;
        VerifyingKey::from_bytes(&config.public_key)
            .map_err(|_| "invalid public key")?
            .verify_strict(&payload, &signature)
            .map_err(|_| "update signature verification failed")?;
        let manifest: Manifest = serde_json::from_slice(&payload)
            .map_err(|e| format!("invalid release manifest: {e}"))?;
        if manifest.format != "solid-gpui-update-v1"
            || manifest.archive_format != "app-tar-v1"
            || manifest.app_id != config.app_id
            || manifest.channel != config.channel
            || manifest.platform != platform()
            || manifest.bundle
                != config
                    .install_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or("invalid bundle name")?
            || manifest.executable != config.executable
        {
            return Err("release identity or archive format mismatch".into());
        }
        if manifest.version.is_empty()
            || manifest.version.len() > 128
            || manifest.archive_bytes == 0
            || manifest.archive_bytes > config.max_archive_bytes
            || manifest.sha256.len() != 64
        {
            return Err("release manifest exceeds configured limits".into());
        }
        unhex(&manifest.sha256)?;
        validate_url(config, &manifest.url)?;
        if Url::parse(&manifest.url).unwrap().origin()
            != Url::parse(&config.feed_url).unwrap().origin()
        {
            return Err("artifact URL must share the trusted feed origin".into());
        }
        Ok(manifest)
    }
}

pub(super) fn validate_url(config: &UpdaterConfig, text: &str) -> Result<(), String> {
    if text.len() > 2048 {
        return Err("update URL exceeds limits".into());
    }
    let url = Url::parse(text).map_err(|_| "invalid update URL")?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.host().is_none()
    {
        return Err("update URL must not contain credentials or fragments".into());
    }
    let loopback = matches!(url.host(), Some(gpui::http_client::Host::Ipv4(ip)) if ip.is_loopback())
        || matches!(url.host(), Some(gpui::http_client::Host::Ipv6(ip)) if ip.is_loopback());
    if url.scheme() != "https"
        && !(url.scheme() == "http" && config.allow_loopback_http && loopback)
    {
        return Err(
            "updates require HTTPS (or explicitly allowed literal loopback HTTP fixtures)".into(),
        );
    }
    Ok(())
}

pub(super) fn validate_config(config: &UpdaterConfig) -> Result<(), String> {
    if config.app_id.is_empty()
        || config.app_id.len() > 128
        || config.channel.is_empty()
        || config.channel.len() > 64
        || config.max_archive_bytes == 0
        || config.max_archive_bytes > 256 * 1024 * 1024
        || config.max_unpacked_bytes == 0
        || config.max_unpacked_bytes > 1024 * 1024 * 1024
        || config.max_entries == 0
        || config.max_entries > 100000
        || config.timeout.is_zero()
        || config.timeout > Duration::from_secs(300)
    {
        return Err("invalid updater configuration limits".into());
    }
    let key = VerifyingKey::from_bytes(&config.public_key).map_err(|_| "invalid public key")?;
    if key.is_weak() {
        return Err("weak Ed25519 public key is forbidden".into());
    }
    installation::relative_path(&config.executable)?;
    if !config.executable.starts_with("Contents/MacOS/") {
        return Err("executable must be inside Contents/MacOS".into());
    }
    validate_url(config, &config.feed_url)
}

/// Mirrors host image acquisition: a streaming owned body with a hard byte cap.
/// Cancellation includes stalled headers/body; redirects never leave the pinned origin.
pub(super) fn download(
    config: &UpdaterConfig,
    client: &Arc<dyn gpui::http_client::HttpClient>,
    url: &str,
    limit: u32,
    context: &NativeCallContext,
) -> Result<Vec<u8>, String> {
    context.check_cancelled()?;
    validate_url(config, url)?;
    let request = Request::builder()
        .uri(url)
        .follow_redirects(RedirectPolicy::NoFollow)
        .timeout(config.timeout)
        .body(AsyncBody::empty())
        .map_err(|e| e.to_string())?;
    futures::executor::block_on(async {
        tokio::select! {
            _ = context.cancelled() => Err("native command was cancelled".into()),
            result = tokio::time::timeout(config.timeout, async {
                let mut response = client.send(request).await.map_err(|e| format!("update request failed: {e}"))?;
                if response.status() != gpui::http_client::StatusCode::OK { return Err(format!("update HTTP status {}", response.status())); }
                let mut bytes = Vec::new();
                response.body_mut().take(u64::from(limit) + 1).read_to_end(&mut bytes).await.map_err(|e| e.to_string())?;
                if bytes.len() > limit as usize { return Err("update download exceeds byte limit".into()); }
                Ok(bytes)
            }) => result.map_err(|_| "update download timed out".to_owned())?,
        }
    })
}
