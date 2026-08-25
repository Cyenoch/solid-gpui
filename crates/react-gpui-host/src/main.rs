use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gpui::{
    App, AppContext, Bounds, Context, Entity, Subscription, TitlebarOptions, WindowBounds,
    WindowHandle, WindowId, WindowOptions, px, size,
};
use react_gpui::{
    COMMAND_OPEN_SURFACE, Command, CommandValue, MenuAction, Patch, ProcessAdapter, ProtocolError,
    ReactRoot, RuntimeAdapter, RuntimeStatus, Snapshot, fatal_runtime_failure,
};
#[cfg(feature = "embedded-bun")]
use react_gpui::{Event, send_event_or_exit};
#[cfg(feature = "embedded-bun")]
use react_gpui_bun::EmbeddedBunAdapter;
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fmt::Display;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
const CRASH_DIR_ENV: &str = "REACT_GPUI_CRASH_DIR";
const COMMAND_ENV: &str = "REACT_GPUI_RENDERER_COMMAND";
const ARGS_ENV: &str = "REACT_GPUI_RENDERER_ARGS";
const LOG_ENV: &str = "REACT_GPUI_LOG";
const DEFAULT_EMBEDDED_ENTRY: &str = "packages/react-gpui/examples/counter.tsx";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeMode {
    Process,
    Embedded,
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
        eprintln!("react-gpui-host: {message}");
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
        if let Err(error) = write_crash_report(&crash_dir, info) {
            eprintln!("react-gpui-host: unable to write crash report: {error}");
        }
    }));
}

fn write_crash_report(
    crash_dir: &std::path::Path,
    info: &panic::PanicHookInfo<'_>,
) -> std::io::Result<()> {
    fs::create_dir_all(crash_dir)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let path = crash_dir.join(format!(
        "react-gpui-host-{}-{timestamp}.log",
        std::process::id()
    ));
    let mut report = OpenOptions::new().create_new(true).write(true).open(path)?;
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
    writeln!(report, "react-gpui-host crash report")?;
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
    report.flush()
}

enum ReaderMessage {
    Payload(Vec<u8>),
    Terminated(RuntimeStatus),
    Transport(ProtocolError),
}

struct Surface {
    window: WindowHandle<ReactRoot>,
    root: Entity<ReactRoot>,
}

struct SurfaceRegistry {
    runtime: Arc<dyn RuntimeAdapter>,
    surfaces: HashMap<u32, Surface>,
    windows: HashMap<WindowId, u32>,
    next_surface_id: u32,
    transport_terminated: bool,
    close_subscription: Option<Subscription>,
}

impl SurfaceRegistry {
    fn new(runtime: Arc<dyn RuntimeAdapter>) -> Self {
        Self {
            runtime,
            surfaces: HashMap::new(),
            windows: HashMap::new(),
            next_surface_id: 1,
            transport_terminated: false,
            close_subscription: None,
        }
    }

    fn allocate_surface_id(&mut self) -> Result<u32, String> {
        let id = self.next_surface_id;
        if id == 0 {
            return Err("surface id space exhausted".to_owned());
        }
        self.next_surface_id = id
            .checked_add(1)
            .ok_or_else(|| "surface id space exhausted".to_owned())?;
        Ok(id)
    }

