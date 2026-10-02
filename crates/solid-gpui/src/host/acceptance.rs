//! Explicitly launched, process-owned acceptance over the production native host.
//! No listener, environment auto-enable, DOM facade, or alternate tree decoder.
use super::*;
use crate::renderer::acceptance::PaintedNode;
use gpui::{
    InputEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ScrollDelta, ScrollWheelEvent, TestAppContext, TestDispatcher, point,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    sync::Mutex,
    time::Duration,
};

const LIMIT: usize = 16 * 1024 * 1024;
const EVENT_LIMIT: usize = 4096;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Request {
    id: u32,
    action: Action,
    #[serde(default)]
    frames: Vec<Vec<u8>>,
    #[serde(default = "initial_surface")]
    surface_id: u32,
    target: Option<Target>,
    point: Option<Position>,
    from: Option<Position>,
    delta: Option<Position>,
    text: Option<String>,
    milliseconds: Option<u64>,
    width: Option<u32>,
    height: Option<u32>,
}
fn initial_surface() -> u32 {
    1
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Action {
    Flush,
    Snapshot,
    Click,
    Type,
    Key,
    Drag,
    Wheel,
    Resize,
    AdvanceClock,
    Screenshot,
    ClipboardText,
    Close,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Target {
    surface_id: u32,
    epoch: u32,
    revision: u32,
    id: u32,
    listener_id: u32,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    id: u32,
    error: Option<String>,
    frames: Vec<Vec<u8>>,
    result: serde_json::Value,
}

/// Event bytes are bounded before retaining them, including native extension events.
struct AcceptanceRuntime {
    events: Mutex<(VecDeque<Vec<u8>>, usize)>,
    closed: std::sync::atomic::AtomicBool,
}
impl AcceptanceRuntime {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            events: Default::default(),
            closed: false.into(),
        })
    }
    fn drain(&self) -> Vec<Vec<u8>> {
        let mut events = self.events.lock().unwrap();
        events.1 = 0;
        events.0.drain(..).collect()
    }
}
impl RuntimeAdapter for AcceptanceRuntime {
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, crate::ProtocolError> {
        Ok(None)
    }
    fn send_event(&self, event: Event) -> Result<(), crate::ProtocolError> {
        let mut frame = Vec::new();
        event.encode_frame_into(&mut frame)?;
        let mut events = self.events.lock().unwrap();
        if events.0.len() >= EVENT_LIMIT || events.1 + frame.len() > LIMIT / 4 {
            return Err(crate::ProtocolError::Io(std::io::Error::other(
                "acceptance event budget exceeded",
            )));
        }
        events.1 += frame.len();
        events.0.push_back(frame);
        Ok(())
    }
    fn request_shutdown(&self) -> Result<(), crate::ProtocolError> {
        self.closed
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    fn shutdown(&self) -> Result<(), crate::ProtocolError> {
        self.request_shutdown()
    }
    fn status(&self) -> crate::RuntimeStatus {
        if self.closed.load(std::sync::atomic::Ordering::Relaxed) {
            crate::RuntimeStatus::Shutdown
        } else {
            crate::RuntimeStatus::Running
        }
    }
}

enum NativeContext {
    Deterministic(TestAppContext),
    #[cfg(target_os = "macos")]
    Gpu(gpui::VisualTestAppContext),
}
impl NativeContext {
    fn resize(
        &mut self,
        handle: AnyWindowHandle,
        viewport: gpui::Size<gpui::Pixels>,
    ) -> Result<(), String> {
        match self {
            Self::Deterministic(cx) => cx.simulate_window_resize(handle, viewport),
            #[cfg(target_os = "macos")]
            Self::Gpu(cx) => cx
                .update(|cx| cx.update_window(handle, |_, window, _| window.resize(viewport)))
                .map_err(|error| error.to_string())?,
        }
        Ok(())
    }
    fn update<R>(&mut self, f: impl FnOnce(&mut App) -> R) -> R {
        match self {
            Self::Deterministic(cx) => cx.update(f),
            #[cfg(target_os = "macos")]
            Self::Gpu(cx) => cx.update(f),
        }
    }
    fn park(&self) {
        match self {
            Self::Deterministic(cx) => cx.run_until_parked(),
            #[cfg(target_os = "macos")]
            Self::Gpu(cx) => {
                cx.run_until_parked();
                // AppKit window operations use the platform's main dispatch queue.
                core_foundation::runloop::CFRunLoop::run_in_mode(
                    unsafe { core_foundation::runloop::kCFRunLoopDefaultMode },
                    Duration::from_millis(1),
                    false,
                );
                cx.run_until_parked();
            }
        }
    }
    fn advance(&self, duration: Duration) {
        match self {
            Self::Deterministic(cx) => cx.executor().advance_clock(duration),
            #[cfg(target_os = "macos")]
            Self::Gpu(cx) => cx.advance_clock(duration),
        }
    }
}

struct AcceptanceSession {
    registry: Entity<NativeStateRegistry>,
    cx: NativeContext,
    runtime: Arc<AcceptanceRuntime>,
    gpu: bool,
}
impl AcceptanceSession {
    fn draw(&mut self) -> Result<(), String> {
        self.cx.park();
        self.cx.update(|cx| {
            let windows = self
                .registry
                .read(cx)
                .surfaces
                .values()
                .map(|s| (s.window, s.root.clone()))
                .collect::<Vec<_>>();
            for (handle, root) in windows {
                root.update(cx, |root, _| {
                    if root.acceptance.is_none() {
                        root.enable_acceptance();
                    }
                });
                cx.update_window(handle, |_, window, cx| {
                    window.draw(cx).clear(cx);
                    // Manual draws must also deliver production frame observers.
                    window.simulate_next_frame(cx);
                })
                .map_err(|e| e.to_string())?;
            }
            Ok::<(), String>(())
        })?;
        self.cx.park();
        Ok(())
    }
    fn window(&mut self, id: u32) -> Result<AnyWindowHandle, String> {
        self.cx
            .update(|cx| self.registry.read(cx).surfaces.get(&id).map(|s| s.window))
            .ok_or_else(|| "acceptance surface is closed or absent".into())
    }
    fn target(&mut self, target: &Target) -> Result<PaintedNode, String> {
        self.cx.update(|cx| {
            let surface = self.registry.read(cx).surfaces.get(&target.surface_id).ok_or("acceptance surface is closed or absent")?;
            let root = surface.root.read(cx);
            if root.store().epoch() != target.epoch { return Err("acceptance target belongs to a stale epoch".into()); }
            if root.store().revision() != target.revision { return Err("acceptance target belongs to a stale revision; locate it again after application commits".into()); }
            root.acceptance_nodes().into_iter().find(|node| node.id == target.id && node.listener_id == target.listener_id)
                .ok_or_else(|| "acceptance target is stale, removed, or not painted".into())
        })
    }
    fn event<E: InputEvent>(&mut self, handle: AnyWindowHandle, event: E) -> Result<(), String> {
        self.cx
            .update(|cx| {
                cx.update_window(handle, |_, window, cx| {
                    window.dispatch_event(event.to_platform_input(), cx);
                })
            })
            .map_err(|e| e.to_string())?;
        self.cx.park();
        Ok(())
    }
    fn key(&mut self, handle: AnyWindowHandle, key: Keystroke) -> Result<(), String> {
        self.cx
            .update(|cx| {
                cx.update_window(handle, |_, window, cx| window.dispatch_keystroke(key, cx))
            })
            .map_err(|e| e.to_string())?;
        self.cx.park();
        Ok(())
    }
    fn execute(&mut self, request: &Request) -> Result<serde_json::Value, String> {
        if request.frames.len() > 256 {
            return Err("acceptance commit count exceeds 256".into());
        }
        if request.frames.iter().map(Vec::len).sum::<usize>() > LIMIT / 4 {
            return Err("acceptance commit bytes exceed 4 MiB".into());
        }
        for frame in &request.frames {
            let mut reader = frame.as_slice();
            let payload = crate::read_frame(&mut reader)
                .map_err(|e| e.to_string())?
                .ok_or("empty acceptance commit")?;
            if !reader.is_empty() {
                return Err("acceptance commit contains trailing bytes".into());
            }
            self.cx.update(|cx| {
                self.registry
                    .update(cx, |registry, cx| registry.route_payload(&payload, cx))
            })?;
        }
        self.draw()?;
        if matches!(request.action, Action::Close) {
            self.cleanup();
            return Ok(self.cx.update(|cx| {
                let registry = self.registry.read(cx);
                serde_json::json!({ "surfaces": registry.surfaces.len(), "windows": cx.windows().len(), "popups": registry.popups.len() })
            }));
        }
        let handle = self.window(request.surface_id)?;
        let mut result = serde_json::Value::Null;
        match request.action {
            Action::Flush => {}
            Action::Resize => {
                let width = request.width.ok_or("resize requires width")?;
                let height = request.height.ok_or("resize requires height")?;
                if width == 0 || height == 0 || width > 16_384 || height > 16_384 {
                    return Err("acceptance resize requires dimensions from 1 to 16384".into());
                }
                self.cx
                    .resize(handle, gpui::size(px(width as f32), px(height as f32)))?;
            }
            Action::Snapshot => {
                result = self.cx.update(|cx| {
                    let root = self.registry.read(cx).surfaces[&request.surface_id].root.read(cx);
                    serde_json::json!({"surfaceId": root.store().surface_id(), "epoch": root.store().epoch(), "revision": root.store().revision(), "nodes": root.acceptance_nodes()})
                });
            }
            Action::Click | Action::Drag | Action::Wheel => {
                let target = request
                    .target
                    .as_ref()
                    .ok_or("acceptance action requires a captured target")?;
                if target.surface_id != request.surface_id {
                    return Err("acceptance target surface mismatch".into());
                }
                let node = self.target(target)?;
                if matches!(request.action, Action::Drag) {
                    finite(request.point.ok_or("drag requires a destination")?)?;
                }
                if matches!(request.action, Action::Wheel) {
                    finite(request.delta.ok_or("wheel requires delta")?)?;
                }
                let start = request.from.unwrap_or(Position {
                    x: node.bounds.x + node.bounds.width / 2.,
                    y: node.bounds.y + node.bounds.height / 2.,
                });
                finite(start)?;
                if start.x < node.bounds.x
                    || start.y < node.bounds.y
                    || start.x >= node.bounds.x + node.bounds.width
                    || start.y >= node.bounds.y + node.bounds.height
                {
                    return Err("acceptance pointer start is outside painted target bounds".into());
                }
                let at = point(px(start.x), px(start.y));
                self.event(
                    handle,
                    MouseMoveEvent {
                        position: at,
                        pressed_button: None,
                        modifiers: Modifiers::default(),
                    },
                )?;
                if matches!(request.action, Action::Wheel) {
                    let delta = request.delta.ok_or("wheel requires delta")?;
                    finite(delta)?;
                    self.event(
                        handle,
                        ScrollWheelEvent {
                            position: at,
                            delta: ScrollDelta::Pixels(point(px(delta.x), px(delta.y))),
                            ..Default::default()
                        },
                    )?;
                } else {
                    self.event(
                        handle,
                        MouseDownEvent {
                            position: at,
                            button: MouseButton::Left,
                            click_count: 1,
                            modifiers: Modifiers::default(),
                            first_mouse: false,
                        },
                    )?;
                    let mut end = at;
                    if matches!(request.action, Action::Drag) {
                        let destination = request.point.ok_or("drag requires a destination")?;
                        finite(destination)?;
                        for step in 1..=8 {
                            let amount = step as f32 / 8.;
                            end = point(
                                px(start.x + (destination.x - start.x) * amount),
                                px(start.y + (destination.y - start.y) * amount),
                            );
                            self.event(
                                handle,
                                MouseMoveEvent {
                                    position: end,
                                    pressed_button: Some(MouseButton::Left),
                                    modifiers: Modifiers::default(),
                                },
                            )?;
                            self.draw()?;
                        }
                    }
                    self.event(
                        handle,
                        MouseUpEvent {
                            position: end,
                            button: MouseButton::Left,
                            click_count: 1,
                            modifiers: Modifiers::default(),
                        },
                    )?;
                }
            }
            Action::Type => {
                let text = request.text.as_deref().ok_or("type requires text")?;
                if text.chars().count() > 512 {
                    return Err("acceptance typing exceeds 512 Unicode scalar values; split typing into actions".into());
                }
                let root = self.cx.update(|cx| {
                    self.registry.read(cx).surfaces[&request.surface_id]
                        .root
                        .clone()
                });
                let focused = self
                    .cx
                    .update(|cx| {
                        cx.update_window(handle, |_, window, cx| {
                            root.read(cx).acceptance_has_focus(window, cx)
                        })
                    })
                    .map_err(|e| e.to_string())?;
                if !focused {
                    return Err(
                        "acceptance typing requires native focus; click an editor first".into(),
                    );
                }
                for character in text.chars() {
                    let key = character.to_string();
                    self.key(
                        handle,
                        Keystroke {
                            modifiers: Modifiers::default(),
                            key: key.clone(),
                            key_char: Some(key),
                        },
                    )?;
                }
            }
            Action::Key => {
                let key = Keystroke::parse(
                    request
                        .text
                        .as_deref()
                        .ok_or("key requires a GPUI keystroke")?,
                )
                .map_err(|e| e.to_string())?;
                self.key(handle, key)?;
            }
            Action::AdvanceClock => {
                let duration = request
                    .milliseconds
                    .ok_or("advance-clock requires milliseconds")?;
                if duration > 60_000 {
                    return Err("acceptance clock advance exceeds 60 seconds".into());
                }
                self.cx.advance(Duration::from_millis(duration));
            }
            Action::Screenshot => {
                if !self.gpu {
                    return Err(
                        "screenshots are unsupported in deterministic mode; use macOS gpu mode"
                            .into(),
                    );
                }
                self.cx
                    .update(|cx| {
                        cx.update_window(handle, |_, window, _| {
                            let size = window.viewport_size();
                            let scale = window.scale_factor();
                            if f32::from(size.width) * f32::from(size.height) * scale * scale
                                > 4. * 1024. * 1024.
                            {
                                Err("acceptance screenshot exceeds 4 million pixels".to_owned())
                            } else {
                                Ok(())
                            }
                        })
                    })
                    .map_err(|e| e.to_string())??;
                let image = self
                    .cx
                    .update(|cx| cx.update_window(handle, |_, window, _| window.render_to_image()))
                    .map_err(|e| e.to_string())?
                    .map_err(|e| e.to_string())?;
                if image.width() as usize * image.height() as usize > 4 * 1024 * 1024 {
                    return Err("acceptance screenshot exceeds 4 million pixels".into());
                }
                let (width, height) = image.dimensions();
                let mut png = std::io::Cursor::new(Vec::new());
                image::DynamicImage::ImageRgba8(image)
                    .write_to(&mut png, image::ImageFormat::Png)
                    .map_err(|e| e.to_string())?;
                if png.get_ref().len() > LIMIT / 8 {
                    return Err("acceptance PNG exceeds 2 MiB transport budget".into());
                }
                result = serde_json::json!({ "width": width, "height": height, "png": png.into_inner() });
            }
            Action::ClipboardText => {
                let text = self
                    .cx
                    .update(|cx| cx.read_from_clipboard().and_then(|item| item.text()));
                if text.as_ref().is_some_and(|text| text.len() > LIMIT / 8) {
                    return Err("acceptance clipboard text exceeds 2 MiB".into());
                }
                result = serde_json::to_value(text).map_err(|e| e.to_string())?;
            }
            Action::Close => unreachable!("close is application-scoped"),
        }
        if !matches!(request.action, Action::Close) {
            self.draw()?;
        }
        Ok(result)
    }
    fn cleanup(&mut self) {
        let _ = self.runtime.shutdown();
        self.cx.update(|cx| {
            for handle in cx.windows() {
                let _ = cx.update_window(handle, |_, window, cx| {
                    window.replace_root(cx, |_, _| EmptyFrame);
                    window.draw(cx).clear(cx);
                    window.remove_window();
                });
            }
            self.registry.update(cx, |registry, _| {
                registry.surfaces.clear();
                registry.windows.clear();
                registry.popups.clear();
            });
        });
        self.cx.park();
    }
}
struct EmptyFrame;
impl gpui::Render for EmptyFrame {
    fn render(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        gpui::div()
    }
}
fn finite(point: Position) -> Result<(), String> {
    if point.x.is_finite() && point.y.is_finite() {
        Ok(())
    } else {
        Err("acceptance coordinates must be finite".into())
    }
}

struct AcceptanceProfile<P>(P, bool);
impl<P: HostProfile> HostProfile for AcceptanceProfile<P> {
    fn native_bindings(&self) -> Result<String, String> {
        self.0.native_bindings()
    }
    fn window_options(&self, options: WindowOptions, cx: &App) -> WindowOptions {
        let mut options = self.0.window_options(options, cx);
        if self.1 {
            let size = options
                .window_bounds
                .map(|b| b.get_bounds().size)
                .unwrap_or(size(px(800.), px(600.)));
            options.window_bounds = Some(WindowBounds::Windowed(Bounds::new(
                point(px(-10000.), px(-10000.)),
                size,
            )));
            options.focus = false;
        }
        options
    }
    fn capabilities(&self) -> HostCapabilities {
        self.0.capabilities()
    }
    fn extension_registry(&self) -> Rc<dyn ExtensionRegistry> {
        self.0.extension_registry()
    }
    fn initialize(&mut self, cx: &mut App) {
        self.0.initialize(cx)
    }
    fn open_window(
        &self,
        options: WindowOptions,
        runtime: Arc<dyn RuntimeAdapter>,
        extensions: Rc<dyn ExtensionRegistry>,
        cx: &mut App,
    ) -> Result<(AnyWindowHandle, Entity<SolidRoot>), String> {
        self.0.open_window(options, runtime, extensions, cx)
    }
    fn restore_keybindings(&self, baseline: &[KeyBinding], dynamic: Vec<KeyBinding>, cx: &mut App) {
        self.0.restore_keybindings(baseline, dynamic, cx)
    }
    fn rejected_command_reason(&self, kind: CommandKind) -> Option<&'static str> {
        self.0.rejected_command_reason(kind)
    }
}

