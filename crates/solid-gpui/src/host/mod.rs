mod application_lifecycle;
mod popup;
use crate::motion;
#[cfg(feature = "embedded-bun")]
use crate::runtime::embedded::EmbeddedBunAdapter;
use crate::{
    Command, CommandKind, CommandMeta, CommandOperation, CommandValue, DecodedMessage,
    ExtensionRegistry, KeybindingDefinition, MenuAction, NoExtensions, PROTOCOL_VERSION,
    ProcessAdapter, RuntimeAdapter, SolidRoot, WindowOpenOptions, decode_message,
    fatal_runtime_failure,
};
use crate::{Event, send_event_or_exit};
use application_lifecycle::ApplicationLifecycle;
#[cfg(any(test, feature = "test-support"))]
use gpui::WindowHandle;
use gpui::{
    AnyWindowHandle, App, AppContext, Bounds, Context, DummyKeyboardMapper, Entity, KeyBinding,
    Subscription, SystemNotificationResponse, TitlebarOptions, WindowBounds, WindowId, WindowKind,
    WindowOptions, px, size,
};
use std::backtrace::Backtrace;
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsString;
use std::fmt::Display;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
const CRASH_DIR_ENV: &str = "SOLID_GPUI_CRASH_DIR";
const COMMAND_ENV: &str = "SOLID_GPUI_RENDERER_COMMAND";
const ARGS_ENV: &str = "SOLID_GPUI_RENDERER_ARGS";
const LOG_ENV: &str = "SOLID_GPUI_LOG";
mod commit_pump;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
use commit_pump::CommitPump;

struct HostAssets;
impl gpui::AssetSource for HostAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        #[cfg(feature = "gpui-component")]
        if path.starts_with("icons/") {
            return gpui_kit_assets::Assets.load(path);
        }
        if let Some(bytes) = crate::icons::load(path) {
            return Ok(Some(std::borrow::Cow::Borrowed(bytes)));
        }
        gpui_iconify::IconAssets.load(path)
    }
    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        let icons = gpui_iconify::IconAssets.list(path)?;
        #[cfg(feature = "gpui-component")]
        let icons = icons
            .into_iter()
            .chain(gpui_kit_assets::Assets.list(path)?)
            .collect();
        Ok(icons)
    }
}

/// Host capabilities describe local policy; they are not negotiated on the
/// renderer protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostCapabilities {
    /// Whether the runner forwards system-notification responses to Solid.
    pub notification_responses: bool,
    /// Whether the runner admits renderer keybinding commands.
    pub set_keybindings: bool,
}

/// The reusable host seam. Profiles provide only the provider-specific pieces;
/// protocol admission, surface ownership, and runtime lifecycle stay here.
pub trait HostProfile: 'static {
    fn native_bindings(&self) -> Result<String, String> {
        Ok(String::new())
    }
    /// Configure native window defaults before renderer open-surface overrides.
    fn window_options(&self, options: WindowOptions, _cx: &App) -> WindowOptions {
        options
    }
    fn capabilities(&self) -> HostCapabilities;
    fn extension_registry(&self) -> Rc<dyn ExtensionRegistry>;
    fn initialize(&mut self, cx: &mut App);
    fn open_window(
        &self,
        options: WindowOptions,
        runtime: Arc<dyn RuntimeAdapter>,
        extensions: Rc<dyn ExtensionRegistry>,
        cx: &mut App,
    ) -> Result<(AnyWindowHandle, Entity<SolidRoot>), String>;
    fn restore_keybindings(&self, baseline: &[KeyBinding], dynamic: Vec<KeyBinding>, cx: &mut App);
    fn rejected_command_reason(&self, _: CommandKind) -> Option<&'static str> {
        None
    }
}

/// The default host profile: no extension adapters and a direct SolidRoot
/// window root. This preserves the existing host behavior.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultHostProfile;

impl HostProfile for DefaultHostProfile {
    fn capabilities(&self) -> HostCapabilities {
        HostCapabilities {
            notification_responses: true,
            set_keybindings: true,
        }
    }

    fn extension_registry(&self) -> Rc<dyn ExtensionRegistry> {
        Rc::new(NoExtensions)
    }

    fn initialize(&mut self, _: &mut App) {}

    fn open_window(
        &self,
        options: WindowOptions,
        runtime: Arc<dyn RuntimeAdapter>,
        extensions: Rc<dyn ExtensionRegistry>,
        cx: &mut App,
    ) -> Result<(AnyWindowHandle, Entity<SolidRoot>), String> {
        let window = cx
            .open_window(options, move |_, cx| {
                cx.new(|_| SolidRoot::with_extensions(runtime, extensions))
            })
            .map_err(|error| format!("failed to open GPUI window: {error}"))?;
        let root = window
            .entity(cx)
            .map_err(|error| format!("GPUI did not return a root entity: {error}"))?;
        Ok((window.into(), root))
    }

    fn restore_keybindings(&self, baseline: &[KeyBinding], dynamic: Vec<KeyBinding>, cx: &mut App) {
        cx.clear_key_bindings();
        cx.bind_keys(baseline.iter().cloned().chain(dynamic));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeMode {
    Process,
    Embedded,
    QuickJs,
    QuickJsDev,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum LogLevel {
    Off,
    Error,
    Info,
    Debug,
}

fn parse_log_level(value: Option<&str>) -> Result<LogLevel, &'static str> {
    match value.map(str::to_ascii_lowercase).as_deref() {
        None | Some("error") => Ok(LogLevel::Error),
        Some("off") => Ok(LogLevel::Off),
        Some("info") => Ok(LogLevel::Info),
        Some("debug") => Ok(LogLevel::Debug),
        Some(_) => Err("expected off, error, info, or debug"),
    }
}

fn resolve_log_level<F>(value: Option<&str>, warn: F) -> LogLevel
where
    F: FnOnce(&str),
{
    match parse_log_level(value) {
        Ok(level) => level,
        Err(reason) => {
            warn(reason);
            LogLevel::Error
        }
    }
}

fn host_log(level: LogLevel, message_level: LogLevel, message: impl Display) {
    if level >= message_level {
        eprintln!("solid-gpui-host: {message}");
    }
}

fn install_panic_hook() {
    let crash_dir = env::var_os(CRASH_DIR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    install_panic_hook_in(crash_dir);
}

fn install_panic_hook_in(crash_dir: PathBuf) {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        default_hook(info);
        match write_crash_report(&crash_dir, info) {
            Ok(path) => eprintln!("solid-gpui-host: crash report: {}", path.display()),
            Err(error) => eprintln!("solid-gpui-host: unable to write crash report: {error}"),
        }
    }));
}

