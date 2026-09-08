//! Single-threaded browser host for the canonical Solid GPUI byte protocol.
#![cfg(target_family = "wasm")]

use gpui::{
    AnyWindowHandle, AppContext, ApplicationHandle, Context, Entity, IntoElement, ParentElement,
    Render, Styled, Window, WindowOptions, div,
};
use gpui_component::Root;
use solid_gpui::native::NativeModules;
use std::rc::Rc;
mod assets;
use solid_gpui::{Event, ProtocolError, RuntimeAdapter, RuntimeStatus, SolidRoot};
use std::{
    borrow::Cow,
    cell::RefCell,
    io,
    sync::{Arc, Mutex},
};
use wasm_bindgen::prelude::*;

#[derive(Default)]
struct BrowserAdapter {
    events: Mutex<Vec<u8>>,
}
impl RuntimeAdapter for BrowserAdapter {
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, ProtocolError> {
        Err(ProtocolError::Io(io::Error::other(
            "browser commits are submitted on foreground",
        )))
    }
    fn send_event(&self, event: Event) -> Result<(), ProtocolError> {
        let payload = event.encode()?;
        let mut events = self.events.lock().unwrap();
        if events.len() + payload.len() + 4 > solid_gpui::MAX_FRAME_LENGTH {
            return Err(ProtocolError::Io(io::Error::other(
                "browser event queue capacity exceeded",
            )));
        }
        solid_gpui::write_frame(&mut *events, &payload)
    }
    fn request_shutdown(&self) -> Result<(), ProtocolError> {
        Ok(())
    }
    fn shutdown(&self) -> Result<(), ProtocolError> {
        Ok(())
    }
    fn status(&self) -> RuntimeStatus {
        RuntimeStatus::Running
    }
}

struct BrowserSurface {
    root: Entity<SolidRoot>,
}
impl Render for BrowserSurface {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .font_family("Inter Variable")
            .child(self.root.clone())
            .children(Root::render_notification_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_dialog_layer(window, cx))
    }
}

struct Host {
    app: ApplicationHandle,
    window: AnyWindowHandle,
    root: Entity<SolidRoot>,
    adapter: Arc<BrowserAdapter>,
}
thread_local! { static HOST: RefCell<Option<Host>> = const { RefCell::new(None) }; }

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    if HOST.with(|host| host.borrow().is_some()) {
        return Err(JsValue::from_str("browser host is already started"));
    }
    gpui_platform::web_init();
    let adapter = Arc::new(BrowserAdapter::default());
    let runtime = adapter.clone();
    let (opened, result) = futures::channel::oneshot::channel();
    let app = gpui_platform::single_threaded_web()
        .with_assets(assets::WebAssets)
        .run_embedded(move |cx| {
            let window = (|| -> gpui::Result<_> {
                solid_gpui::motion::initialize(cx);
                solid_gpui::components::initialize(cx);
                gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);
                cx.global_mut::<gpui_component::Theme>().font_family = "Inter Variable".into();
                let root_slot = Rc::new(RefCell::new(None));
                let slot = root_slot.clone();
                cx.text_system().add_fonts(vec![
                    Cow::Borrowed(include_bytes!("../fonts/IBMPlexSans-Regular.ttf")),
                    Cow::Borrowed(include_bytes!("../fonts/Inter-Regular.ttf")),
                    Cow::Borrowed(include_bytes!("../fonts/NotoSansSC-Regular.ttf")),
                ])?;
                let window = cx.open_window(
                    WindowOptions {
                        window_background: gpui::WindowBackgroundAppearance::Transparent,
                        ..WindowOptions::default()
                    },
                    move |window, cx| {
                        let modules = Rc::new(NativeModules::new(vec![
                            solid_gpui::components::native_module(),
                        ]));
                        let root = cx.new(|_| SolidRoot::with_extensions(runtime, modules));
                        *slot.borrow_mut() = Some(root.clone());
                        let content = cx.new(|_| BrowserSurface { root });
                        cx.new(|cx| {
                            Root::new(content, window, cx)
                                .bg(gpui::transparent_black())
                                .bordered(false)
                        })
                    },
                )?;
                let root = root_slot
                    .borrow_mut()
                    .take()
                    .expect("window created the Solid root");
                Ok((window, root))
            })();
            let _ = opened.send(window);
        });
    let (window, root) = result.await.map_err(js_error)?.map_err(js_error)?;
    HOST.with(|host| {
        *host.borrow_mut() = Some(Host {
            app,
            window: window.into(),
            root,
            adapter,
        })
    });
    Ok(())
}

#[wasm_bindgen]
pub fn submit(frame: &[u8]) -> Result<(), JsValue> {
    let mut input = io::Cursor::new(frame);
    let payload = solid_gpui::read_frame(&mut input)
        .map_err(js_error)?
        .ok_or_else(|| JsValue::from_str("missing commit frame"))?;
    if input.position() != frame.len() as u64 {
        return Err(JsValue::from_str("expected exactly one commit frame"));
    }
    let message = solid_gpui::decode_message(&payload).map_err(js_error)?;
    HOST.with(|host| {
        let host = host.borrow();
        let host = host
            .as_ref()
            .ok_or_else(|| JsValue::from_str("browser host is not started"))?;
        host.app
            .update(|cx| {
                host.window.update(cx, |_, window, cx| {
                    host.root.update(cx, |root, cx| {
                        if let solid_gpui::DecodedMessage::Command(command) = &message {
                            use solid_gpui::CommandOperation as Op;
                            if !matches!(
                                command.operation,
                                Op::Focus
                                    | Op::Blur
                                    | Op::SetSelection { .. }
                                    | Op::ScrollToIndex { .. }
                                    | Op::ScrollToEnd
                                    | Op::FocusNext
                                    | Op::FocusPrev
                                    | Op::GetWindowSize
                                    | Op::GetFocus
                                    | Op::GetWindowBounds
                                    | Op::GetWindowState
                                    | Op::GetScrollOffset
                                    | Op::ScrollToOffset { .. }
                                    | Op::SetKeybindings { .. }
                                    | Op::InvokeNative { .. }
                                    | Op::CancelNative { .. }
                            ) {
                                root.emit_command_ack(
                                    command.meta,
                                    command.operation.kind(),
                                    false,
                                    Some(
                                        "This system command is unavailable in the browser host"
                                            .into(),
                                    ),
                                    None,
                                );
                                return Ok(());
                            }
                        }
                        root.apply_decoded_message_in_window(message, window, cx)
                    })
                })
            })
            .map_err(js_error)?
            .map_err(js_error)
    })
}

#[wasm_bindgen]
pub fn drain_events() -> Vec<u8> {
    HOST.with(|host| {
        host.borrow()
            .as_ref()
            .map(|host| std::mem::take(&mut *host.adapter.events.lock().unwrap()))
            .unwrap_or_default()
    })
}

#[wasm_bindgen]
pub fn stop() {
    HOST.with(|host| {
        host.borrow_mut().take();
    });
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