    fn open_window(
        &self,
        title: Option<&str>,
        width: u32,
        height: u32,
        cx: &mut Context<Self>,
    ) -> Result<Surface, String> {
        let bounds = Bounds::centered(None, size(px(width as f32), px(height as f32)), cx);
        let mut options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };
        if let Some(title) = title {
            options.titlebar = Some(TitlebarOptions {
                title: Some(title.to_owned().into()),
                ..Default::default()
            });
        }
        let runtime = Arc::clone(&self.runtime);
        let mut root = None;
        let window = cx
            .open_window(options, |_, cx| {
                let entity = cx.new(|_| ReactRoot::new(runtime));
                root = Some(entity.clone());
                entity
            })
            .map_err(|error| format!("failed to open GPUI window: {error}"))?;
        let root = root.ok_or_else(|| "GPUI did not return a root entity".to_owned())?;
        Ok(Surface { window, root })
    }

    fn open_initial(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        let surface_id = self.allocate_surface_id()?;
        debug_assert_eq!(surface_id, 1);
        let surface = self.open_window(None, 800, 600, cx)?;
        self.insert_surface(surface_id, surface);
        Ok(())
    }

    fn insert_surface(&mut self, surface_id: u32, surface: Surface) {
        let window_id = surface.window.window_id();
        self.windows.insert(window_id, surface_id);
        self.surfaces.insert(surface_id, surface);
    }

    fn route_payload(&mut self, payload: &[u8], cx: &mut Context<Self>) -> Result<(), String> {
        if let Ok(snapshot) = Snapshot::decode(payload) {
            return self.apply_to_surface(snapshot.surface_id, payload, cx);
        }
        if let Ok(patch) = Patch::decode(payload) {
            return self.apply_to_surface(patch.surface_id, payload, cx);
        }
        let command = Command::decode(payload)
            .map_err(|error| format!("rejected renderer commit: {error}"))?;
        if !self.surfaces.contains_key(&command.surface_id) {
            return Err(format!("unknown surface {}", command.surface_id));
        }
        if command.kind == COMMAND_OPEN_SURFACE {
            self.open_surface(command, cx)
        } else {
            self.apply_to_surface(command.surface_id, payload, cx)
        }
    }

    fn apply_to_surface(
        &mut self,
        surface_id: u32,
        payload: &[u8],
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let Some(surface) = self.surfaces.get(&surface_id) else {
            return Err(format!("unknown surface {surface_id}"));
        };
        let root = surface.root.clone();
        root.update(cx, |root, cx| root.apply_payload(payload, cx))
            .map_err(|error| format!("surface {surface_id} rejected commit: {error}"))
    }

    fn open_surface(&mut self, command: Command, cx: &mut Context<Self>) -> Result<(), String> {
        let title = command
            .title
            .as_deref()
            .ok_or_else(|| "open-surface command is missing a title".to_owned())?;
        let (width, height) = command
            .payload
            .ok_or_else(|| "open-surface command is missing a size pair".to_owned())?;
        let (width, height) = if width == 0 && height == 0 {
            (800, 600)
        } else {
            (width, height)
        };
        let surface_id = self.allocate_surface_id()?;
        let opened = self.open_window(Some(title), width, height, cx);
        match opened {
            Ok(surface) => {
                self.insert_surface(surface_id, surface);
                self.send_command_result(
                    &command,
                    true,
                    None,
                    Some(CommandValue::Number(surface_id as f32)),
                    cx,
                );
            }
            Err(error) => {
                self.send_command_result(&command, false, Some(error), None, cx);
            }
        }
        Ok(())
    }

    fn send_command_result(
        &mut self,
        command: &Command,
        success: bool,
        error: Option<String>,
        value: Option<CommandValue>,
        cx: &mut Context<Self>,
    ) {
        let Some(surface) = self.surfaces.get(&command.surface_id) else {
            return;
        };
        let root = surface.root.clone();
        let request_id = command.request_id;
        let command_kind = command.kind;
        let node_id = command.node_id;
        root.update(cx, |root, _| {
            root.emit_command_ack(request_id, command_kind, node_id, success, error, value);
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

    fn window_closed(&mut self, window_id: WindowId, cx: &mut Context<Self>) -> bool {
        let Some(surface_id) = self.windows.get(&window_id).copied() else {
            return false;
        };
        if !self.transport_terminated {
            let root = self
                .surfaces
                .get(&surface_id)
                .map(|surface| surface.root.clone());
            if let Some(root) = root {
                root.update(cx, |root, _| root.emit_surface_closed());
            }
        }
        self.windows.remove(&window_id);
        self.surfaces.remove(&surface_id);
        self.surfaces.is_empty()
    }
    fn close_all(&mut self, cx: &mut Context<Self>) {
        self.transport_terminated = true;
        let windows = self
            .surfaces
            .values()
            .map(|surface| surface.window)
            .collect::<Vec<_>>();
        self.windows.clear();
        self.surfaces.clear();
        for window in windows {
            let _ = window.update(cx, |_, window, _| window.remove_window());
        }
    }
}

fn start_commit_reader(
    registry: Entity<SurfaceRegistry>,
    runtime: Arc<dyn RuntimeAdapter>,
    cx: &App,
) {
    let (mut sender, mut receiver) = mpsc::channel::<ReaderMessage>(32);
    let runtime_for_reader = Arc::clone(&runtime);
    thread::Builder::new()
        .name("react-gpui-host-commit-reader".to_owned())
        .spawn(move || {
            loop {
                match runtime_for_reader.recv_commit() {
                    Ok(Some(payload)) => {
                        runtime_for_reader.tap_inbound_payload(&payload);
                        if futures::executor::block_on(sender.send(ReaderMessage::Payload(payload)))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Ok(None) => {
                        let status = match runtime_for_reader.status() {
                            RuntimeStatus::Running => RuntimeStatus::Exited {
                                code: None,
                                signal: None,
                            },
                            status => status,
                        };
                        let _ = futures::executor::block_on(
                            sender.send(ReaderMessage::Terminated(status)),
                        );
                        break;
                    }
                    Err(error) => {
                        let _ = futures::executor::block_on(
                            sender.send(ReaderMessage::Transport(error)),
                        );
                        break;
                    }
                }
            }
        })
        .expect("failed to start React GPUI host commit reader");

    cx.spawn(async move |cx| {
        while let Some(message) = receiver.next().await {
            match message {
                ReaderMessage::Payload(payload) => {
                    let result =
                        registry.update(cx, |registry, cx| registry.route_payload(&payload, cx));
                    if let Err(error) = result {
                        fatal_runtime_failure(runtime.as_ref(), "rejected renderer commit", error);
                    }
                }
                ReaderMessage::Terminated(status) => {
                    registry.update(cx, |registry, cx| registry.close_all(cx));
                    if status.is_failure() {
                        fatal_runtime_failure(
                            runtime.as_ref(),
                            "renderer runtime terminated unexpectedly",
                            status,
                        );
                    }
                    break;
                }
                ReaderMessage::Transport(error) => {
                    registry.update(cx, |registry, cx| registry.close_all(cx));
                    fatal_runtime_failure(runtime.as_ref(), "renderer transport error", error);
                }
            }
        }
    })
    .detach();
}

fn main() {
    install_panic_hook();
    let log_level = resolve_log_level(env::var(LOG_ENV).ok().as_deref(), |reason| {
        eprintln!("react-gpui-host: invalid {LOG_ENV}: {reason}; defaulting to error");
    });
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let action = match parse_host_args(&args) {
        Ok(action) => action,
        Err(error) => {
            host_log(log_level, LogLevel::Error, error);
            std::process::exit(2);
        }
    };
    let (mode, renderer_args) = match action {
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
        } => (mode, renderer_args),
    };
    let runtime = match start_runtime(mode, &renderer_args) {
        Ok(runtime) => runtime,
        Err(error) => {
            host_log(log_level, LogLevel::Error, error);
            std::process::exit(1);
        }
    };
    host_log(
        log_level,
        LogLevel::Info,
        format!(
            "starting mode={mode:?} entry={} pid={}",
            renderer_entry(mode, &renderer_args),
            std::process::id()
        ),
    );

    let runtime_for_quit = Arc::clone(&runtime);
    let runtime_for_registry = Arc::clone(&runtime);
    let log_level_for_quit = log_level;
    gpui_platform::application().run(move |cx: &mut App| {
        let registry = cx.new(|_| SurfaceRegistry::new(runtime_for_registry));
        if let Err(error) = registry.update(cx, |registry, cx| registry.open_initial(cx)) {
            host_log(log_level, LogLevel::Error, error);
            let _ = runtime_for_quit.shutdown();
            std::process::exit(1);
        }
        let registry_for_action = registry.downgrade();
        cx.on_action(move |action: &MenuAction, cx| {
            if let Some(registry) = registry_for_action.upgrade() {
                registry.update(cx, |registry, cx| {
                    registry.emit_action(action.name.clone(), cx)
                });
            }
        });

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

        start_commit_reader(registry, Arc::clone(&runtime), cx);
        cx.on_app_quit(move |_| {
            let runtime = Arc::clone(&runtime_for_quit);
            async move {
                let _ = runtime.shutdown();
                host_log(
                    log_level_for_quit,
                    LogLevel::Info,
                    format!("runtime terminated status={:?}", runtime.status()),
                );
            }
        })
        .detach();
        cx.activate(true);
    });
}

#[cfg(test)]
pub mod test_support {
    use super::*;
    use gpui::TestAppContext;
    use react_gpui::{
        COMMAND_BLUR, COMMAND_CLIPBOARD_READ, COMMAND_CLIPBOARD_WRITE, COMMAND_FOCUS,
        COMMAND_FOCUS_NEXT, COMMAND_FOCUS_PREV, COMMAND_GET_FOCUS, COMMAND_GET_WINDOW_SIZE,
        COMMAND_OPEN_SURFACE, COMMAND_OPEN_URL, COMMAND_RESIZE_WINDOW, COMMAND_SCROLL_TO_END,
        COMMAND_SCROLL_TO_INDEX, COMMAND_SET_MENUS, COMMAND_SET_SELECTION, COMMAND_SET_TITLE,
        COMMAND_SHOW_NOTIFICATION, COMMAND_TOGGLE_FULLSCREEN, EventPayload, HostProperties,
        InMemoryAdapter, KIND_TEXT_INPUT, KIND_VIEW, KIND_VIRTUAL_LIST, MenuDefinition,
        MenuItemDefinition, Node, PROTOCOL_VERSION, TextInputProperties, VirtualListProperties,
    };

    fn command(
        request_id: u32,
        kind: u32,
        node_id: u32,
        payload: Option<(u32, u32)>,
        title: Option<&str>,
        body: Option<&str>,
        menus: Option<Vec<MenuDefinition>>,
    ) -> Command {
        Command {
            protocol: PROTOCOL_VERSION,
            message: react_gpui::COMMAND_MESSAGE,
            surface_id: 1,
            epoch: 1,
            after_revision: 1,
            request_id,
            node_id,
            kind,
            payload,
            title: title.map(str::to_owned),
            body: body.map(str::to_owned),
            menus,
        }
    }

    fn snapshot() -> Snapshot {
        snapshot_for(1)
    }

    fn snapshot_for(surface_id: u32) -> Snapshot {
        let mut view = Node::new(2, 1, 0, KIND_VIEW);
        view.listener_id = 10;
        view.focusable = true;

        let mut input = Node::new(3, 1, 1, KIND_TEXT_INPUT);
        input.listener_id = 11;
        input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
            value: "hello".to_owned(),
            placeholder: None,
            multiline: false,
            disabled: false,
            controlled: false,
            ack_edit_seq: 0,
            selection_start: 0,
            selection_end: 0,
            marked_start: None,
            marked_end: None,
            max_length: None,
            selection_reversed: false,
        }));

        let mut list = Node::new(4, 1, 2, KIND_VIRTUAL_LIST);
        list.listener_id = 12;
        list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
            item_count: 100,
            range_start: 0,
            range_end: 10,
            estimated_item_size: 24.0,
            overscan: 2,
        }));

        Snapshot::new(
            surface_id,
            1,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), view, input, list],
        )
    }

    fn window_for(
        registry: &Entity<SurfaceRegistry>,
        cx: &mut TestAppContext,
        surface_id: u32,
    ) -> WindowHandle<ReactRoot> {
        registry.read_with(cx, |registry, _| {
            registry
                .surfaces
                .get(&surface_id)
                .expect("test surface")
                .window
        })
    }

    fn draw_surface(
        registry: &Entity<SurfaceRegistry>,
        cx: &mut TestAppContext,
        surface_id: u32,
    ) -> WindowHandle<ReactRoot> {
        let window = window_for(registry, cx, surface_id);
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw test surface");
        cx.run_until_parked();
        window
    }
    fn advance_frame(window: WindowHandle<ReactRoot>, cx: &mut TestAppContext) {
        cx.update_window(window.into(), |_, window, cx| {
            window.simulate_next_frame(cx);
        })
        .expect("advance test frame");
        cx.run_until_parked();
    }

    fn route_command(
        registry: &Entity<SurfaceRegistry>,
        cx: &mut TestAppContext,
        command: Command,
    ) {
        let payload = command.encode().expect("encode test command");
        registry
            .update(cx, |registry, cx| registry.route_payload(&payload, cx))
            .expect("route test command");
        draw_surface(registry, cx, command.surface_id);
    }

    fn take_events(runtime: &InMemoryAdapter) -> Vec<react_gpui::Event> {
        let mut events = Vec::new();
        while let Some(event) = runtime.take_event().expect("read test event") {
            events.push(event);
        }
        events
    }

    fn command_result(events: &[react_gpui::Event], request_id: u32) -> react_gpui::CommandResult {
        events
            .iter()
            .find_map(|event| match &event.payload {
                Some(EventPayload::CommandResult(result)) if result.request_id == request_id => {
                    Some(result.clone())
                }
                _ => None,
            })
            .expect("command result event")
    }

    pub fn command_roundtrip(cx: &mut TestAppContext) {
        let runtime = InMemoryAdapter::new();
        let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
        registry
            .update(cx, |registry, cx| registry.open_initial(cx))
            .expect("open initial test surface");
        let window = draw_surface(&registry, cx, 1);

        let action_registry = registry.downgrade();
        cx.update(|cx| {
            cx.on_action(move |action: &MenuAction, cx| {
                if let Some(registry) = action_registry.upgrade() {
                    registry.update(cx, |registry, cx| {
                        registry.emit_action(action.name.clone(), cx)
                    });
                }
            });
        });

        let snapshot_payload = snapshot().encode().expect("encode test snapshot");
        registry
            .update(cx, |registry, cx| {
                registry.route_payload(&snapshot_payload, cx)
            })
            .expect("route test snapshot");
        draw_surface(&registry, cx, 1);
        advance_frame(window, cx);
        assert!(registry.read_with(cx, |registry, _| {
            registry.surfaces.get(&1).is_some_and(|surface| {
                surface
                    .root
                    .read_with(cx, |root, _| root.store().get(4).is_some())
            })
        }));

        route_command(
            &registry,
            cx,
            command(1, COMMAND_FOCUS, 2, None, None, None, None),
        );
        let events = take_events(&runtime);
        assert!(command_result(&events, 1).success);
        for event_type in [
            react_gpui::EVENT_WINDOW_RESIZE,
            react_gpui::EVENT_WINDOW_ACTIVATION,
            react_gpui::EVENT_WINDOW_APPEARANCE,
            react_gpui::protocol::EVENT_LAYOUT,
        ] {
            assert!(
                events.iter().any(|event| event.event_type == event_type),
                "initial observer event {event_type}"
            );
        }

        route_command(
            &registry,
            cx,
            command(2, COMMAND_GET_FOCUS, 2, None, None, None, None),
        );
        let events = take_events(&runtime);
        assert_eq!(
            command_result(&events, 2).value,
            Some(react_gpui::CommandValue::Bool(true))
        );

        route_command(
            &registry,
            cx,
            command(3, COMMAND_FOCUS, 3, None, None, None, None),
        );
        let events = take_events(&runtime);
        assert!(command_result(&events, 3).success);
        assert!(
            events
                .iter()
                .any(|event| { event.event_type == react_gpui::EVENT_FOCUS })
        );
        route_command(
            &registry,
            cx,
            command(4, COMMAND_SET_SELECTION, 3, Some((1, 4)), None, None, None),
        );
        let events = take_events(&runtime);
        assert!(command_result(&events, 4).success);
        assert!(events.iter().any(|event| {
            event.event_type == react_gpui::EVENT_SELECTION
                && matches!(
                    &event.payload,
                    Some(EventPayload::TextInput(input))
                        if input.selection_start == 1 && input.selection_end == 4
                )
        }));

        route_command(
            &registry,
            cx,
            command(20, COMMAND_BLUR, 3, None, None, None, None),
        );
        let events = take_events(&runtime);
        assert!(command_result(&events, 20).success);
        assert!(
            events
                .iter()
                .any(|event| { event.event_type == react_gpui::EVENT_BLUR })
        );
        route_command(
            &registry,
            cx,
            command(21, COMMAND_GET_FOCUS, 3, None, None, None, None),
        );
        assert_eq!(
            command_result(&take_events(&runtime), 21).value,
            Some(react_gpui::CommandValue::Bool(false))
        );
        route_command(
            &registry,
            cx,
            command(
                5,
                COMMAND_CLIPBOARD_WRITE,
                1,
                None,
                Some("copied"),
                None,
                None,
            ),
        );
        assert!(command_result(&take_events(&runtime), 5).success);
        route_command(
            &registry,
            cx,
            command(6, COMMAND_CLIPBOARD_READ, 1, None, None, None, None),
        );
        assert_eq!(
            command_result(&take_events(&runtime), 6).value,
            Some(react_gpui::CommandValue::Text("copied".to_owned()))
        );
        assert_eq!(
            cx.read_from_clipboard()
                .and_then(|item| item.text())
                .as_deref(),
            Some("copied")
        );

        route_command(
            &registry,
            cx,
            command(
                7,
                COMMAND_SCROLL_TO_INDEX,
                4,
                Some((42, 0)),
                None,
                None,
                None,
            ),
        );
        advance_frame(window, cx);
        let events = take_events(&runtime);
        assert!(command_result(&events, 7).success);
        assert!(
            events
                .iter()
                .any(|event| event.event_type == react_gpui::EVENT_VISIBLE_RANGE)
        );
        route_command(
            &registry,
            cx,
            command(8, COMMAND_SCROLL_TO_END, 4, None, None, None, None),
        );
        assert!(command_result(&take_events(&runtime), 8).success);

        route_command(
            &registry,
            cx,
            command(9, COMMAND_SET_TITLE, 1, None, Some("Headless"), None, None),
        );
        assert!(command_result(&take_events(&runtime), 9).success);
        route_command(
            &registry,
            cx,
            command(10, COMMAND_GET_WINDOW_SIZE, 1, None, None, None, None),
        );
        assert!(matches!(
            command_result(&take_events(&runtime), 10).value,
            Some(react_gpui::CommandValue::Pair((width, height)))
                if width > 0.0 && height > 0.0
        ));

        for (request_id, kind, payload) in [
            (11, COMMAND_RESIZE_WINDOW, Some((640, 480))),
            (13, COMMAND_TOGGLE_FULLSCREEN, None),
        ] {
            route_command(
                &registry,
                cx,
                command(request_id, kind, 1, payload, None, None, None),
            );
            assert!(command_result(&take_events(&runtime), request_id).success);
        }
        route_command(
            &registry,
            cx,
            command(
                14,
                COMMAND_OPEN_URL,
                1,
                None,
                Some("https://example.test"),
                None,
                None,
            ),
        );
        assert!(command_result(&take_events(&runtime), 14).success);
        assert_eq!(cx.opened_url().as_deref(), Some("https://example.test"));

        for (request_id, kind) in [(15, COMMAND_FOCUS_NEXT), (16, COMMAND_FOCUS_PREV)] {
            route_command(
                &registry,
                cx,
                command(request_id, kind, 1, None, None, None, None),
            );
            assert!(command_result(&take_events(&runtime), request_id).success);
        }

        route_command(
            &registry,
            cx,
            command(
                17,
                COMMAND_SET_MENUS,
                1,
                None,
                None,
                None,
                Some(vec![MenuDefinition {
                    title: "Test".to_owned(),
                    items: vec![MenuItemDefinition::Action("test-action".to_owned())],
                }]),
            ),
        );
        assert!(command_result(&take_events(&runtime), 17).success);
        cx.update_window(window.into(), |_, window, _| window.activate_window())
            .expect("activate menu test window");
        cx.dispatch_action(
            window.into(),
            MenuAction {
                name: "test-action".to_owned(),
            },
        );
        cx.run_until_parked();
        assert!(take_events(&runtime).iter().any(|event| {
            event.event_type == react_gpui::EVENT_ACTION
                && matches!(
                    &event.payload,
                    Some(EventPayload::EventAction { action }) if action == "test-action"
                )
        }));

        cx.update(|cx| {
            cx.set_app_identity("com.example.react-gpui", "React GPUI");
        });
        route_command(
            &registry,
            cx,
            command(
                18,
                COMMAND_SHOW_NOTIFICATION,
                1,
                None,
                Some("Title"),
                Some("Body"),
                None,
            ),
        );
        assert!(command_result(&take_events(&runtime), 18).success);
        assert_eq!(cx.shown_system_notifications().len(), 1);

        route_command(
            &registry,
            cx,
            command(
                19,
                COMMAND_OPEN_SURFACE,
                1,
                Some((320, 240)),
                Some("Aux"),
                None,
                None,
            ),
        );
        let events = take_events(&runtime);
        assert!(command_result(&events, 19).success);
        assert_eq!(
            registry.read_with(cx, |registry, _| registry.surfaces.len()),
            2
        );
        let auxiliary_snapshot = snapshot_for(2).encode().expect("encode auxiliary snapshot");
        registry
            .update(cx, |registry, cx| {
                registry.route_payload(&auxiliary_snapshot, cx)
            })
            .expect("route auxiliary snapshot");
        let auxiliary_window = draw_surface(&registry, cx, 2);
        let closed_window_id = auxiliary_window.window_id();
        registry.update(cx, |registry, cx| {
            assert!(!registry.window_closed(closed_window_id, cx));
        });
        assert!(take_events(&runtime).iter().any(|event| {
            event.event_type == react_gpui::EVENT_SURFACE_CLOSED && event.surface_id == 2
        }));
    }
}