fn write_crash_report(
    crash_dir: &std::path::Path,
    info: &panic::PanicHookInfo<'_>,
) -> std::io::Result<PathBuf> {
    fs::create_dir_all(crash_dir)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let path = crash_dir.join(format!(
        "solid-gpui-host-{}-{timestamp}.log",
        std::process::id()
    ));
    let mut report = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    let message = info
        .payload()
        .downcast_ref::<&str>()
        .map(|value| (*value).to_owned())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-string panic payload".to_owned());
    let location = info
        .location()
        .map(|location| {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        })
        .unwrap_or_else(|| "unknown".to_owned());
    let backtrace = Backtrace::capture();
    writeln!(report, "solid-gpui-host crash report")?;
    writeln!(report, "version: {}", env!("CARGO_PKG_VERSION"))?;
    writeln!(
        report,
        "platform: {}-{}",
        env::consts::OS,
        env::consts::ARCH
    )?;
    writeln!(report, "pid: {}", std::process::id())?;
    writeln!(report, "timestamp_ms: {timestamp}")?;
    writeln!(report, "panic: {message}")?;
    writeln!(report, "location: {location}")?;
    writeln!(report, "backtrace:\\n{backtrace}")?;
    report.flush()?;
    Ok(path)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClosePolicy {
    Allow,
    RequireConfirmation,
}

struct Surface {
    window: AnyWindowHandle,
    root: Entity<SolidRoot>,
    _activation: Subscription,
    close_policy: ClosePolicy,
    pending_close_request: Option<u32>,
    next_close_request: u32,
}

#[derive(Debug, Eq, PartialEq)]
enum CommandAdmission {
    Accepted,
    Rejected(String),
}

fn check_command_metadata(meta: CommandMeta, actual: (u32, u32, u32)) -> CommandAdmission {
    if meta.surface_id != actual.0 || meta.epoch != actual.1 {
        return CommandAdmission::Rejected("surface or epoch mismatch".to_owned());
    }
    if meta.after_revision != actual.2 {
        return CommandAdmission::Rejected("command revision is stale".to_owned());
    }
    CommandAdmission::Accepted
}

struct NativeStateRegistry {
    popups: HashMap<u32, popup::PopupSession>,
    application: ApplicationLifecycle,
    application_surface_id: Option<u32>,
    runtime: Arc<dyn RuntimeAdapter>,
    profile: Box<dyn HostProfile>,
    surfaces: HashMap<u32, Surface>,
    windows: HashMap<WindowId, u32>,
    keybindings: HashMap<u32, Vec<KeyBinding>>,
    retired_surface_ids: HashSet<u32>,
    next_surface_id: u32,
    transport_terminated: bool,
    close_subscription: Option<Subscription>,
    baseline_keybindings: Vec<KeyBinding>,
}

impl NativeStateRegistry {
    #[cfg(any(test, feature = "test-support"))]
    fn new(runtime: Arc<dyn RuntimeAdapter>) -> Self {
        Self::with_profile(runtime, DefaultHostProfile, Vec::new())
    }

    fn with_profile(
        runtime: Arc<dyn RuntimeAdapter>,
        profile: impl HostProfile,
        baseline_keybindings: Vec<KeyBinding>,
    ) -> Self {
        Self {
            popups: HashMap::new(),
            runtime,
            profile: Box::new(profile),
            application: ApplicationLifecycle::default(),
            application_surface_id: None,
            surfaces: HashMap::new(),
            windows: HashMap::new(),
            keybindings: HashMap::new(),
            retired_surface_ids: HashSet::new(),
            next_surface_id: 1,
            transport_terminated: false,
            close_subscription: None,
            baseline_keybindings,
        }
    }

    fn allocate_surface_id(&mut self) -> Result<u32, String> {
        loop {
            let id = self.next_surface_id;
            if id == 0 {
                return Err("surface id space exhausted".to_owned());
            }
            self.next_surface_id = id
                .checked_add(1)
                .ok_or_else(|| "surface id space exhausted".to_owned())?;
            if !self.surfaces.contains_key(&id) && !self.retired_surface_ids.contains(&id) {
                return Ok(id);
            }
        }
    }

    fn apply_window_open_options(
        options: &mut WindowOptions,
        window_options: Option<&WindowOpenOptions>,
    ) -> Result<(), String> {
        let Some(window_options) = window_options else {
            return Ok(());
        };
        options.kind = match window_options.kind.unwrap_or(0) {
            0 => WindowKind::Normal,
            1 => WindowKind::Floating,
            2 => WindowKind::Dialog,
            _ => return Err("open-surface kind is invalid".to_owned()),
        };
        if let Some(resizable) = window_options.resizable {
            options.is_resizable = resizable;
        }
        if let Some((width, height)) = window_options.min_size {
            options.window_min_size = Some(size(px(width as f32), px(height as f32)));
        }
        Ok(())
    }

    fn open_window(
        &self,
        title: Option<&str>,
        width: u32,
        height: u32,
        window_options: Option<&WindowOpenOptions>,
        cx: &mut Context<Self>,
    ) -> Result<Surface, String> {
        let bounds = Bounds::centered(None, size(px(width as f32), px(height as f32)), cx);
        let mut options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };
        options = self.profile.window_options(options, cx);
        Self::apply_window_open_options(&mut options, window_options)?;
        if let Some(title) = title {
            options
                .titlebar
                .get_or_insert_with(TitlebarOptions::default)
                .title = Some(title.to_owned().into());
        }
        let runtime = Arc::clone(&self.runtime);
        let extensions = self.profile.extension_registry();
        let (window, root) = self.profile.open_window(options, runtime, extensions, cx)?;
        let registry = cx.weak_entity();
        let activation_registry = registry.clone();
        let window_id = window.window_id();
        let activation = window
            .update(cx, |_, window, cx| {
                window.on_window_should_close(cx, move |_, app| {
                    registry
                        .upgrade()
                        .map(|registry| {
                            registry
                                .update(app, |registry, cx| registry.should_close(window_id, cx))
                        })
                        .unwrap_or(true)
                });
                root.update(cx, |_, cx| {
                    cx.observe_window_activation(window, move |_, _, cx| {
                        if let Some(registry) = activation_registry.upgrade() {
                            registry.update(cx, |registry, cx| {
                                registry.restore_keybindings(cx);
                                let registry = cx.weak_entity();
                                cx.defer(move |cx| {
                                    let _ = registry.update(cx, |registry, cx| {
                                        registry.dismiss_unrelated_popups(cx)
                                    });
                                });
                            });
                        }
                    })
                })
            })
            .map_err(|error| format!("failed to install close policy: {error}"))?;
        Ok(Surface {
            window,
            root,
            _activation: activation,
            close_policy: ClosePolicy::Allow,
            pending_close_request: None,
            next_close_request: 1,
        })
    }
    fn should_close(&mut self, window_id: WindowId, cx: &mut Context<Self>) -> bool {
        if self.transport_terminated {
            return true;
        }
        let Some(surface_id) = self.windows.get(&window_id).copied() else {
            return true;
        };
        let Some(surface) = self.surfaces.get_mut(&surface_id) else {
            return true;
        };
        if surface.close_policy == ClosePolicy::Allow {
            return true;
        }
        if surface.pending_close_request.is_some() {
            return false;
        }
        let request_id = surface.next_close_request;
        let Some(next_request_id) = request_id.checked_add(1) else {
            return false;
        };
        surface.next_close_request = next_request_id;
        surface.pending_close_request = Some(request_id);
        let root = surface.root.clone();
        root.update(cx, |root, _| root.emit_close_requested(request_id));
        false
    }
    fn set_close_policy(&mut self, command: Command, cx: &mut Context<Self>) {
        let meta = command.meta;
        let CommandOperation::SetClosePolicy { policy } = command.operation else {
            return;
        };
        if meta.node_id != 1 {
            self.send_command_result(
                meta,
                CommandKind::SetClosePolicy,
                false,
                Some("setClosePolicy requires the root container".to_owned()),
                None,
                cx,
            );
            return;
        }
        let policy = match policy.as_str() {
            "allow" => ClosePolicy::Allow,
            "require-confirmation" => ClosePolicy::RequireConfirmation,
            _ => {
                self.send_command_result(
                    meta,
                    CommandKind::SetClosePolicy,
                    false,
                    Some("close policy is invalid".to_owned()),
                    None,
                    cx,
                );
                return;
            }
        };
        if let Some(surface) = self.surfaces.get_mut(&meta.surface_id)
            && surface.close_policy != policy
        {
            surface.pending_close_request = None;
            surface.close_policy = policy;
        }
        self.send_command_result(meta, CommandKind::SetClosePolicy, true, None, None, cx);
    }
    fn resolve_close_request(&mut self, command: Command, cx: &mut Context<Self>) {
        let meta = command.meta;
        let CommandOperation::ResolveCloseRequest { request_id, allow } = command.operation else {
            return;
        };
        if meta.node_id != 1 {
            self.send_command_result(
                meta,
                CommandKind::ResolveCloseRequest,
                false,
                Some("resolveCloseRequest requires the root container".to_owned()),
                None,
                cx,
            );
            return;
        }
        let Some(surface) = self.surfaces.get_mut(&meta.surface_id) else {
            return;
        };
        if surface.pending_close_request != Some(request_id) {
            self.send_command_result(meta, CommandKind::ResolveCloseRequest, true, None, None, cx);
            return;
        }
        surface.pending_close_request = None;
        let window = surface.window;
        self.send_command_result(meta, CommandKind::ResolveCloseRequest, true, None, None, cx);
        if allow {
            cx.defer(move |cx| {
                let _ = window.update(cx, |_, window, _| window.remove_window());
            });
        }
    }
    fn open_initial(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        let surface_id = self.allocate_surface_id()?;
        debug_assert_eq!(surface_id, 1);
        let surface = self.open_window(None, 800, 600, None, cx)?;
        self.insert_surface(surface_id, surface);
        self.application_surface_id = Some(surface_id);
        Ok(())
    }

    fn activate_application(
        &mut self,
        reason: &'static str,
        urls: Vec<String>,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if self.transport_terminated {
            return Ok(());
        }
        self.application.enqueue(reason, urls)?;
        self.deliver_activations(cx)
    }

    fn deliver_activations(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        for activation in self.application.pending() {
            let surface_id = match self
                .application_surface_id
                .filter(|id| self.surfaces.contains_key(id))
            {
                Some(id) => id,
                None => {
                    let id = self.allocate_surface_id()?;
                    let surface = self.open_window(None, 800, 600, None, cx)?;
                    self.insert_surface(id, surface);
                    self.application_surface_id = Some(id);
                    id
                }
            };
            if activation.reason != "launch" {
                cx.activate(true);
                self.surfaces[&surface_id]
                    .window
                    .update(cx, |_, window, _| {
                        window.activate_window();
                    })
                    .map_err(|error| format!("failed to activate application window: {error}"))?;
            }
            let event = Event::new(
                crate::protocol::EventMeta {
                    surface_id: 0,
                    epoch: self.application.epoch,
                    revision: 0,
                    sequence: activation.sequence,
                    node_id: 0,
                    listener_id: 0,
                },
                crate::protocol::EventPayload::ApplicationActivation {
                    target_surface_id: surface_id,
                    reason: activation.reason.into(),
                    urls: activation.urls,
                },
            );
            if !send_event_or_exit(self.runtime.as_ref(), "application activation", event) {
                return Err("application activation transport terminated".into());
            }
            self.application.sent(activation.sequence);
        }
        Ok(())
    }

    fn insert_surface(&mut self, surface_id: u32, surface: Surface) {
        debug_assert!(!self.retired_surface_ids.contains(&surface_id));
        let window_id = surface.window.window_id();
        self.windows.insert(window_id, surface_id);
        self.surfaces.insert(surface_id, surface);
    }

    #[cfg(feature = "quickjs")]
    fn prepare_generation(
        &self,
        snapshots: &[crate::Snapshot],
        configuration: &Command,
        epoch: u32,
        cx: &Context<Self>,
    ) -> Result<ApplicationLifecycle, String> {
        let ids: std::collections::HashSet<_> = snapshots.iter().map(|s| s.surface_id).collect();
        if ids.len() != snapshots.len()
            || ids.len()
                != self
                    .surfaces
                    .keys()
                    .filter(|id| {
                        !self.popups.contains_key(id) && !self.retired_surface_ids.contains(id)
                    })
                    .count()
            || ids.iter().any(|id| {
                !self.surfaces.contains_key(id)
                    || self.popups.contains_key(id)
                    || self.retired_surface_ids.contains(id)
            })
        {
            return Err("candidate must replace exactly the current persistent Surface set".into());
        }
        for snapshot in snapshots {
            let surface = &self.surfaces[&snapshot.surface_id];
            surface
                .window
                .read(cx, |_: Entity<SolidRoot>, _| ())
                .map_err(|e| e.to_string())?;
            surface
                .root
                .read(cx)
                .validate_replacement(snapshot)
                .map_err(|e| e.to_string())?;
        }
        let CommandOperation::ConfigureApplication {
            keep_alive,
            quit,
            acknowledged_sequence,
        } = configuration.operation
        else {
            return Err("candidate configuration is invalid".into());
        };
        let mut application = self.application.clone();
        application.configure(epoch, keep_alive, quit, acknowledged_sequence)?;
        Ok(application)
    }

    #[cfg(feature = "quickjs")]
    fn replace_generation(
        &mut self,
        mut replacement: crate::runtime::reload::Replacement,
        cx: &mut Context<Self>,
    ) {
        let application = match self.prepare_generation(
            &replacement.snapshots,
            &replacement.configuration,
            replacement.epoch,
            cx,
        ) {
            Ok(application) => application,
            Err(error) => {
                replacement.reject(error);
                return;
            }
        };
        if let Err(error) = replacement.begin() {
            replacement.reject(error);
            return;
        }
        // No foreground suspension occurs between validation and installation.
        // Window identity and Rust services are retained; old epoch work is cancelled.
        for snapshot in std::mem::take(&mut replacement.snapshots) {
            self.apply_to_surface(DecodedMessage::Snapshot(snapshot), cx)
                .expect("validated generation installation on the same foreground turn");
        }
        self.application = application;
        replacement.finish();
        if let Err(error) = self.deliver_activations(cx) {
            eprintln!("reload activation: {error}");
        }
    }

    fn route_payload(&mut self, payload: &[u8], cx: &mut Context<Self>) -> Result<(), String> {
        let message = decode_message(payload)
            .map_err(|error| format!("rejected renderer commit: {error}"))?;
        let surface_id = match &message {
            DecodedMessage::Snapshot(snapshot) => snapshot.surface_id,
            DecodedMessage::Patch(patch) => patch.surface_id,
            DecodedMessage::Command(command) => command.meta.surface_id,
        };
        if self.retired_surface_ids.contains(&surface_id) {
            // Native dismissal can race the JS acknowledgement and initial
            // Snapshot. The late child root needs its own terminal event.
            if let DecodedMessage::Snapshot(snapshot) = message {
                send_event_or_exit(
                    self.runtime.as_ref(),
                    "retired surface closed event",
                    Event::surface_closed(surface_id, snapshot.epoch, snapshot.revision, 1),
                );
            }
            return Ok(());
        }
        match message {
            DecodedMessage::Snapshot(snapshot) => {
                self.apply_to_surface(DecodedMessage::Snapshot(snapshot), cx)
            }
            DecodedMessage::Patch(patch) => self.apply_to_surface(DecodedMessage::Patch(patch), cx),
            DecodedMessage::Command(command) => {
                if let CommandOperation::ConfigureApplication {
                    keep_alive,
                    quit,
                    acknowledged_sequence,
                } = command.operation
                {
                    if self.application.configure(
                        command.meta.epoch,
                        keep_alive,
                        quit,
                        acknowledged_sequence,
                    )? {
                        cx.quit();
                        return Ok(());
                    }
                    return self.deliver_activations(cx);
                }
                let kind = command.operation.kind();
                let capabilities = self.profile.capabilities();
                if kind == CommandKind::SetKeybindings && !capabilities.set_keybindings {
                    self.send_command_result(
                        command.meta,
                        kind,
                        false,
                        Some("SetKeybindings is unavailable in this host profile".to_owned()),
                        None,
                        cx,
                    );
                    return Ok(());
                }
                if let Some(error) = self.profile.rejected_command_reason(kind) {
                    self.send_command_result(
                        command.meta,
                        kind,
                        false,
                        Some(error.to_owned()),
                        None,
                        cx,
                    );
                    return Ok(());
                }
                if let CommandAdmission::Rejected(error) = self.admit_command(&command, cx)? {
                    self.send_command_result(command.meta, kind, false, Some(error), None, cx);
                    return Ok(());
                }
                match kind {
                    CommandKind::SetKeybindings => {
                        self.set_keybindings(command, cx);
                        Ok(())
                    }
                    CommandKind::SetClosePolicy => {
                        self.set_close_policy(command, cx);
                        Ok(())
                    }
                    CommandKind::ResolveCloseRequest => {
                        self.resolve_close_request(command, cx);
                        Ok(())
                    }
                    CommandKind::OpenSurface => self.open_surface(command, cx),
                    CommandKind::OpenPopup => self.open_popup(command, cx),
                    CommandKind::ClosePopup => self.close_popup_command(command, cx),
                    _ => self.apply_to_surface(DecodedMessage::Command(command), cx),
                }
            }
        }
    }
    fn admit_command(
        &self,
        command: &Command,
        cx: &mut Context<Self>,
    ) -> Result<CommandAdmission, String> {
        let surface = self
            .surfaces
            .get(&command.meta.surface_id)
            .ok_or_else(|| format!("unknown surface {}", command.meta.surface_id))?;
        let metadata = surface.root.read_with(cx, |root, _| {
            (
                root.store().surface_id(),
                root.store().epoch(),
                root.store().revision(),
            )
        });
        Ok(check_command_metadata(command.meta, metadata))
    }

    fn apply_to_surface(
        &mut self,
        message: DecodedMessage,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let surface_id = match &message {
            DecodedMessage::Snapshot(snapshot) => snapshot.surface_id,
            DecodedMessage::Patch(patch) => patch.surface_id,
            DecodedMessage::Command(command) => command.meta.surface_id,
        };
        let Some(surface) = self.surfaces.get(&surface_id) else {
            return Err(format!("unknown surface {surface_id}"));
        };
        let root = surface.root.clone();
        let previous_epoch = root.read(cx).store().epoch();
        let first_snapshot = previous_epoch == 0 && matches!(message, DecodedMessage::Snapshot(_));
        surface
            .window
            .update(cx, |_, window, cx| {
                root.update(cx, |root, cx| {
                    root.apply_decoded_message_in_window(message, window, cx)
                })
            })
            .map_err(|error| format!("surface {surface_id} window is unavailable: {error}"))?
            .map_err(|error| format!("surface {surface_id} rejected commit: {error}"))?;
        if first_snapshot {
            self.popup_received_snapshot(surface_id, cx);
        }
        if root.read(cx).store().epoch() != previous_epoch && previous_epoch != 0 {
            self.close_owned_popups(surface_id, cx);
        }
        if root.read(cx).store().epoch() != previous_epoch
            && self.keybindings.remove(&surface_id).is_some()
        {
            self.restore_keybindings(cx);
        }
        Ok(())
    }
    fn compile_keybindings(
        surface_id: u32,
        bindings: &[KeybindingDefinition],
    ) -> Result<Vec<KeyBinding>, String> {
        bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| {
                if binding.keystrokes.split_whitespace().next().is_none() {
                    return Err(format!("keybinding {surface_id}[{index}] has no keystroke"));
                }
                KeyBinding::load(
                    &binding.keystrokes,
                    Box::new(MenuAction {
                        name: binding.action_name.clone(),
                    }),
                    None,
                    false,
                    None,
                    &DummyKeyboardMapper,
                )
                .map_err(|error| {
                    format!("keybinding {surface_id}[{index}] invalid keystroke: {error}")
                })
            })
            .collect()
    }

    fn restore_keybindings(&self, cx: &mut App) {
        // GPUI's keymap is application-wide. Select the active window's map so
        // shortcuts also work without focused content and inside native overlays.
        // Keep compiled bindings: activation must never parse renderer input.
        let dynamic = cx
            .active_window()
            .and_then(|window| self.windows.get(&window.window_id()))
            .and_then(|id| self.keybindings.get(id))
            .cloned()
            .unwrap_or_default();
        self.profile
            .restore_keybindings(&self.baseline_keybindings, dynamic, cx);
    }

    fn set_keybindings(&mut self, command: Command, cx: &mut Context<Self>) {
        let meta = command.meta;
        let CommandOperation::SetKeybindings { bindings } = command.operation else {
            return;
        };
        if meta.node_id != 1 {
            self.send_command_result(
                meta,
                CommandKind::SetKeybindings,
                false,
                Some("setKeybindings requires the root container".to_owned()),
                None,
                cx,
            );
            return;
        }
        match Self::compile_keybindings(meta.surface_id, &bindings) {
            Ok(compiled) => {
                self.keybindings.insert(meta.surface_id, compiled);
                self.restore_keybindings(cx);
                self.send_command_result(meta, CommandKind::SetKeybindings, true, None, None, cx);
            }
            Err(error) => {
                self.send_command_result(
                    meta,
                    CommandKind::SetKeybindings,
                    false,
                    Some(error),
                    None,
                    cx,
                );
            }
        }
    }

    fn open_surface(&mut self, command: Command, cx: &mut Context<Self>) -> Result<(), String> {
        let meta = command.meta;
        let CommandOperation::OpenSurface {
            title,
            width,
            height,
            options,
        } = command.operation
        else {
            return Ok(());
        };
        if meta.node_id != 1 {
            self.send_command_result(
                meta,
                CommandKind::OpenSurface,
                false,
                Some("openSurface requires the root container".to_owned()),
                None,
                cx,
            );
            return Ok(());
        }
        let surface_id = self.allocate_surface_id()?;
        let (width, height) = if width == 0 && height == 0 {
            (800, 600)
        } else {
            (width, height)
        };
        match self.open_window(Some(&title), width, height, options.as_ref(), cx) {
            Ok(surface) => {
                self.insert_surface(surface_id, surface);
                self.send_command_result(
                    meta,
                    CommandKind::OpenSurface,
                    true,
                    None,
                    Some(CommandValue::Number(surface_id)),
                    cx,
                );
            }
            Err(error) => self.send_command_result(
                meta,
                CommandKind::OpenSurface,
                false,
                Some(error),
                None,
                cx,
            ),
        }
        Ok(())
    }

    fn send_command_result(
        &mut self,
        meta: CommandMeta,
        command: CommandKind,
        success: bool,
        error: Option<String>,
        value: Option<CommandValue>,
        cx: &mut Context<Self>,
    ) {
        let Some(surface) = self.surfaces.get(&meta.surface_id) else {
            return;
        };
        let root = surface.root.clone();
        root.update(cx, |root, _| {
            root.emit_command_ack(meta, command, success, error, value);
        });
    }

    fn emit_action(&mut self, action: String, cx: &mut Context<Self>) {
        let Some(window) = cx.active_window() else {
            return;
        };
        let Some(surface_id) = self.windows.get(&window.window_id()).copied() else {
            return;
        };
        let Some(surface) = self.surfaces.get(&surface_id) else {
            return;
        };
        let root = surface.root.clone();
        root.update(cx, |root, _| root.emit_action(action));
    }
    fn emit_notification_response(
        &mut self,
        response: SystemNotificationResponse,
        cx: &mut Context<Self>,
    ) {
        let tag = response.tag.to_string();
        let Some(surface_id) = tag
            .strip_prefix("solid-gpui:")
            .and_then(|value| value.split_once(':'))
            .and_then(|(surface_id, _)| surface_id.parse::<u32>().ok())
        else {
            return;
        };
        let Some(surface) = self.surfaces.get(&surface_id) else {
            return;
        };
        let root = surface.root.clone();
        let action_id = response.action_id.map(|action| action.to_string());
        root.update(cx, |root, _| {
            root.emit_notification_response(tag, action_id)
        });
    }

    fn window_closed(&mut self, window_id: WindowId, cx: &mut Context<Self>) -> bool {
        let Some(surface_id) = self.windows.get(&window_id).copied() else {
            return false;
        };
        self.close_owned_popups(surface_id, cx);
        self.release_popup(surface_id, cx);
        if !self.transport_terminated {
            let root = self
                .surfaces
                .get(&surface_id)
                .map(|surface| surface.root.clone());
            if let Some(root) = root {
                root.update(cx, |root, _| {
                    // An uninitialized child has no wire identity yet. A late
                    // Snapshot receives a terminal event from route_payload.
                    if root.store().epoch() != 0 {
                        root.emit_surface_closed();
                    }
                });
            }
        }
        self.windows.remove(&window_id);
        self.retired_surface_ids.insert(surface_id);
        self.surfaces.remove(&surface_id);
        self.keybindings.remove(&surface_id);
        self.restore_keybindings(cx);
        self.surfaces
            .keys()
            .all(|id| self.popups.contains_key(id) || self.retired_surface_ids.contains(id))
            && !self.application.keep_alive
    }
    fn close_all(&mut self, cx: &mut Context<Self>) {
        self.transport_terminated = true;
        // Removing a window invokes `on_window_closed` synchronously. Drop the
        // registry's callback before that update can re-enter this entity.
        self.close_subscription.take();
        let mut ids = self.surfaces.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable_by(|a, b| b.cmp(a));
        let windows = ids
            .into_iter()
            .map(|id| self.surfaces[&id].window)
            .collect::<Vec<_>>();
        self.popups.clear();
        self.keybindings.clear();
        self.profile
            .restore_keybindings(&self.baseline_keybindings, Vec::new(), cx);
        self.surfaces.clear();
        for window in windows {
            let _ = window.update(cx, |_, window, _| window.remove_window());
        }
    }
}