/// Run on the process main thread. Only this explicit executable entry enables
/// automation. A custom host can pass its production HostProfile here.
pub fn run(profile: impl HostProfile) -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 || args[0] != "--native-acceptance" {
        return Err("requires --native-acceptance deterministic|gpu".into());
    }
    let gpu = match args[1].as_str() {
        "deterministic" => false,
        "gpu" => true,
        _ => return Err("unsupported acceptance mode".into()),
    };
    let mut cx = if gpu {
        #[cfg(target_os = "macos")]
        {
            NativeContext::Gpu(gpui::VisualTestAppContext::with_asset_source(
                gpui_platform::current_platform(false),
                Arc::new(HostAssets),
            ))
        }
        #[cfg(not(target_os = "macos"))]
        {
            return Err("GPU acceptance and screenshots are unsupported on this platform; the linked GPUI visual context requires macOS".into());
        }
    } else {
        NativeContext::Deterministic(TestAppContext::build(TestDispatcher::new(0), None))
    };
    let runtime = AcceptanceRuntime::new();
    let registry = cx.update(|cx| {
        let mut profile = profile;
        profile.initialize(cx);
        let baseline = cx.key_bindings().borrow().bindings().cloned().collect();
        let registry = cx.new(|_| {
            NativeStateRegistry::with_profile(
                runtime.clone(),
                AcceptanceProfile(profile, gpu),
                baseline,
            )
        });
        registry.update(cx, |registry, cx| registry.open_initial(cx))?;
        Ok::<_, String>(registry)
    })?;
    let mut session = AcceptanceSession {
        cx,
        registry,
        runtime,
        gpu,
    };
    let result = serve(&mut session);
    session.cleanup();
    result
}