#[derive(Debug, Eq, PartialEq)]
enum CliAction {
    Help,
    Version,
    Run {
        mode: RuntimeMode,
        renderer_args: Vec<OsString>,
    },
}

fn parse_host_args(args: &[OsString]) -> Result<CliAction, String> {
    let mut mode = RuntimeMode::Process;
    let mut renderer_args = Vec::new();
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
                return Err("--runtime requires `process` or `embedded`".to_owned());
            };
            mode = match value.to_string_lossy().as_ref() {
                "process" => RuntimeMode::Process,
                "embedded" => RuntimeMode::Embedded,
                value => return Err(format!("unknown runtime `{value}`")),
            };
        } else if arg == "--embedded" {
            mode = RuntimeMode::Embedded;
        } else if arg == "--watch" || arg == "--smoke-press" {
            renderer_args.push(arg.clone());
        } else if arg.to_string_lossy().starts_with('-') {
            return Err(format!("unknown host option `{}`", arg.to_string_lossy()));
        } else {
            host_options = false;
            renderer_args.push(arg.clone());
        }
        index += 1;
    }

    Ok(CliAction::Run {
        mode,
        renderer_args,
    })
}

fn renderer_entry(mode: RuntimeMode, renderer_args: &[OsString]) -> String {
    match mode {
        RuntimeMode::Process => renderer_args
            .first()
            .map(|arg| arg.to_string_lossy().into_owned())
            .or_else(|| env::var(COMMAND_ENV).ok())
            .unwrap_or_else(|| "bun".to_owned()),
        RuntimeMode::Embedded => renderer_args
            .iter()
            .find(|arg| *arg != "--watch" && *arg != "--smoke-press")
            .map(|arg| arg.to_string_lossy().into_owned())
            .unwrap_or_else(|| DEFAULT_EMBEDDED_ENTRY.to_owned()),
    }
}