/// Run the host with the default, extension-free profile.
pub fn run_default() {
    run_with_profile(DefaultHostProfile);
}

/// Run the host with a provider profile.
pub fn run_with_profile<P: HostProfile>(profile: P) {
    if env::args_os()
        .skip(1)
        .eq([OsString::from("--export-native")])
    {
        match profile.native_bindings() {
            Ok(source) => {
                print!("{source}");
                return;
            }
            Err(error) => {
                eprintln!("native bindings: {error}");
                std::process::exit(2);
            }
        }
    }
    install_panic_hook();
    let log_level = resolve_log_level(env::var(LOG_ENV).ok().as_deref(), |reason| {
        eprintln!("solid-gpui-host: invalid {LOG_ENV}: {reason}; defaulting to error");
    });
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let action = match parse_host_args(&args) {
        Ok(action) => action,
        Err(error) => {
            host_log(log_level, LogLevel::Error, error);
            std::process::exit(2);
        }
    };
    let (mode, renderer_args, smoke_press) = match action {
        CliAction::Help => {
            print_help();
            return;
        }
        CliAction::Version => {
            print_version();
            return;
        }
        CliAction::Run {
            mode,
            renderer_args,
            smoke_press,
        } => (mode, renderer_args, smoke_press),
    };
    let runtime = match start_runtime(mode, &renderer_args, smoke_press) {
        Ok(runtime) => runtime,
        Err(error) => {
            host_log(log_level, LogLevel::Error, error);
            std::process::exit(1);
        }
    };
    host_log(
        log_level,
        LogLevel::Info,
        startup_diagnostic(
            mode,
            renderer_entry(mode, &renderer_args),
            std::process::id(),
        ),
    );

    run_profile(profile, runtime, log_level);
}

