# Windows/Linux activation ingress

Date: 2026-09-07. Read-only design against the current acknowledged application lifecycle. No implementation/dependency changes or builds performed. OS association installation and real Windows/Linux desktop acceptance are separate steps.

## Recommended boundary

Add one **app-owned pre-runtime instance bootstrap**, followed by a bounded typed activation ingress that feeds the existing host lifecycle. The bootstrap must happen before starting QuickJS/Bun, native services, or GPUI windows. Current [run_application](../../crates/solid-gpui/src/host/mod.rs:1519) takes an already-created runtime, so wrapping only `run_profile` is too late: a secondary process could already run application side effects.

Suggested API shape, with names provisional:

```rust
struct InstanceConfig {
    application_id: ApplicationId, // stable bounded identifier, chosen by the app
    profile: InstanceProfile,      // bounded logical profile; never an arbitrary endpoint path
    allowed_url_schemes: Vec<String>,
    allow_documents: bool,
    startup_deadline: Duration,
}
enum InstanceStart {
    Primary(PrimaryInstance),
    Forwarded(DeliveryReceipt),
}
fn prepare_instance(config: &InstanceConfig, request: ActivationRequest) -> Result<InstanceStart>;
```

The app constructs its runtime only for Primary, then passes the owned guard/receiver into the shared host runner. `--help`, `--version`, native code generation and bundle self-check run before this bootstrap and do not contact a running app. Development profiles can explicitly disable single-instance behavior or use a separate profile; do not derive identity from CWD/executable path and accidentally merge distinct apps.

**Scope policy:** Windows uses the current logon SID/session; Linux uses the current UID and runtime directory. This is an explicit default, not a claim of identical cross-session semantics: XDG_RUNTIME_DIR is shared by all simultaneous logins of one user. Applications needing a distinct Linux graphical session must supply a bounded session/profile key from their launcher; do not guess it from whichever of DISPLAY/Wayland/XDG_SESSION_ID happens to be set. Never broaden Windows to all machine users or make an elevated instance the implicit receiver for ordinary launches.

## Existing lifecycle to reuse

[ApplicationLifecycle](../../crates/solid-gpui/src/host/application_lifecycle.rs:14) already queues 32 events, limits each to 64 URLs × 4096 bytes, assigns sequence IDs, waits for a renderer generation, and retains events until acknowledged. [Host ingress](../../crates/solid-gpui/src/host/mod.rs:997) currently receives bounded native `open-urls`/`reopen` events and [activate_application](../../crates/solid-gpui/src/host/mod.rs:521) enqueues/delivers them. [Solid activation handling](../../packages/solid-gpui/src/application.ts:142) runs onActivate before updating the acknowledged sequence. Keep this as the authoritative delivery lifecycle.

Replace the ingress tuple with a request carrying validated reason/URLs, request ID and a bounded response waiter. Return its assigned lifecycle sequence from enqueue; correlate that sequence with external request ID. Complete a Delivered receipt when `ApplicationLifecycle::configure` consumes the matching acknowledgement, not when bytes enter a channel. Fix `activate_application`'s current transport-terminated success return for external ingress: it must report ShuttingDown rather than claim acceptance.

## Linux ownership and transport

