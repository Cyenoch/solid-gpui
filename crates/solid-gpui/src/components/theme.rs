//! Application-wide native appearance. GPUI Component themes are App globals.
use crate::native::{CommandDefinition, ModuleDefinition};
use gpui::{App, Global, Window};
use gpui_component::Theme;

#[crate::native_type]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}
#[derive(Default)]
struct Appearance(ThemeMode);
impl Global for Appearance {}

#[crate::native_type]
pub struct ThemeState {
    pub mode: ThemeMode,
    pub dark: bool,
}

pub(super) fn initialize(cx: &mut App) {
    cx.set_global(Appearance::default());
    Theme::sync_system_appearance(None, cx);
}
pub(super) fn sync_system(window: &mut Window, cx: &mut App) {
    if cx.global::<Appearance>().0 == ThemeMode::System {
        Theme::sync_system_appearance(Some(window), cx);
    }
}
fn state(_: (), _: &mut Window, cx: &mut App) -> Result<ThemeState, String> {
    Ok(ThemeState {
        mode: cx.global::<Appearance>().0,
        dark: Theme::global(cx).is_dark(),
    })
}
fn change(mode: ThemeMode, window: &mut Window, cx: &mut App) -> Result<ThemeState, String> {
    cx.global_mut::<Appearance>().0 = mode;
    match mode {
        ThemeMode::System => Theme::sync_system_appearance(Some(window), cx),
        ThemeMode::Light => Theme::change(gpui_component::ThemeMode::Light, Some(window), cx),
        ThemeMode::Dark => Theme::change(gpui_component::ThemeMode::Dark, Some(window), cx),
    }
    cx.refresh_windows();
    state((), window, cx)
}
pub(super) fn native_module() -> ModuleDefinition {
    ModuleDefinition::new(
        "theme",
        vec![],
        vec![
            CommandDefinition::foreground("getTheme", state),
            CommandDefinition::foreground("setTheme", change),
        ],
    )
    .with_contract(include_str!("theme.rs"))
}