fn image_http_client() -> Result<Arc<dyn gpui::http_client::HttpClient>, String> {
    reqwest_client::ReqwestClient::proxy_user_agent_and_read_timeout(
        None,
        concat!("solid-gpui/", env!("CARGO_PKG_VERSION")),
        Some(std::time::Duration::from_secs(15)),
    )
    .map(|client| Arc::new(client) as Arc<dyn gpui::http_client::HttpClient>)
    .map_err(|error| error.to_string())
}

#[test]
fn image_http_client_performs_a_real_local_request() {
    use futures::AsyncReadExt as _;
    use std::io::{Read as _, Write as _};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let (mut connection, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(5))
                }
                Err(error) => panic!("HTTP fixture was not requested: {error}"),
            }
        };
        connection.set_nonblocking(false).unwrap();
        connection
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        let mut request = [0; 8192];
        let mut count = 0;
        while !request[..count]
            .windows(4)
            .any(|bytes| bytes == b"\r\n\r\n")
        {
            assert!(
                count < request.len(),
                "fixture request headers exceed the bound"
            );
            let read = connection.read(&mut request[count..]).unwrap();
            assert!(read > 0, "fixture request ended before headers");
            count += read;
        }
        assert!(request[..count].starts_with(b"GET /fixture.png HTTP/1.1"));
        connection
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nimage")
            .unwrap();
    });
    let client = image_http_client().unwrap();
    let body = futures::executor::block_on(async {
        let mut response = client
            .get(&format!("http://{address}/fixture.png"), ().into(), true)
            .await
            .unwrap();
        let mut body = Vec::new();
        response.body_mut().read_to_end(&mut body).await.unwrap();
        body
    });
    server.join().unwrap();
    assert_eq!(body, b"image");
}

