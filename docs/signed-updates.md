# Signed application updates

Signed updates are an opt-in application-owned native service. Enable the Rust
`signed-updater` feature and explicitly register `updater::native_module` with
your host. The default host and Gallery never configure or install updates.
Solid owns consent, progress messages, and restart UI; Rust owns trust, download,
verification, extraction, installation, and rollback. No archive bytes, keys,
installation paths, or GPUI handles travel through the JavaScript interface.

## Configure the trusted service

Create `Arc<SignedUpdater>` with `SignedUpdater::new(config, client)`, then pass
it to `native_module(Some(service))`. `UpdaterConfig` is native application code:

| Field                               | Contract                                                                    |
| ----------------------------------- | --------------------------------------------------------------------------- |
| `app_id`, `channel`                 | Exact identities bound into the signed release                              |
| `feed_url`                          | One HTTPS endpoint, no credentials, fragments, or redirects                 |
| `public_key`                        | Pinned 32-byte Ed25519 verifying key; weak keys rejected                    |
| `current_sequence`                  | Monotonically increasing release number compiled into each host             |
| `install_path`                      | Existing absolute canonical `.app` directory; symlinks rejected             |
| `executable`                        | Exact path beneath `Contents/MacOS`, relative to the bundle                 |
| `max_archive_bytes`                 | Application limit, at most 256 MiB                                          |
| `max_unpacked_bytes`, `max_entries` | Extraction limits, at most 1 GiB and 100,000 entries                        |
| `timeout`                           | Whole-request deadline, positive and at most five minutes                   |
| `allow_loopback_http`               | Explicit literal loopback HTTP fixture permission; keep false in production |

Supply the same `Arc<dyn gpui::http_client::HttpClient>` used by the host for
downloads, or its `ReqwestClient`. There is no alternate downloader. The native
executor runs commands in its bounded blocking pool, checks cooperative
cancellation between chunks, and retains admission until the worker exits.
Stalled HTTP headers/body are interruptible through `NativeCallContext`.
At most one operation per updater can own acquisition/installation state.
An OS advisory lock prevents a second updater instance owning the same bundle.

## Generated JavaScript commands

The actual registered module generates these methods in the application's
`#native` bindings. It uses the existing native request identity, byte limits,
Surface cancellation, and `NativeCallOptions.signal` semantics.

| Method                     | Behavior                                                                                                                                             |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `updateStatus()`           | Enabled state, platform, pending restart and rollback availability                                                                                   |
| `checkUpdate()`            | Fetch/verify the signed feed; return a bounded offer with opaque token, version, sequence and archive byte count, or null for an older/equal release |
| `installUpdate({ token })` | Require the most recently checked offer; acquire, hash, verify, stage and exchange the bundle; retain rollback and report pending restart            |
| `rollbackUpdate()`         | Atomically exchange the previous bundle back before deleting the rejected candidate                                                                  |
| `confirmUpdate()`          | Delete the backup only when the newly restarted host has the signed release sequence and its application health check has succeeded                  |

```tsx
const native = useNative(); // generated application bindings
const controller = new AbortController();
onCleanup(() => controller.abort());
const offer = await native.checkUpdate(undefined, { signal: controller.signal });
// Present offer.version and offer.archiveBytes; obtain application/user consent.
if (offer) {
  const installed = await native.installUpdate({ token: offer.token }, { signal: controller.signal });
  // installed.pendingRestart: ask the user to finish work and restart the app.
}
```

Registration with `native_module(None)` exports the same real contract, returns
disabled status, and rejects acquisition/installation commands. The desktop
example uses this form. A checked token does not permit an arbitrary URL or path.
Checking again replaces the offer; pending installations must be confirmed or
rolled back before another update.

## Feed and packaging format

The feed is a strict JSON object with `payload` and `signature`. Both are
lowercase hex. `payload` contains the **exact UTF-8 JSON bytes** signed with
Ed25519; `signature` is a 64-byte Ed25519 signature (128 hex characters).
Verification uses `ed25519-dalek::VerifyingKey::verify_strict`; it happens before
parsing or using the payload. Unknown fields and malformed encodings fail.
Keep private signing keys in your release tooling, outside the app and repository.

