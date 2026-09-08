//! Application-wide native appearance. GPUI Component themes are App globals.
use super::primitives::Color;
use super::validation::LogicalPixels;
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
struct Appearance {
    mode: ThemeMode,
    application: Option<ApplicationTheme>,
}
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
    if cx.global::<Appearance>().mode == ThemeMode::System {
        Theme::sync_system_appearance(Some(window), cx);
        apply_application(cx);
    }
}
fn state(_: (), _: &mut Window, cx: &mut App) -> Result<ThemeState, String> {
    Ok(ThemeState {
        mode: cx.global::<Appearance>().mode,
        dark: Theme::global(cx).is_dark(),
    })
}
fn change(mode: ThemeMode, window: &mut Window, cx: &mut App) -> Result<ThemeState, String> {
    cx.global_mut::<Appearance>().mode = mode;
    match mode {
        ThemeMode::System => Theme::sync_system_appearance(Some(window), cx),
        ThemeMode::Light => Theme::change(gpui_component::ThemeMode::Light, Some(window), cx),
        ThemeMode::Dark => Theme::change(gpui_component::ThemeMode::Dark, Some(window), cx),
    }
    apply_application(cx);
    cx.refresh_windows();
    state((), window, cx)
}
pub(super) fn native_module() -> ModuleDefinition {
    ModuleDefinition::new(
        "theme",
        vec![],
        vec![
            CommandDefinition::foreground("getMotionPreference", |(): (), _, cx| {
                crate::motion::get(cx)
            }),
            CommandDefinition::foreground(
                "setMotionPreference",
                |mode: crate::motion::MotionMode, _, cx| crate::motion::set(mode, cx),
            ),
            CommandDefinition::foreground("getTheme", state),
            CommandDefinition::foreground("setTheme", change),
            CommandDefinition::foreground("setApplicationTheme", set_application),
        ],
    )
    .with_contract(include_str!("theme.rs"))
}

/// Application-wide overrides of the linked native component theme.
/// Omitted tokens use the selected light/dark base; each call replaces previous overrides.
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationTheme {
    pub components: Option<ApplicationComponentMetrics>,
    pub input_background: Option<Color>,
    pub colors: ApplicationThemeColors,
    pub font_family: Option<String>,
    pub font_size: Option<LogicalPixels>,
    pub line_height: Option<LogicalPixels>,
    pub radius: Option<LogicalPixels>,
    pub radius_lg: Option<LogicalPixels>,
}