fn run_profile<P: HostProfile>(
    mut profile: P,
    runtime: Arc<dyn RuntimeAdapter>,
    log_level: LogLevel,
) {
    let runtime_for_quit = Arc::clone(&runtime);
    let runtime_for_registry = Arc::clone(&runtime);
    let log_level_for_quit = log_level;
    let (activation_sender, mut activations) = futures::channel::mpsc::channel(32);
    let mut urls_sender = activation_sender.clone();
    let mut reopen_sender = activation_sender;
    let http_client = image_http_client().unwrap_or_else(|error| {
        fatal_runtime_failure(
            runtime.as_ref(),
            "failed to initialize image HTTP client",
            error,
        )
    });
    let application = gpui_platform::application()
        .with_assets(HostAssets)
        .with_http_client(http_client);
    application.on_open_urls(move |urls| {
        if let Err(error) = urls_sender.try_send(("open-urls", urls)) {
            eprintln!("solid-gpui-host: activation rejected: {error}");
        }
    });
    application.on_reopen(move |_| {
        if let Err(error) = reopen_sender.try_send(("reopen", Vec::new())) {
            eprintln!("solid-gpui-host: activation rejected: {error}");
        }
    });
    application.run(move |cx: &mut App| {
        let capabilities = profile.capabilities();
        motion::initialize(cx);
        profile.initialize(cx);
        let baseline_keybindings = cx.key_bindings().borrow().bindings().cloned().collect();
        let registry = cx.new(|_| {
            NativeStateRegistry::with_profile(runtime_for_registry, profile, baseline_keybindings)
        });
        if let Err(error) = registry.update(cx, |registry, cx| registry.open_initial(cx)) {
            host_log(log_level, LogLevel::Error, error);
            let _ = runtime_for_quit.request_shutdown();
            std::process::exit(1);
        }
        let registry_for_activation = registry.downgrade();
        cx.spawn(async move |cx| {
            use futures::StreamExt;
            while let Some((reason, urls)) = activations.next().await {
                let Some(registry) = registry_for_activation.upgrade() else {
                    break;
                };
                if let Err(error) = registry.update(cx, |registry, cx| {
                    registry.activate_application(reason, urls, cx)
                }) {
                    eprintln!("solid-gpui-host: activation rejected: {error}");
                }
            }
        })
        .detach();
        let registry_for_action = registry.downgrade();
        cx.on_action(move |action: &MenuAction, cx| {
            if let Some(registry) = registry_for_action.upgrade() {
                registry.update(cx, |registry, cx| {
                    registry.emit_action(action.name.clone(), cx)
                });
            }
        });
        if capabilities.notification_responses {
            let registry_for_notification = registry.downgrade();
            cx.on_system_notification_response(move |response, cx| {
                if let Some(registry) = registry_for_notification.upgrade() {
                    registry.update(cx, |registry, cx| {
                        registry.emit_notification_response(response, cx)
                    });
                }
            });
        }

        let registry_for_close = registry.downgrade();
        let close_subscription = cx.on_window_closed(move |cx, window_id| {
            if let Some(registry) = registry_for_close.upgrade() {
                let should_quit =
                    registry.update(cx, |registry, cx| registry.window_closed(window_id, cx));
                if should_quit {
                    cx.quit();
                }
            }
        });
        registry.update(cx, |registry, _| {
            registry.close_subscription = Some(close_subscription);
        });

        let pump = CommitPump::start(Arc::clone(&runtime)).unwrap_or_else(|error| {
            fatal_runtime_failure(runtime.as_ref(), "failed to start commit pump", error)
        });
        pump.attach(registry, Arc::clone(&runtime), cx);
        cx.on_app_quit(move |cx| {
            let runtime = Arc::clone(&runtime_for_quit);
            cx.background_spawn(async move {
                let _ = runtime.shutdown();
                host_log(
                    log_level_for_quit,
                    LogLevel::Info,
                    format!("runtime terminated status={:?}", runtime.status()),
                );
            })
        })
        .detach();
        cx.activate(true);
    });
}