Use a filesystem Unix stream socket in an app subdirectory of the actual `$XDG_RUNTIME_DIR`. Validate that the runtime directory is absolute, owned by effective UID, and has 0700 access, following the [XDG runtime-directory contract](https://specifications.freedesktop.org/basedir/latest/). Create/open the app directory with mode 0700 using directory-relative no-follow operations; validate ownership/type of existing entries. Missing/invalid runtime directory returns a clear single-instance-unavailable error unless the app explicitly provides an equivalent private local runtime directory. Do not silently fall back to a public predictable `/tmp/app.sock`.

Use a **held file lock** as ownership, not a PID file and not existence of the socket. Installed rustix 1.1.4 provides [flock](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.4/src/fs/fd.rs:317) and [openat](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.4/src/fs/at.rs:78); add the exact explicit fs/process dependencies/features rather than importing a transitive crate accidentally. A nonblocking exclusive flock yields either primary ownership or secondary contention. Keep the lock FD open for the entire primary lifetime and close-on-exec; never unlink/recreate the lock file during ordinary shutdown because that can split ownership across two inodes.

While holding the lock, remove a stale socket only after validating that the entry is the expected socket in the private directory, then bind/listen. A process that failed to obtain the lock must **never** unlink the socket or kill the recorded PID. PID metadata is diagnostic only; it is not liveness authority. Crash releases the kernel lock; the next lock holder safely replaces the stale socket.

Tokio 1.53.1 already has UnixListener/UnixStream and [UnixStream::peer_cred](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.53.1/src/net/unix/stream.rs:967). Verify peer UID equals the expected UID on both accepted connections and client connection to the primary. Place the socket under the validated directory with restrictive permissions and a short bounded component name; Linux sockaddr path length is bounded, so fail clearly for an unusably long configured runtime path rather than truncate collision-prone names.

A secondary that sees a held lock before the socket appears waits/retries connection until its single startup deadline. On connection refusal, check ownership again: it may become primary **only after acquiring the lock**, never merely because a timeout elapsed. Use bounded retry/backoff for this startup race, not a permanent poller.

## Windows ownership and transport

Use a local named pipe whose name incorporates the bounded application/profile identity and current logon identity. Do not place arbitrary user input directly into a pipe path. One pipe instance is enough for ownership and a bounded small number for concurrent handoffs.

Installed Tokio 1.53.1 exposes:

- [first_pipe_instance(true)](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.53.1/src/net/windows/named_pipe.rs:2051) for exclusive initial creation.
- [reject_remote_clients(true)](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.53.1/src/net/windows/named_pipe.rs:2164).
- [create_with_security_attributes_raw](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.53.1/src/net/windows/named_pipe.rs:2304) for a supplied security descriptor.
- [ClientOptions security QOS](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.53.1/src/net/windows/named_pipe.rs:2429) defaults to SECURITY_IDENTIFICATION; retain that protection rather than allow arbitrary server impersonation.

Create the initial server with first_pipe_instance(true). Success elects the primary. An already-existing endpoint sends the process down the client path; permission/descriptor/invalid-name failures remain explicit errors. **Do not interpret every ACCESS_DENIED as proof of a legitimate primary**: authenticate the connected endpoint and enforce the startup deadline.

Keep at least one server pipe handle alive continuously while the primary owns the application. Once an instance connects, create the next listening instance (first_pipe_instance false) **before** handing the current connection to a worker. Use a held listener/guard and a small explicit max instance count. A period with no server handles permits a second primary; prevent that by construction. This avoids an additional named-mutex owner thread and abandoned-mutex protocol. Crash closes kernel handles and removes the pipe instances; clients can then retry initial creation within their deadline. During graceful shutdown, retain ownership until the application/runtime is actually stopped.

Supply an explicit DACL limited to the current logon SID (plus necessary server owner/system access), and reject remote clients. Windows' default pipe descriptor is too broad: [Microsoft documents](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights) read access for Everyone/anonymous and warns that generic write access can also permit creating pipe instances. Use the required individual pipe rights rather than a blanket broad ACL. Name scoping alone is not access control. Relevant installed bindings: TokenLogonSid/GetTokenInformation, ConvertStringSecurityDescriptorToSecurityDescriptorW, GetNamedPipeClientProcessId and GetNamedPipeServerProcessId in windows 0.62.2. Lifetime-own the security descriptor while creating handles and release native allocations correctly.

The trust boundary is **other users/logon sessions**, not hostile code already executing as the same user/logon. Same-principal processes can generally modify the app/environment too; do not advertise the local endpoint as a sandbox boundary. Peer-process token verification can confirm expected user/logon where needed; do not invent an executable-path string check as cryptographic application identity.

After authenticating the server, a launcher entitled to activate a foreground window can call `AllowSetForegroundWindow(server_pid)` before forwarding. This grants only that PID, not ASFW_ANY. [Microsoft's API](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-allowsetforegroundwindow) is conditional; failure does not mean document delivery failed. Report focus permission separately from a Delivered receipt.

## Bounded protocol and acknowledgement

Use a separate small activation protocol, not the internal renderer control protocol. A local launcher must not gain access to arbitrary Native Modules, process commands, snapshots or window mutation.

One length-prefixed request per connection is sufficient. A closed versioned DTO contains protocol version, application/profile identity, a random 128-bit request ID, and `reopen` or `open-urls` plus validated URL list. Responses contain request ID, primary generation nonce, and a closed status (`delivered`, `busy`, `rejected`, `shutting-down`, `incompatible`). Use existing generated/typed serialization or a bounded parser; do not introduce JSON-RPC dispatch or shell commands.

Enforce limits **before allocation**: fixed maximum encoded frame (e.g. 272 KiB, enough for the existing raw URL budget plus a compact binary envelope), at most 64 items, at most 4096 UTF-8 bytes each, no controls, fixed count/depth, no trailing data. If choosing JSON, bound the encoded bytes independently because escapes can expand raw strings; do not assume 64×4096 is a sufficient JSON frame maximum. Limit concurrent connections, partial-frame duration, and pending requests to the existing lifecycle capacity; bound total retained bytes, not just item count. Slow clients must not occupy all slots indefinitely.

Enqueue on the GPUI foreground using the existing bounded channel and a response oneshot. Queue-full returns Busy without mutation. Initial primary launch includes its own requested URL/document payload through this same normalized path; avoid an empty launch followed by duplicated open events. A secondary with no payload maps to reopen, while a primary maps to launch (or launch followed by one explicit open-urls event if the contract deliberately keeps launch payload-free).

**Delivered means:** the current Solid application invoked onActivate and returned synchronously, then acknowledged that sequence. It does not mean the app finished opening/parsing/saving a document, since onActivate is not an async completion contract. Startup can bind/listen early, but no Delivered response should be sent before the renderer is ready and the callback acknowledgement arrives. An optional Accepted status may expose queued state, but must not be treated as final delivery success.

**Retries/dedup:** reuse the same request ID across reconnects. Keep bounded in-flight request→sequence state and a bounded completed-receipt cache for the stated retry window. If a duplicate is pending, attach a bounded waiter rather than enqueue again; if complete, replay the receipt. Do not evict in-flight entries; return Busy instead. Correlate with a fresh primary generation nonce so clients can detect restart. A crash after callback side effects but before receipt creates an unavoidable uncertain outcome without a durable application transaction; do not promise exactly-once delivery across process crashes or blindly replay after generation changes. Return a specific indeterminate-delivery error after an unacknowledged write unless the caller explicitly chooses retry. Runtime sequence acknowledgement already handles HMR within the same primary generation.

## CLI URL/document normalization

App config chooses registered schemes and document support. Accept native `OsString` arguments, treat `--` as the end of options, and distinguish `--open-url <url>` from document arguments. Never execute forwarded command-line text or reinterpret it as host/runtime flags.

Normalize relative document paths against the **secondary launcher's CWD before forwarding**, not the primary's CWD. Convert to local file URLs with a real URL/path API; preserve percent-encoding and Unicode, do not concatenate `file://` strings. The current cross-runtime payload is a UTF-8 URL string: a Linux path with non-UTF-8 bytes or a Windows path with unpaired surrogates needs either an explicit unsupported-path error or a deliberate new byte-preserving document DTO. Never use lossy conversion silently. Do not require the target to exist merely to normalize the path; the app should report missing documents. Decide UNC/network file URLs explicitly because they may trigger network access.

Validate scheme allowlist both before send and in the primary, and limit application-specific URL parsing to the app. Incoming deep links can contain credentials/tokens; log request identity/status, never full raw URLs by default. Activation opens content but should not execute commands embedded in a URL unless that behavior is separately defined by the application.

Linux focus/desktop launch tokens (`XDG_ACTIVATION_TOKEN`/startup notification) are a distinct bounded optional activation field if implemented. Do not forward the entire environment. Delivery can succeed while compositor policy refuses focus; do not classify it as duplicate-instance failure.

## Packaging versus forwarding

The endpoint handles launches **after the executable has been invoked**. It does not register a protocol or document type.

- Linux package ownership: desktop entry, MimeType including selected `x-scheme-handler/...` and document types, optional MIME definitions. Use a single correctly escaped Exec field with `%U` or `%F`; the [Desktop Entry specification](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html) states each is a list of separate arguments and must stand alone. Keep `DBusActivatable=false`/absent unless implementing the actual D-Bus application interface; a Unix socket does not satisfy it.
- Windows package ownership: explicit per-user protocol/file associations with quoted executable/argument invocation, or packaged-app manifest declarations. Do not overwrite user defaults automatically. The executable uses the same bootstrap whether launched by a terminal, Explorer, or an OS protocol association.
- macOS already has real on_open_urls/on_reopen hooks in this host; avoid running a second conflicting single-instance scheme there without a separate requirement. Web tabs likewise are outside this Windows/Linux process ingress scope.

## Shutdown and crash recovery

When quit begins, mark ingress ShuttingDown and stop admitting new activations. Reject unresolved waiters with a structured status; don't report Delivered for work the renderer won't acknowledge. Cancel connection tasks and unregister listeners without blocking the GPUI foreground on joins. Keep ownership until renderer/native-service shutdown completes, then release it. Linux unlinks only its socket while still owning the lock, then closes the lock FD; leave the lock file inode. Windows closes final pipe handles last. Crash cleanup relies on kernel locks/handles, not a stale PID timeout.

The bootstrap itself can use a small existing-runtime-compatible IO task; it must not start an application VM just to discover it is secondary. If the public API accepts an already-running runtime, add a separate `prepare_instance` boundary or refactor construction to a closure so secondary startup has no app side effects.

## Concrete file scope and key acceptance

Add host `instance.rs` plus `instance/linux.rs` and `instance/windows.rs`, with one shared bounded parser/request registry. Refactor host bootstrap and the activation receiver/ack paths in `host/mod.rs` and `application_lifecycle.rs`. App/example launchers choose config; packaging scripts consume app association metadata separately. Explicit target deps can reuse locked Tokio 1.53.1, rustix 1.1.4 and windows 0.62.2; no general IPC framework is required.

Keep a few meaningful process tests:

1. Race two starters: exactly one runtime factory runs; secondary gets Delivered only after the simulated/real application acknowledgement. Include startup delay and same-ID reconnect.
2. Crash primary, then restart: no stale ownership; Linux socket recovered only by lock owner, Windows fresh first instance succeeds. A hung live owner produces a timeout rather than force takeover.
3. Malformed/oversize/slow input and queue pressure: bounded memory/connections, no activation mutation, honest Busy/Rejected responses.
4. Zero-window keep-alive then URL/document activation creates a fresh Surface through existing lifecycle; HMR replays pending sequence without duplicate completed request.
5. User/session isolation and actual OS association/focus behavior on native Windows/Linux are desktop acceptance. Do not substitute a shared-trait compile or macOS Unix-socket test for Windows named-pipe ACL validation.

This design reuses the application acknowledgement already implemented, isolates forwarding from app execution, and makes crash uncertainty explicit without adding a second application lifecycle.