/// Native component tokens, including interaction states and overlay chrome.
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationThemeColors {
    pub accent: Option<Color>,
    pub accent_foreground: Option<Color>,
    pub accordion: Option<Color>,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub button: Option<Color>,
    pub button_active: Option<Color>,
    pub button_foreground: Option<Color>,
    pub button_hover: Option<Color>,
    pub button_danger: Option<Color>,
    pub button_danger_active: Option<Color>,
    pub button_danger_foreground: Option<Color>,
    pub button_danger_hover: Option<Color>,
    pub button_info: Option<Color>,
    pub button_info_active: Option<Color>,
    pub button_info_foreground: Option<Color>,
    pub button_info_hover: Option<Color>,
    pub button_primary: Option<Color>,
    pub button_primary_active: Option<Color>,
    pub button_primary_foreground: Option<Color>,
    pub button_primary_hover: Option<Color>,
    pub button_secondary: Option<Color>,
    pub button_secondary_active: Option<Color>,
    pub button_secondary_foreground: Option<Color>,
    pub button_secondary_hover: Option<Color>,
    pub button_success: Option<Color>,
    pub button_success_active: Option<Color>,
    pub button_success_foreground: Option<Color>,
    pub button_success_hover: Option<Color>,
    pub button_warning: Option<Color>,
    pub button_warning_active: Option<Color>,
    pub button_warning_foreground: Option<Color>,
    pub button_warning_hover: Option<Color>,
    pub group_box: Option<Color>,
    pub group_box_foreground: Option<Color>,
    pub caret: Option<Color>,
    pub chart_1: Option<Color>,
    pub chart_2: Option<Color>,
    pub chart_3: Option<Color>,
    pub chart_4: Option<Color>,
    pub chart_5: Option<Color>,
    pub chart_bullish: Option<Color>,
    pub chart_bearish: Option<Color>,
    pub danger: Option<Color>,
    pub danger_active: Option<Color>,
    pub danger_foreground: Option<Color>,
    pub danger_hover: Option<Color>,
    pub description_list_label: Option<Color>,
    pub description_list_label_foreground: Option<Color>,
    pub drag_border: Option<Color>,
    pub drop_target: Option<Color>,
    pub foreground: Option<Color>,
    pub info: Option<Color>,
    pub info_active: Option<Color>,
    pub info_foreground: Option<Color>,
    pub info_hover: Option<Color>,
    pub input: Option<Color>,
    pub link: Option<Color>,
    pub link_active: Option<Color>,
    pub link_hover: Option<Color>,
    pub list: Option<Color>,
    pub list_active: Option<Color>,
    pub list_active_border: Option<Color>,
    pub list_even: Option<Color>,
    pub list_head: Option<Color>,
    pub list_hover: Option<Color>,
    pub muted: Option<Color>,
    pub muted_foreground: Option<Color>,
    pub popover: Option<Color>,
    pub popover_foreground: Option<Color>,
    pub primary: Option<Color>,
    pub primary_active: Option<Color>,
    pub primary_foreground: Option<Color>,
    pub primary_hover: Option<Color>,
    pub progress_bar: Option<Color>,
    pub ring: Option<Color>,
    pub scrollbar: Option<Color>,
    pub scrollbar_thumb: Option<Color>,
    pub scrollbar_thumb_hover: Option<Color>,
    pub secondary: Option<Color>,
    pub secondary_active: Option<Color>,
    pub secondary_foreground: Option<Color>,
    pub secondary_hover: Option<Color>,
    pub selection: Option<Color>,
    pub sidebar: Option<Color>,
    pub sidebar_accent: Option<Color>,
    pub sidebar_accent_foreground: Option<Color>,
    pub sidebar_border: Option<Color>,
    pub sidebar_foreground: Option<Color>,
    pub sidebar_primary: Option<Color>,
    pub sidebar_primary_foreground: Option<Color>,
    pub skeleton: Option<Color>,
    pub slider_bar: Option<Color>,
    pub slider_thumb: Option<Color>,
    pub success: Option<Color>,
    pub success_foreground: Option<Color>,
    pub success_hover: Option<Color>,
    pub success_active: Option<Color>,
    pub switch: Option<Color>,
    pub switch_thumb: Option<Color>,
    pub tab: Option<Color>,
    pub tab_active: Option<Color>,
    pub tab_active_foreground: Option<Color>,
    pub tab_bar: Option<Color>,
    pub tab_bar_segmented: Option<Color>,
    pub tab_foreground: Option<Color>,
    pub table: Option<Color>,
    pub table_active: Option<Color>,
    pub table_active_border: Option<Color>,
    pub table_even: Option<Color>,
    pub table_head: Option<Color>,
    pub table_head_foreground: Option<Color>,
    pub table_foot: Option<Color>,
    pub table_foot_foreground: Option<Color>,
    pub table_hover: Option<Color>,
    pub table_row_border: Option<Color>,
    pub title_bar: Option<Color>,
    pub title_bar_border: Option<Color>,
    pub status_bar: Option<Color>,
    pub status_bar_border: Option<Color>,
    pub tiles: Option<Color>,
    pub warning: Option<Color>,
    pub warning_active: Option<Color>,
    pub warning_hover: Option<Color>,
    pub warning_foreground: Option<Color>,
    pub overlay: Option<Color>,
    pub window_border: Option<Color>,
    pub red: Option<Color>,
    pub red_light: Option<Color>,
    pub green: Option<Color>,
    pub green_light: Option<Color>,
    pub blue: Option<Color>,
    pub blue_light: Option<Color>,
    pub yellow: Option<Color>,
    pub yellow_light: Option<Color>,
    pub magenta: Option<Color>,
    pub magenta_light: Option<Color>,
    pub cyan: Option<Color>,
    pub cyan_light: Option<Color>,
}