#[derive(Debug, Eq, PartialEq)]
enum CliAction {
    Help,
    Version,
    Run {
        mode: RuntimeMode,
        renderer_args: Vec<OsString>,
        smoke_press: bool,
    },
}

fn parse_host_args(args: &[OsString]) -> Result<CliAction, String> {
    let mut mode = RuntimeMode::Process;
    let mut renderer_args = Vec::new();
    let mut smoke_press = false;
    let mut host_options = true;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if !host_options {
            renderer_args.push(arg.clone());
            index += 1;
            continue;
        }

        if arg == "--" {
            host_options = false;
        } else if arg == "--help" || arg == "-h" {
            return Ok(CliAction::Help);
        } else if arg == "--version" || arg == "-V" {
            return Ok(CliAction::Version);
        } else if arg == "--runtime" {
            index += 1;
            let Some(value) = args.get(index) else {
                return Err("--runtime requires `process`, `embedded`, or `quickjs`".to_owned());
            };
            mode = match value.to_string_lossy().as_ref() {
                "process" => RuntimeMode::Process,
                "embedded" => RuntimeMode::Embedded,
                "quickjs" => RuntimeMode::QuickJs,
                "quickjs-dev" => RuntimeMode::QuickJsDev,
                value => return Err(format!("unknown runtime `{value}`")),
            };
        } else if arg == "--embedded" {
            mode = RuntimeMode::Embedded;
        } else if arg == "--smoke-press" {
            smoke_press = true;
        } else if arg.to_string_lossy().starts_with('-') {
            return Err(format!("unknown host option `{}`", arg.to_string_lossy()));
        } else {
            host_options = false;
            renderer_args.push(arg.clone());
        }
        index += 1;
    }

    if smoke_press && mode != RuntimeMode::Embedded {
        return Err("--smoke-press requires `--runtime embedded`".to_owned());
    }

    if mode == RuntimeMode::Embedded && renderer_args.is_empty() {
        return Err("embedded runtime requires an explicit application entry".to_owned());
    }
    if mode == RuntimeMode::QuickJsDev && renderer_args.len() != 2 {
        return Err("quickjs-dev requires a bundle and loopback tooling endpoint".into());
    }
    if mode == RuntimeMode::QuickJs && renderer_args.len() != 1 {
        return Err("quickjs runtime requires exactly one bundled JavaScript entry".to_owned());
    }

    Ok(CliAction::Run {
        mode,
        renderer_args,
        smoke_press,
    })
}

fn renderer_entry(mode: RuntimeMode, renderer_args: &[OsString]) -> String {
    match mode {
        RuntimeMode::Process => renderer_args
            .first()
            .map(|arg| arg.to_string_lossy().into_owned())
            .or_else(|| env::var(COMMAND_ENV).ok())
            .unwrap_or_else(|| "bun".to_owned()),
        RuntimeMode::Embedded | RuntimeMode::QuickJs | RuntimeMode::QuickJsDev => renderer_args
            .first()
            .map(|arg| arg.to_string_lossy().into_owned())
            .expect("embedded entry validated by CLI"),
    }
}