fn start_runtime(
    mode: RuntimeMode,
    renderer_args: &[OsString],
) -> Result<Arc<dyn RuntimeAdapter>, String> {
    match mode {
        RuntimeMode::Process => ProcessAdapter::spawn(renderer_command(renderer_args))
            .map(|runtime| runtime as Arc<dyn RuntimeAdapter>)
            .map_err(|error| format!("failed to spawn process renderer: {error}")),
        RuntimeMode::Embedded => start_embedded(renderer_args),
    }
}

fn start_embedded(renderer_args: &[OsString]) -> Result<Arc<dyn RuntimeAdapter>, String> {
    #[cfg(feature = "embedded-bun")]
    {
        let entry = renderer_args
            .iter()
            .find(|arg| *arg != "--smoke-press" && *arg != "--watch")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_EMBEDDED_ENTRY));
        let runtime = EmbeddedBunAdapter::start(&entry)
            .map_err(|error| format!("failed to start embedded Bun: {error}"))?;
        if renderer_args.iter().any(|arg| arg == "--watch") {
            runtime
                .watch(&entry)
                .map_err(|error| format!("failed to watch embedded entry: {error}"))?;
        }
        if renderer_args.iter().any(|arg| arg == "--smoke-press") {
            let runtime_for_smoke = Arc::clone(&runtime);
            let watch_enabled = renderer_args.iter().any(|arg| arg == "--watch");
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(2000));
                let sent = send_event_or_exit(
                    runtime_for_smoke.as_ref(),
                    "embedded smoke press event",
                    &Event::press(1, 1, 1, 1, 7, 1),
                );
                std::thread::sleep(std::time::Duration::from_millis(2000));
                eprintln!(
                    "react-gpui-host: embedded smoke press sent={sent}, commits={}, status={:?}",
                    runtime_for_smoke.commit_count(),
                    runtime_for_smoke.runtime_status()
                );
                if watch_enabled {
                    std::thread::sleep(std::time::Duration::from_millis(3000));
                    eprintln!(
                        "react-gpui-host: embedded refresh commits={}, queued={}, status={:?}",
                        runtime_for_smoke.commit_count(),
                        runtime_for_smoke.refresh_count(),
                        runtime_for_smoke.runtime_status()
                    );
                }
            });
        }
        return Ok(runtime as Arc<dyn RuntimeAdapter>);
    }
    #[cfg(not(feature = "embedded-bun"))]
    {
        let _ = renderer_args;
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
fn print_version() {
    println!("react-gpui-host {}", env!("CARGO_PKG_VERSION"));
}

fn print_help() {
    println!(
        "react-gpui-host\n\nUsage:\n  react-gpui-host [host-options] [renderer-command [args...]]\n  react-gpui-host --runtime embedded [entry.tsx]\n\nHost options:\n  -h, --help           print this help without starting GPUI\n  -V, --version        print the package version without starting GPUI\n  --runtime process    child-process ProcessAdapter (default)\n  --runtime embedded   in-process Bun/JSC adapter (build with --features embedded-bun)\n  --embedded           alias for --runtime embedded\n  --watch              watch the embedded entry and apply Fast Refresh updates\n  --smoke-press        send one current-wire press for embedded counter smoke\n  --                   stop parsing host options; pass the rest to the renderer\n\nConfiguration:\n  {COMMAND_ENV}  renderer executable (process mode; default: bun)\n  {ARGS_ENV}     whitespace-separated default renderer args\n  {LOG_ENV}      diagnostics level: off, error (default), info, or debug\n\nThe embedded default entry is {DEFAULT_EMBEDDED_ENTRY}. Both runtimes use\nlength-prefixed MessagePack Commit Batches and current-wire events."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
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
        assert_eq!(
            parse_host_args(&args(&["--runtime", "embedded", "entry.tsx"])),
            Ok(CliAction::Run {
                mode: RuntimeMode::Embedded,
                renderer_args: args(&["entry.tsx"]),
            })
        );
    }

    #[test]
    fn separator_preserves_renderer_help_flag() {
        assert_eq!(
            parse_host_args(&args(&["--", "bun", "run", "app.tsx", "--help"])),
            Ok(CliAction::Run {
                mode: RuntimeMode::Process,
                renderer_args: args(&["bun", "run", "app.tsx", "--help"]),
            })
        );
    }

    #[test]
    fn renderer_command_stops_host_option_parsing() {
        assert_eq!(
            parse_host_args(&args(&["bun", "run", "app.tsx", "--version"])),
            Ok(CliAction::Run {
                mode: RuntimeMode::Process,
                renderer_args: args(&["bun", "run", "app.tsx", "--version"]),
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
            env::temp_dir().join(format!("react-gpui-host-panic-test-{}", std::process::id()));
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
    fn surface_ids_are_host_allocated_without_implicit_surface_entries() {
        let mut registry = SurfaceRegistry::new(react_gpui::InMemoryAdapter::new());
        assert_eq!(registry.allocate_surface_id(), Ok(1));
        assert_eq!(registry.allocate_surface_id(), Ok(2));
        assert!(registry.surfaces.is_empty());
    }

    #[test]
    fn surface_id_allocator_rejects_exhaustion() {
        let mut registry = SurfaceRegistry::new(react_gpui::InMemoryAdapter::new());
        registry.next_surface_id = u32::MAX;
        assert_eq!(
            registry.allocate_surface_id(),
            Err("surface id space exhausted".to_owned())
        );
    }
}
