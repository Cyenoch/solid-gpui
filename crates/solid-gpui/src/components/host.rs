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

use gpui_fps::{FpsMonitor, FpsOverlay, FrameRateMode};

struct ProviderContent {
    _appearance: gpui::Subscription,
    frame_monitor: Option<Entity<FpsMonitor>>,
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
            .line_height(
                gpui_base::Theme::global(cx)
                    .tokens
                    .typography
                    .md
                    .line_height,
            )
            .relative()
            .size_full()
            .child(self.solid_root.clone())
            .children(notification_layer)
            .children(sheet_layer)
            .children(dialog_layer)
            .children(self.frame_monitor.as_ref().map(|monitor| {
                FpsOverlay::new(monitor).offset(gpui::point(gpui::px(56.), gpui::px(8.)))
            }))
    }
}

type WindowOptionsFactory = Rc<dyn Fn(WindowOptions, &App) -> WindowOptions>;

/// Host profile that renders Solid GPUI through gpui-component's root and overlays.
#[derive(Clone)]
pub struct ComponentHost {
    modules: Rc<NativeModules>,
    window_options: Option<WindowOptionsFactory>,
    performance_monitor: bool,
}
impl ComponentHost {
    pub fn new(modules: Vec<ModuleDefinition>) -> Self {
        Self {
            modules: Rc::new(NativeModules::new(modules)),
            window_options: None,
            performance_monitor: false,
        }
    }
}
impl ComponentHost {
    /// Customize GPUI options using TitleBar::window_options() as the base for a custom title bar.
    pub fn with_window_options(
        mut self,
        configure: impl Fn(WindowOptions, &App) -> WindowOptions + 'static,
    ) -> Self {
        self.window_options = Some(Rc::new(configure));
        self
    }

    /// Enable the development performance overlay explicitly. Disabled by default in all builds.
    pub fn with_performance_monitor(mut self, enabled: bool) -> Self {
        self.performance_monitor = enabled;
        self
    }
}
impl Default for ComponentHost {
    fn default() -> Self {
        Self::new(vec![super::native_module()])
    }
}

impl HostProfile for ComponentHost {
    fn window_options(&self, options: WindowOptions, cx: &App) -> WindowOptions {
        match &self.window_options {
            Some(configure) => configure(options, cx),
            None => options,
        }
    }
    fn native_bindings(&self) -> Result<String, String> {
        self.modules
            .typescript()
            .map(|source| format!("{source}{}", crate::icons::typescript()))
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
        super::initialize(cx);
    }

    fn open_window(
        &self,
        options: WindowOptions,
        runtime: Arc<dyn RuntimeAdapter>,
        extensions: Rc<dyn ExtensionRegistry>,
        cx: &mut App,
    ) -> Result<(AnyWindowHandle, Entity<SolidRoot>), String> {
        let monitor_enabled = self.performance_monitor;
        let solid_root = Rc::new(RefCell::new(None));
        let solid_root_for_window = Rc::clone(&solid_root);
        let window = cx
            .open_window(options, move |window, cx| {
                let root = cx.new(|_| SolidRoot::with_extensions(runtime, extensions));
                *solid_root_for_window.borrow_mut() = Some(root.clone());
                let frame_monitor = monitor_enabled.then(|| {
                    cx.new(|cx| {
                        FpsMonitor::new(window, cx).frame_rate_mode(FrameRateMode::Observed)
                    })
                });
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