fn start_runtime(
    mode: RuntimeMode,
    renderer_args: &[OsString],
    smoke_press: bool,
) -> Result<Arc<dyn RuntimeAdapter>, String> {
    match mode {
        RuntimeMode::Process => ProcessAdapter::spawn(renderer_command(renderer_args))
            .map(|runtime| runtime as Arc<dyn RuntimeAdapter>)
            .map_err(|error| format!("failed to spawn process renderer: {error}")),
        RuntimeMode::Embedded => start_embedded(renderer_args, smoke_press),
        RuntimeMode::QuickJsDev => {
            #[cfg(feature = "quickjs")]
            {
                crate::runtime::reload::ReloadableQuickJs::start(
                    &renderer_args[0].to_string_lossy(),
                    &renderer_args[1].to_string_lossy(),
                )
                .map(|r| r as Arc<dyn RuntimeAdapter>)
            }
            #[cfg(not(feature = "quickjs"))]
            {
                Err("QuickJS runtime is not compiled; use --features quickjs".into())
            }
        }
        RuntimeMode::QuickJs => {
            #[cfg(feature = "quickjs")]
            {
                crate::runtime::quickjs::QuickJsAdapter::start(&renderer_args[0])
                    .map(|runtime| runtime as Arc<dyn RuntimeAdapter>)
                    .map_err(|error| format!("failed to start QuickJS renderer: {error}"))
            }
            #[cfg(not(feature = "quickjs"))]
            {
                Err("QuickJS runtime is not compiled; use `--features quickjs`".to_owned())
            }
        }
    }
}

fn start_embedded(
    renderer_args: &[OsString],
    smoke_press: bool,
) -> Result<Arc<dyn RuntimeAdapter>, String> {
    #[cfg(feature = "embedded-bun")]
    {
        let entry = renderer_args
            .first()
            .map(PathBuf::from)
            .ok_or_else(|| "embedded runtime requires an explicit application entry".to_owned())?;
        let runtime = EmbeddedBunAdapter::start(&entry)
            .map_err(|error| format!("failed to start embedded Bun: {error}"))?;
        if smoke_press {
            let runtime_for_smoke = Arc::clone(&runtime);
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(2000));
                let sent = send_event_or_exit(
                    runtime_for_smoke.as_ref(),
                    "embedded smoke press event",
                    Event::press(1, 1, 1, 1, 7, 1),
                );
                std::thread::sleep(std::time::Duration::from_millis(2000));
                eprintln!(
                    "solid-gpui-host: embedded smoke press sent={sent}, commits={}, status={:?}",
                    runtime_for_smoke.commit_count(),
                    runtime_for_smoke.runtime_status()
                );
            });
        }
        Ok(runtime as Arc<dyn RuntimeAdapter>)
    }
    #[cfg(not(feature = "embedded-bun"))]
    {
        let _ = (renderer_args, smoke_press);
        Err("embedded runtime is not compiled; use `--features embedded-bun`".to_owned())
    }
}

fn renderer_command(renderer_args: &[OsString]) -> ProcessCommand {
    let mut command = renderer_args
        .first()
        .cloned()
        .map(ProcessCommand::new)
        .unwrap_or_else(|| {
            let executable = env::var_os(COMMAND_ENV).unwrap_or_else(|| "bun".into());
            ProcessCommand::new(executable)
        });
    if renderer_args.len() > 1 {
        command.args(&renderer_args[1..]);
    } else if renderer_args.is_empty()
        && let Some(configured_args) = env::var_os(ARGS_ENV)
    {
        command.args(configured_args.to_string_lossy().split_whitespace());
    }
    command
}
fn startup_diagnostic(mode: RuntimeMode, entry: String, pid: u32) -> String {
    format!("starting mode={mode:?} protocol=v{PROTOCOL_VERSION} entry={entry} pid={pid}")
}

fn version_line() -> String {
    format!(
        "solid-gpui-host {} protocol=v{PROTOCOL_VERSION}",
        env!("CARGO_PKG_VERSION")
    )
}

fn print_version() {
    println!("{}", version_line());
}