fn serve(session: &mut AcceptanceSession) -> Result<(), String> {
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    let hello = serde_json::json!({ "version": 2, "mode": if session.gpu { "gpu" } else { "deterministic" }, "platform": std::env::consts::OS, "screenshots": session.gpu, "clock": true });
    write_packet(&mut output, &hello)?;
    loop {
        let mut header = [0; 4];
        if input.read(&mut header[..1]).map_err(|e| e.to_string())? == 0 {
            return Ok(());
        }
        input
            .read_exact(&mut header[1..])
            .map_err(|e| e.to_string())?;
        let length = u32::from_le_bytes(header) as usize;
        if length == 0 || length > LIMIT {
            return Err("acceptance packet exceeds budget".into());
        }
        let mut packet = vec![0; length];
        input.read_exact(&mut packet).map_err(|e| e.to_string())?;
        let request: Request = serde_json::from_slice(&packet).map_err(|e| e.to_string())?;
        let result = session.execute(&request);
        let response = Response {
            id: request.id,
            error: result.as_ref().err().cloned(),
            result: result.unwrap_or_default(),
            frames: session.runtime.drain(),
        };
        write_packet(&mut output, &response)?;
        if matches!(request.action, Action::Close) {
            return Ok(());
        }
    }
}
fn write_packet(output: &mut impl Write, value: &impl Serialize) -> Result<(), String> {
    let packet = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if packet.len() > LIMIT {
        return Err("acceptance response exceeds budget".into());
    }
    output
        .write_all(&(packet.len() as u32).to_le_bytes())
        .and_then(|_| output.write_all(&packet))
        .and_then(|_| output.flush())
        .map_err(|e| e.to_string())
}