The decoded payload has this exact shape:

```json
{
  "format": "solid-gpui-update-v1",
  "appId": "com.example.desktop",
  "channel": "stable",
  "platform": "macos-aarch64",
  "sequence": 2,
  "version": "1.1.0",
  "archiveFormat": "app-tar-v1",
  "url": "https://updates.example.com/desktop-1.1.0.tar",
  "archiveBytes": 10240,
  "sha256": "64 lowercase hexadecimal characters describing the final archive",
  "bundle": "Desktop.app",
  "executable": "Contents/MacOS/desktop"
}
```

The artifact must share the pinned feed's exact origin. Redirects are rejected.
The signed hash and byte count are checked against the entire bounded downloaded
archive, then the manifest signature is checked again before staging. The feed
is capped at 32 KiB; the decoded payload is capped at 12 KiB. This authenticates
the artifact and all routing/identity metadata, rather than trusting an unsigned
version or channel alongside an artifact signature. Replay of releases at or
below `current_sequence` produces no offer. Publishing uses one increasing
sequence per application/channel/platform; version strings are display labels.

`app-tar-v1` is an **uncompressed USTAR** archive with one exact `.app` root and
only regular files/directories. No symlinks, hardlinks, sparse entries, device
files, PAX/GNU extension headers, absolute paths, backslashes, colons, traversal,
duplicate paths, or more than 32 path components are accepted. Ownership,
xattrs, and privileged mode bits are not restored. Regular files use 0644 or
0755 according to their execute bits. Directory contents, entry count and total
file bytes are bounded; the configured executable must be a regular executable
file. Package self-contained bundles without framework symlinks for this format.

Generic packaging must finish bundling, code signing, notarization policy,
extracted-artifact verification, and release identity assignment **before**
creating the updater archive and signed manifest. A normal distributable ZIP is
not this update archive. `packageSignedUpdate()` from `@solid-gpui/vite/package`
emits the additional USTAR and verifies an application-owned Ed25519 signer;
it never treats archive checksum JSON as a signed updater feed. Changing
archive format requires a new explicit contract, not an extraction fallback.

## Installation and restart policy

The real installation backend is macOS. It stages in a private sibling directory
on the same filesystem, synchronizes staged files/directories, records old/new
directory identities and the signed release, and uses `renameatx_np(RENAME_SWAP)`
for one atomic exchange. The live install path is never temporarily absent.
Unsupported filesystems fail explicitly. Installation errors or cancellation
after exchange attempt immediate atomic rollback; a failed rollback preserves
the transaction and reports the failure. No installer shell scripts run.

The sibling `.Bundle.app.solid-update` holds an advisory lock, atomic recovery
record, and previous bundle until explicit confirmation. Reopening the service
recognizes old/new directory identities: an uncommitted exchange is cleaned up;
a committed exchange retains rollback. Unexpected identities stop recovery for
manual inspection. The installer does not use `current_exe` or discover an
installation directory automatically. The app explicitly grants the install path.

There is **no automatic exit, process spawn, or relaunch**. The application saves
documents, coordinates other instances, and asks the user to quit/reopen using
its own lifecycle policy. `pendingRestart` reports disk installation, not a new
running process. After restarting, the app performs its own health check and
calls `confirmUpdate`; the old process cannot confirm a newer sequence.
Rolling back also reports `pendingRestart` in the process that performed the
exchange, because changing the on-disk bundle does not replace running code.

Windows/Linux return an unsupported error when constructing `SignedUpdater`;
they can export disabled bindings. The module is absent on WASM. Source gates
are explicit; successful macOS fixture tests do not qualify another OS, real
notarized releases, Gatekeeper, power-loss durability, or external relaunch.

The security boundary assumes trusted release signing code/key, native config,
and app-owned install parent/transaction directory. This is not an elevated
installer or protection against another process with permission to rewrite those
directories concurrently. Preserve backups and stop when identity checks fail.
Test qualification uses temporary application paths, local HTTP and fixture keys;
it never installs into the active host or publishes releases.