fn print_help() {
    println!(
        "solid-gpui-host\n\nUsage:\n  solid-gpui-host [host-options] [renderer-command [args...]]\n  solid-gpui-host --runtime embedded entry.ts\n  solid-gpui-host --runtime quickjs app.js\n\nHost options:\n  -h, --help           print this help without starting GPUI\n  -V, --version        print the package version without starting GPUI\n  --runtime process    child-process ProcessAdapter (default)\n  --runtime embedded   in-process Bun/JSC adapter (build with --features embedded-bun)\n  --runtime quickjs-dev bundle endpoint    QuickJS development generation supervisor\n  --runtime quickjs    embedded QuickJS for a bundled JS entry (build with --features quickjs)\n  --embedded           alias for --runtime embedded\n  --smoke-press        send one embedded pointer press before waiting\n  --                  stop host option parsing and run the renderer command",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn version_and_startup_diagnostics_include_protocol_version() {
        assert_eq!(
            version_line(),
            format!(
                "solid-gpui-host {} protocol=v{}",
                env!("CARGO_PKG_VERSION"),
                PROTOCOL_VERSION
            )
        );
        assert_eq!(
            startup_diagnostic(RuntimeMode::Process, "bun".to_owned(), 42),
            "starting mode=Process protocol=v5 entry=bun pid=42"
        );
    }

    #[test]
    fn parses_version_before_renderer_command() {
        assert_eq!(
            parse_host_args(&args(&["--version"])),
            Ok(CliAction::Version)
        );
    }

    #[test]
    fn parses_help_before_renderer_command() {
        assert_eq!(parse_host_args(&args(&["--help"])), Ok(CliAction::Help));
    }

    #[test]
    fn parses_embedded_runtime_entry() {
        assert!(parse_host_args(&args(&["--runtime", "embedded"])).is_err());
        assert_eq!(
            parse_host_args(&args(&["--runtime", "embedded", "entry.ts"])),
            Ok(CliAction::Run {
                mode: RuntimeMode::Embedded,
                renderer_args: args(&["entry.ts"]),
                smoke_press: false,
            })
        );
        assert!(parse_host_args(&args(&["--runtime", "quickjs"])).is_err());
        assert!(parse_host_args(&args(&["--runtime", "quickjs", "one.js", "two.js"])).is_err());
        assert_eq!(
            parse_host_args(&args(&["--runtime", "quickjs", "app.js"])),
            Ok(CliAction::Run {
                mode: RuntimeMode::QuickJs,
                renderer_args: args(&["app.js"]),
                smoke_press: false,
            })
        );
    }

    #[test]
    fn separator_preserves_renderer_help_flag() {
        assert_eq!(
            parse_host_args(&args(&["--", "bun", "run", "app.ts", "--help"])),
            Ok(CliAction::Run {
                mode: RuntimeMode::Process,
                renderer_args: args(&["bun", "run", "app.ts", "--help"]),
                smoke_press: false,
            })
        );
    }

    #[test]
    fn renderer_command_stops_host_option_parsing() {
        assert_eq!(
            parse_host_args(&args(&["bun", "run", "app.ts", "--version"])),
            Ok(CliAction::Run {
                mode: RuntimeMode::Process,
                renderer_args: args(&["bun", "run", "app.ts", "--version"]),
                smoke_press: false,
            })
        );
    }

    #[test]
    fn embedded_smoke_press_is_not_a_process_renderer_command() {
        assert_eq!(
            parse_host_args(&args(&["--smoke-press"])),
            Err("--smoke-press requires `--runtime embedded`".to_owned())
        );
        assert_eq!(
            parse_host_args(&args(&[
                "--runtime",
                "embedded",
                "--smoke-press",
                "entry.ts"
            ])),
            Ok(CliAction::Run {
                mode: RuntimeMode::Embedded,
                renderer_args: args(&["entry.ts"]),
                smoke_press: true,
            })
        );
    }

    #[test]
    fn unknown_host_option_is_rejected_before_renderer_command() {
        assert_eq!(
            parse_host_args(&args(&["--not-a-host-option"])),
            Err("unknown host option `--not-a-host-option`".to_owned())
        );
    }

    #[test]
    fn parses_log_levels_case_insensitively() {
        assert_eq!(parse_log_level(None), Ok(LogLevel::Error));
        assert_eq!(parse_log_level(Some("off")), Ok(LogLevel::Off));
        assert_eq!(parse_log_level(Some("INFO")), Ok(LogLevel::Info));
        assert_eq!(parse_log_level(Some("Debug")), Ok(LogLevel::Debug));
    }

    #[test]
    fn invalid_log_level_falls_back_to_error_and_warns_once() {
        let mut warnings = 0;
        let level = resolve_log_level(Some("verbose"), |_| warnings += 1);
        assert_eq!(level, LogLevel::Error);
        assert_eq!(warnings, 1);
    }

    #[test]
    fn panic_hook_writes_crash_report_with_location() {
        let directory =
            env::temp_dir().join(format!("solid-gpui-host-panic-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        install_panic_hook_in(directory.clone());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            panic!("panic hook test");
        }));
        assert!(result.is_err());
        let report = fs::read_dir(&directory)
            .expect("crash report directory")
            .find_map(|entry| {
                let path = entry.ok()?.path();
                path.extension().filter(|ext| *ext == "log")?;
                Some(fs::read_to_string(path).expect("crash report contents"))
            })
            .expect("panic crash report");
        assert!(report.contains("panic: panic hook test"));
        assert!(report.contains("location:"));
        assert!(report.contains("backtrace:"));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn command_admission_centralizes_surface_epoch_and_revision_checks() {
        let meta = CommandMeta {
            surface_id: 7,
            epoch: 3,
            after_revision: 42,
            request_id: 1,
            node_id: 1,
        };
        assert_eq!(
            check_command_metadata(meta, (7, 3, 42)),
            CommandAdmission::Accepted
        );
        assert_eq!(
            check_command_metadata(meta, (8, 3, 42)),
            CommandAdmission::Rejected("surface or epoch mismatch".to_owned())
        );
        assert_eq!(
            check_command_metadata(meta, (7, 3, 41)),
            CommandAdmission::Rejected("command revision is stale".to_owned())
        );
    }
    #[test]
    fn surface_ids_are_host_allocated_without_implicit_surface_entries() {
        let mut registry = NativeStateRegistry::new(crate::InMemoryAdapter::new());
        assert_eq!(registry.allocate_surface_id(), Ok(1));
        assert_eq!(registry.allocate_surface_id(), Ok(2));
        assert!(registry.surfaces.is_empty());
    }

    #[test]
    fn surface_id_allocator_rejects_exhaustion() {
        let mut registry = NativeStateRegistry::new(crate::InMemoryAdapter::new());
        registry.next_surface_id = u32::MAX;
        assert_eq!(
            registry.allocate_surface_id(),
            Err("surface id space exhausted".to_owned())
        );
    }
    #[test]
    fn retired_surface_ids_are_never_reallocated() {
        let mut registry = NativeStateRegistry::new(crate::InMemoryAdapter::new());
        registry.retired_surface_ids.insert(1);
        assert_eq!(registry.allocate_surface_id(), Ok(2));
    }
}

#[cfg(test)]
mod icon_asset_tests {
    use gpui::AssetSource;

    #[test]
    fn icon_assets_resolve_compiled_svg_path() {
        let assets = super::HostAssets;
        let bytes = assets
            .load("iconify/lucide/play.svg")
            .expect("icon asset lookup should succeed")
            .expect("compiled play icon should resolve");
        assert!(bytes.starts_with(b"<svg"), "icon asset is not SVG data");
        #[cfg(feature = "gpui-component")]
        {
            let native_icons = assets.list("icons/").unwrap();
            assert!(
                !native_icons.is_empty(),
                "native control icons must be installed"
            );
            for path in native_icons {
                let bytes = assets.load(&path).unwrap().expect("native icon bytes");
                assert!(
                    std::str::from_utf8(&bytes).unwrap().contains("<svg"),
                    "invalid icon {path}"
                );
            }
        }
    }
}

/// Run an application's Rust module, exporting the exact linked contract with --export-native.
pub fn run(module: crate::native::ModuleDefinition) {
    run_with_profile(application_profile(module));
}

/// Run an application-owned runtime without interpreting host CLI arguments.
/// The host owns shutdown and joins the runtime when the application quits.
pub fn run_application(module: crate::native::ModuleDefinition, runtime: Arc<dyn RuntimeAdapter>) {
    run_application_with_profile(application_profile(module), runtime);
}

/// Run an application-owned profile and runtime without parsing CLI arguments.
/// Protocols, overlays, close handling and runtime shutdown remain host-owned.
pub fn run_application_with_profile<P: HostProfile>(profile: P, runtime: Arc<dyn RuntimeAdapter>) {
    install_panic_hook();
    let log_level = resolve_log_level(env::var(LOG_ENV).ok().as_deref(), |reason| {
        eprintln!("solid-gpui-host: invalid {LOG_ENV}: {reason}; defaulting to error");
    });
    run_profile(profile, runtime, log_level);
}

fn application_profile(module: crate::native::ModuleDefinition) -> impl HostProfile {
    #[cfg(feature = "gpui-component")]
    {
        crate::components::host::ComponentHost::new(vec![
            crate::components::native_module(),
            module,
        ])
    }
    #[cfg(not(feature = "gpui-component"))]
    {
        NativeHostProfile(Rc::new(crate::native::NativeModules::new(vec![module])))
    }
}

#[cfg(not(feature = "gpui-component"))]
struct NativeHostProfile(Rc<crate::native::NativeModules>);
#[cfg(not(feature = "gpui-component"))]
impl HostProfile for NativeHostProfile {
    fn native_bindings(&self) -> Result<String, String> {
        self.0
            .typescript()
            .map(|source| format!("{source}{}", crate::icons::typescript()))
    }
    fn capabilities(&self) -> HostCapabilities {
        DefaultHostProfile.capabilities()
    }
    fn extension_registry(&self) -> Rc<dyn ExtensionRegistry> {
        self.0.clone()
    }
    fn initialize(&mut self, cx: &mut App) {
        DefaultHostProfile.initialize(cx);
    }
    fn open_window(
        &self,
        options: WindowOptions,
        runtime: Arc<dyn RuntimeAdapter>,
        extensions: Rc<dyn ExtensionRegistry>,
        cx: &mut App,
    ) -> Result<(AnyWindowHandle, Entity<SolidRoot>), String> {
        DefaultHostProfile.open_window(options, runtime, extensions, cx)
    }
    fn restore_keybindings(&self, baseline: &[KeyBinding], dynamic: Vec<KeyBinding>, cx: &mut App) {
        DefaultHostProfile.restore_keybindings(baseline, dynamic, cx);
    }
}