fn apply_application(cx: &mut App) {
    let Some(config) = cx.global::<Appearance>().application.clone() else {
        return;
    };
    let theme = Theme::global_mut(cx);
    if let Some(value) = config.colors.accent {
        theme.colors.accent = value.native();
    }
    if let Some(value) = config.colors.accent_foreground {
        theme.colors.accent_foreground = value.native();
    }
    if let Some(value) = config.colors.accordion {
        theme.colors.accordion = value.native();
    }
    if let Some(value) = config.colors.background {
        theme.colors.background = value.native();
    }
    if let Some(value) = config.colors.border {
        theme.colors.border = value.native();
    }
    if let Some(value) = config.colors.button {
        theme.colors.button = value.native();
    }
    if let Some(value) = config.colors.button_active {
        theme.colors.button_active = value.native();
    }
    if let Some(value) = config.colors.button_foreground {
        theme.colors.button_foreground = value.native();
    }
    if let Some(value) = config.colors.button_hover {
        theme.colors.button_hover = value.native();
    }
    if let Some(value) = config.colors.button_danger {
        theme.colors.button_danger = value.native();
    }
    if let Some(value) = config.colors.button_danger_active {
        theme.colors.button_danger_active = value.native();
    }
    if let Some(value) = config.colors.button_danger_foreground {
        theme.colors.button_danger_foreground = value.native();
    }
    if let Some(value) = config.colors.button_danger_hover {
        theme.colors.button_danger_hover = value.native();
    }
    if let Some(value) = config.colors.button_info {
        theme.colors.button_info = value.native();
    }
    if let Some(value) = config.colors.button_info_active {
        theme.colors.button_info_active = value.native();
    }
    if let Some(value) = config.colors.button_info_foreground {
        theme.colors.button_info_foreground = value.native();
    }
    if let Some(value) = config.colors.button_info_hover {
        theme.colors.button_info_hover = value.native();
    }
    if let Some(value) = config.colors.button_primary {
        theme.colors.button_primary = value.native();
    }
    if let Some(value) = config.colors.button_primary_active {
        theme.colors.button_primary_active = value.native();
    }
    if let Some(value) = config.colors.button_primary_foreground {
        theme.colors.button_primary_foreground = value.native();
    }
    if let Some(value) = config.colors.button_primary_hover {
        theme.colors.button_primary_hover = value.native();
    }
    if let Some(value) = config.colors.button_secondary {
        theme.colors.button_secondary = value.native();
    }
    if let Some(value) = config.colors.button_secondary_active {
        theme.colors.button_secondary_active = value.native();
    }
    if let Some(value) = config.colors.button_secondary_foreground {
        theme.colors.button_secondary_foreground = value.native();
    }
    if let Some(value) = config.colors.button_secondary_hover {
        theme.colors.button_secondary_hover = value.native();
    }
    if let Some(value) = config.colors.button_success {
        theme.colors.button_success = value.native();
    }
    if let Some(value) = config.colors.button_success_active {
        theme.colors.button_success_active = value.native();
    }
    if let Some(value) = config.colors.button_success_foreground {
        theme.colors.button_success_foreground = value.native();
    }
    if let Some(value) = config.colors.button_success_hover {
        theme.colors.button_success_hover = value.native();
    }
    if let Some(value) = config.colors.button_warning {
        theme.colors.button_warning = value.native();
    }
    if let Some(value) = config.colors.button_warning_active {
        theme.colors.button_warning_active = value.native();
    }
    if let Some(value) = config.colors.button_warning_foreground {
        theme.colors.button_warning_foreground = value.native();
    }
    if let Some(value) = config.colors.button_warning_hover {
        theme.colors.button_warning_hover = value.native();
    }
    if let Some(value) = config.colors.group_box {
        theme.colors.group_box = value.native();
    }
    if let Some(value) = config.colors.group_box_foreground {
        theme.colors.group_box_foreground = value.native();
    }
    if let Some(value) = config.colors.caret {
        theme.colors.caret = value.native();
    }
    if let Some(value) = config.colors.chart_1 {
        theme.colors.chart_1 = value.native();
    }
    if let Some(value) = config.colors.chart_2 {
        theme.colors.chart_2 = value.native();
    }
    if let Some(value) = config.colors.chart_3 {
        theme.colors.chart_3 = value.native();
    }
    if let Some(value) = config.colors.chart_4 {
        theme.colors.chart_4 = value.native();
    }
    if let Some(value) = config.colors.chart_5 {
        theme.colors.chart_5 = value.native();
    }
    if let Some(value) = config.colors.chart_bullish {
        theme.colors.chart_bullish = value.native();
    }
    if let Some(value) = config.colors.chart_bearish {
        theme.colors.chart_bearish = value.native();
    }
    if let Some(value) = config.colors.danger {
        theme.colors.danger = value.native();
    }
    if let Some(value) = config.colors.danger_active {
        theme.colors.danger_active = value.native();
    }
    if let Some(value) = config.colors.danger_foreground {
        theme.colors.danger_foreground = value.native();
    }
    if let Some(value) = config.colors.danger_hover {
        theme.colors.danger_hover = value.native();
    }
    if let Some(value) = config.colors.description_list_label {
        theme.colors.description_list_label = value.native();
    }
    if let Some(value) = config.colors.description_list_label_foreground {
        theme.colors.description_list_label_foreground = value.native();
    }
    if let Some(value) = config.colors.drag_border {
        theme.colors.drag_border = value.native();
    }
    if let Some(value) = config.colors.drop_target {
        theme.colors.drop_target = value.native();
    }
    if let Some(value) = config.colors.foreground {
        theme.colors.foreground = value.native();
    }
    if let Some(value) = config.colors.info {
        theme.colors.info = value.native();
    }
    if let Some(value) = config.colors.info_active {
        theme.colors.info_active = value.native();
    }
    if let Some(value) = config.colors.info_foreground {
        theme.colors.info_foreground = value.native();
    }
    if let Some(value) = config.colors.info_hover {
        theme.colors.info_hover = value.native();
    }
    if let Some(value) = config.colors.input {
        theme.colors.input = value.native();
    }
    if let Some(value) = config.colors.link {
        theme.colors.link = value.native();
    }
    if let Some(value) = config.colors.link_active {
        theme.colors.link_active = value.native();
    }
    if let Some(value) = config.colors.link_hover {
        theme.colors.link_hover = value.native();
    }
    if let Some(value) = config.colors.list {
        theme.colors.list = value.native();
    }
    if let Some(value) = config.colors.list_active {
        theme.colors.list_active = value.native();
    }
    if let Some(value) = config.colors.list_active_border {
        theme.colors.list_active_border = value.native();
    }
    if let Some(value) = config.colors.list_even {
        theme.colors.list_even = value.native();
    }
    if let Some(value) = config.colors.list_head {
        theme.colors.list_head = value.native();
    }
    if let Some(value) = config.colors.list_hover {
        theme.colors.list_hover = value.native();
    }
    if let Some(value) = config.colors.muted {
        theme.colors.muted = value.native();
    }
    if let Some(value) = config.colors.muted_foreground {
        theme.colors.muted_foreground = value.native();
    }
    if let Some(value) = config.colors.popover {
        theme.colors.popover = value.native();
    }
    if let Some(value) = config.colors.popover_foreground {
        theme.colors.popover_foreground = value.native();
    }
    if let Some(value) = config.colors.primary {
        theme.colors.primary = value.native();
    }
    if let Some(value) = config.colors.primary_active {
        theme.colors.primary_active = value.native();
    }
    if let Some(value) = config.colors.primary_foreground {
        theme.colors.primary_foreground = value.native();
    }
    if let Some(value) = config.colors.primary_hover {
        theme.colors.primary_hover = value.native();
    }
    if let Some(value) = config.colors.progress_bar {
        theme.colors.progress_bar = value.native();
    }
    if let Some(value) = config.colors.ring {
        theme.colors.ring = value.native();
    }
    if let Some(value) = config.colors.scrollbar {
        theme.colors.scrollbar = value.native();
    }
    if let Some(value) = config.colors.scrollbar_thumb {
        theme.colors.scrollbar_thumb = value.native();
    }
    if let Some(value) = config.colors.scrollbar_thumb_hover {
        theme.colors.scrollbar_thumb_hover = value.native();
    }
    if let Some(value) = config.colors.secondary {
        theme.colors.secondary = value.native();
    }
    if let Some(value) = config.colors.secondary_active {
        theme.colors.secondary_active = value.native();
    }
    if let Some(value) = config.colors.secondary_foreground {
        theme.colors.secondary_foreground = value.native();
    }
    if let Some(value) = config.colors.secondary_hover {
        theme.colors.secondary_hover = value.native();
    }
    if let Some(value) = config.colors.selection {
        theme.colors.selection = value.native();
    }
    if let Some(value) = config.colors.sidebar {
        theme.colors.sidebar = value.native();
    }
    if let Some(value) = config.colors.sidebar_accent {
        theme.colors.sidebar_accent = value.native();
    }
    if let Some(value) = config.colors.sidebar_accent_foreground {
        theme.colors.sidebar_accent_foreground = value.native();
    }
    if let Some(value) = config.colors.sidebar_border {
        theme.colors.sidebar_border = value.native();
    }
    if let Some(value) = config.colors.sidebar_foreground {
        theme.colors.sidebar_foreground = value.native();
    }
    if let Some(value) = config.colors.sidebar_primary {
        theme.colors.sidebar_primary = value.native();
    }
    if let Some(value) = config.colors.sidebar_primary_foreground {
        theme.colors.sidebar_primary_foreground = value.native();
    }
    if let Some(value) = config.colors.skeleton {
        theme.colors.skeleton = value.native();
    }
    if let Some(value) = config.colors.slider_bar {
        theme.colors.slider_bar = value.native();
    }
    if let Some(value) = config.colors.slider_thumb {
        theme.colors.slider_thumb = value.native();
    }
    if let Some(value) = config.colors.success {
        theme.colors.success = value.native();
    }
    if let Some(value) = config.colors.success_foreground {
        theme.colors.success_foreground = value.native();
    }
    if let Some(value) = config.colors.success_hover {
        theme.colors.success_hover = value.native();
    }
    if let Some(value) = config.colors.success_active {
        theme.colors.success_active = value.native();
    }
    if let Some(value) = config.colors.switch {
        theme.colors.switch = value.native();
    }
    if let Some(value) = config.colors.switch_thumb {
        theme.colors.switch_thumb = value.native();
    }
    if let Some(value) = config.colors.tab {
        theme.colors.tab = value.native();
    }
    if let Some(value) = config.colors.tab_active {
        theme.colors.tab_active = value.native();
    }
    if let Some(value) = config.colors.tab_active_foreground {
        theme.colors.tab_active_foreground = value.native();
    }
    if let Some(value) = config.colors.tab_bar {
        theme.colors.tab_bar = value.native();
    }
    if let Some(value) = config.colors.tab_bar_segmented {
        theme.colors.tab_bar_segmented = value.native();
    }
    if let Some(value) = config.colors.tab_foreground {
        theme.colors.tab_foreground = value.native();
    }
    if let Some(value) = config.colors.table {
        theme.colors.table = value.native();
    }
    if let Some(value) = config.colors.table_active {
        theme.colors.table_active = value.native();
    }
    if let Some(value) = config.colors.table_active_border {
        theme.colors.table_active_border = value.native();
    }
    if let Some(value) = config.colors.table_even {
        theme.colors.table_even = value.native();
    }
    if let Some(value) = config.colors.table_head {
        theme.colors.table_head = value.native();
    }
    if let Some(value) = config.colors.table_head_foreground {
        theme.colors.table_head_foreground = value.native();
    }
    if let Some(value) = config.colors.table_foot {
        theme.colors.table_foot = value.native();
    }
    if let Some(value) = config.colors.table_foot_foreground {
        theme.colors.table_foot_foreground = value.native();
    }
    if let Some(value) = config.colors.table_hover {
        theme.colors.table_hover = value.native();
    }
    if let Some(value) = config.colors.table_row_border {
        theme.colors.table_row_border = value.native();
    }
    if let Some(value) = config.colors.title_bar {
        theme.colors.title_bar = value.native();
    }
    if let Some(value) = config.colors.title_bar_border {
        theme.colors.title_bar_border = value.native();
    }
    if let Some(value) = config.colors.status_bar {
        theme.colors.status_bar = value.native();
    }
    if let Some(value) = config.colors.status_bar_border {
        theme.colors.status_bar_border = value.native();
    }
    if let Some(value) = config.colors.tiles {
        theme.colors.tiles = value.native();
    }
    if let Some(value) = config.colors.warning {
        theme.colors.warning = value.native();
    }
    if let Some(value) = config.colors.warning_active {
        theme.colors.warning_active = value.native();
    }
    if let Some(value) = config.colors.warning_hover {
        theme.colors.warning_hover = value.native();
    }
    if let Some(value) = config.colors.warning_foreground {
        theme.colors.warning_foreground = value.native();
    }
    if let Some(value) = config.colors.overlay {
        theme.colors.overlay = value.native();
    }
    if let Some(value) = config.colors.window_border {
        theme.colors.window_border = value.native();
    }
    if let Some(value) = config.colors.red {
        theme.colors.red = value.native();
    }
    if let Some(value) = config.colors.red_light {
        theme.colors.red_light = value.native();
    }
    if let Some(value) = config.colors.green {
        theme.colors.green = value.native();
    }
    if let Some(value) = config.colors.green_light {
        theme.colors.green_light = value.native();
    }
    if let Some(value) = config.colors.blue {
        theme.colors.blue = value.native();
    }
    if let Some(value) = config.colors.blue_light {
        theme.colors.blue_light = value.native();
    }
    if let Some(value) = config.colors.yellow {
        theme.colors.yellow = value.native();
    }
    if let Some(value) = config.colors.yellow_light {
        theme.colors.yellow_light = value.native();
    }
    if let Some(value) = config.colors.magenta {
        theme.colors.magenta = value.native();
    }
    if let Some(value) = config.colors.magenta_light {
        theme.colors.magenta_light = value.native();
    }
    if let Some(value) = config.colors.cyan {
        theme.colors.cyan = value.native();
    }
    if let Some(value) = config.colors.cyan_light {
        theme.colors.cyan_light = value.native();
    }
    theme.tokens = (&theme.colors).into();
    theme.input_background_override = config.input_background.map(|color| color.native());
    theme.component_metrics = config.components.unwrap_or_default().native();
    if let Some(value) = config.font_family {
        theme.font_family = value.into();
    }
    if let Some(value) = config.font_size {
        theme.font_size = gpui::px(value.0);
    }
    if let Some(value) = config.radius {
        theme.radius = gpui::px(value.0);
    }
    if let Some(value) = config.radius_lg {
        theme.radius_lg = gpui::px(value.0);
    }
    Theme::sync_base(cx);
    if let Some(value) = config.line_height {
        gpui_base::Theme::global_mut(cx)
            .tokens
            .typography
            .md
            .line_height = gpui::px(value.0);
    }
}

