use crate::host::{HostCapabilities, HostProfile};
use crate::native::{ModuleDefinition, NativeModules};
use gpui::{
    AnyWindowHandle, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, WindowOptions, div,
};
use gpui_component::Root;
use solid_gpui::{ExtensionRegistry, RuntimeAdapter, SolidRoot};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[cfg(feature = "frame-profile")]
#[path = "frame_profile.rs"]
mod frame_profile;

use gpui_performance::{MonitorCorner, PerformanceMonitor};

struct ProviderContent {
    _appearance: gpui::Subscription,
    frame_monitor: Option<Entity<PerformanceMonitor>>,
    solid_root: Entity<SolidRoot>,
    #[cfg(feature = "frame-profile")]
    frame_profile: frame_profile::FrameProfile,
}

impl Render for ProviderContent {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(feature = "frame-profile")]
        self.frame_profile.record(window);
        let notification_layer = Root::render_notification_layer(window, cx);
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        div()
            .relative()
            .size_full()
            .child(self.solid_root.clone())
            .children(notification_layer)
            .children(sheet_layer)
            .children(dialog_layer)
            .children(self.frame_monitor.clone())
    }
}

/// Host profile that renders Solid GPUI through gpui-component's root and overlays.
#[derive(Clone)]
pub struct ComponentHost {
    modules: Rc<NativeModules>,
}
impl ComponentHost {
    pub fn new(modules: Vec<ModuleDefinition>) -> Self {
        Self {
            modules: Rc::new(NativeModules::new(modules)),
        }
    }
}
impl Default for ComponentHost {
    fn default() -> Self {
        Self::new(vec![super::native_module()])
    }
}

impl HostProfile for ComponentHost {
    fn native_bindings(&self) -> Result<String, String> {
        self.modules.typescript()
    }
    fn capabilities(&self) -> HostCapabilities {
        HostCapabilities {
            // gpui-component owns the global system notification callback. The
            // profile rejects ShowNotification rather than replacing or chaining it.
            notification_responses: false,
            set_keybindings: true,
        }
    }

    fn extension_registry(&self) -> Rc<dyn ExtensionRegistry> {
        self.modules.clone()
    }

    fn initialize(&mut self, cx: &mut App) {
        gpui_component::init(cx);
        super::theme::initialize(cx);
    }

    fn open_window(
        &self,
        options: WindowOptions,
        runtime: Arc<dyn RuntimeAdapter>,
        extensions: Rc<dyn ExtensionRegistry>,
        cx: &mut App,
    ) -> Result<(AnyWindowHandle, Entity<SolidRoot>), String> {
        let solid_root = Rc::new(RefCell::new(None));
        let solid_root_for_window = Rc::clone(&solid_root);
        let window = cx
            .open_window(options, move |window, cx| {
                let root = cx.new(|_| SolidRoot::with_extensions(runtime, extensions));
                *solid_root_for_window.borrow_mut() = Some(root.clone());
                let frame_monitor = (std::env::var("SOLID_GPUI_PERF_MONITOR").as_deref()
                    != Ok("0"))
                .then(|| cx.new(|cx| PerformanceMonitor::new(MonitorCorner::TopRight, cx)));
                let content = cx.new(|cx| ProviderContent {
                    _appearance: cx.observe_window_appearance(window, |_, window, cx| {
                        super::theme::sync_system(window, cx);
                    }),
                    solid_root: root,
                    frame_monitor,
                    #[cfg(feature = "frame-profile")]
                    frame_profile: frame_profile::FrameProfile::new(window),
                });
                // Root is deliberately the first and actual window view. Its
                // content view composes SolidRoot with provider overlays.
                cx.new(|cx| Root::new(content, window, cx))
            })
            .map_err(|error| format!("failed to open GPUI window: {error}"))?;
        let solid_root = solid_root
            .borrow()
            .clone()
            .ok_or_else(|| "GPUI did not return a SolidRoot entity".to_owned())?;
        Ok((window.into(), solid_root))
    }

    fn restore_keybindings(
        &self,
        baseline: &[gpui::KeyBinding],
        dynamic: Vec<gpui::KeyBinding>,
        cx: &mut App,
    ) {
        cx.clear_key_bindings();
        cx.bind_keys(baseline.iter().cloned().chain(dynamic));
    }

    fn rejected_command_reason(&self, command: solid_gpui::CommandKind) -> Option<&'static str> {
        (command == solid_gpui::CommandKind::ShowNotification).then_some(
            "ShowNotification is unavailable in the gpui-component host because gpui-component owns the global system notification callback",
        )
    }
}

/// Run the provider host binary with the gpui-component profile.
pub fn run() {
    crate::host::run_with_profile(ComponentHost::default());
}

#[cfg(test)]
#[path = "native_call_tests.rs"]
mod native_call_tests;
#[cfg(test)]
#[path = "host_tests.rs"]
mod tests;