fn set_application(
    config: ApplicationTheme,
    window: &mut Window,
    cx: &mut App,
) -> Result<ThemeState, String> {
    if config.font_family.as_ref().is_some_and(|value| {
        value.is_empty() || value.len() > 256 || value.chars().any(char::is_control)
    }) {
        return Err("fontFamily must be a non-empty font name of at most 256 bytes".into());
    }
    if config.font_size.is_some_and(|value| value.0 == 0.)
        || config.line_height.is_some_and(|value| value.0 == 0.)
    {
        return Err("fontSize and lineHeight must be positive".into());
    }
    if let Some(metrics) = &config.components {
        for control in [
            &metrics.button,
            &metrics.input,
            &metrics.select,
            &metrics.tag,
            &metrics.menu,
            &metrics.dialog,
        ] {
            if control.font_size.is_some_and(|v| v.0 == 0.)
                || control.line_height.is_some_and(|v| v.0 == 0.)
            {
                return Err("component fontSize and lineHeight must be positive".into());
            }
        }
    }
    cx.global_mut::<Appearance>().application = Some(config);
    let mode = cx.global::<Appearance>().mode;
    change(mode, window, cx)
}

/// Exact logical dimensions shared by every instance of a native control family.
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationControlMetrics {
    pub height: Option<LogicalPixels>,
    pub font_size: Option<LogicalPixels>,
    pub line_height: Option<LogicalPixels>,
    pub padding_x: Option<LogicalPixels>,
    pub padding_y: Option<LogicalPixels>,
    pub radius: Option<LogicalPixels>,
}
impl ApplicationControlMetrics {
    fn native(self) -> gpui_component::theme::ComponentMetrics {
        gpui_component::theme::ComponentMetrics {
            height: self.height.map(|v| gpui::px(v.0)),
            font_size: self.font_size.map(|v| gpui::px(v.0)),
            line_height: self.line_height.map(|v| gpui::px(v.0)),
            padding_x: self.padding_x.map(|v| gpui::px(v.0)),
            padding_y: self.padding_y.map(|v| gpui::px(v.0)),
            radius: self.radius.map(|v| gpui::px(v.0)),
        }
    }
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ApplicationComponentMetrics {
    pub button: ApplicationControlMetrics,
    pub input: ApplicationControlMetrics,
    pub select: ApplicationControlMetrics,
    pub tag: ApplicationControlMetrics,
    pub menu: ApplicationControlMetrics,
    pub dialog: ApplicationControlMetrics,
}
impl ApplicationComponentMetrics {
    fn native(self) -> gpui_component::theme::ComponentMetricsSet {
        gpui_component::theme::ComponentMetricsSet {
            button: self.button.native(),
            input: self.input.native(),
            select: self.select.native(),
            tag: self.tag.native(),
            menu: self.menu.native(),
            dialog: self.dialog.native(),
        }
    }
}
