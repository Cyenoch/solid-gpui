//! Optional gpui-component integration. The host and TypeScript bindings use this module.
mod charts;
mod choices;
mod command_palette;
mod content;
mod data_list;
mod data_table;
mod dock;
mod dock_layout;
mod extra_elements;
mod groups;
#[cfg(feature = "gpui-component")]
pub mod host;
mod input;
mod menus;
mod native_menu;
mod notifications;
mod overlays;
mod pickers;
mod plot;
mod plot_math;
mod popups;
mod primitives;
mod resizable;
mod rich_text;
mod scroll_views;
mod settings;
mod sidebar;
mod table_elements;
#[cfg(test)]
mod test_support;
mod theme;
mod tree_view;
mod validation;
mod value_controls;
use crate::native::{Deserialize, Serialize, TS};

#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ControlSize {
    Xsmall,
    Small,
    #[default]
    Medium,
    Large,
}
impl From<ControlSize> for gpui_component::Size {
    fn from(v: ControlSize) -> Self {
        match v {
            ControlSize::Xsmall => Self::XSmall,
            ControlSize::Small => Self::Small,
            ControlSize::Medium => Self::Medium,
            ControlSize::Large => Self::Large,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ButtonVariant {
    #[default]
    Default,
    Primary,
    Secondary,
    Danger,
    Info,
    Success,
    Warning,
    Ghost,
    Link,
    Text,
}
impl From<ButtonVariant> for gpui_component::button::ButtonVariant {
    fn from(v: ButtonVariant) -> Self {
        match v {
            ButtonVariant::Default => Self::Default,
            ButtonVariant::Primary => Self::Primary,
            ButtonVariant::Secondary => Self::Secondary,
            ButtonVariant::Danger => Self::Danger,
            ButtonVariant::Info => Self::Info,
            ButtonVariant::Success => Self::Success,
            ButtonVariant::Warning => Self::Warning,
            ButtonVariant::Ghost => Self::Ghost,
            ButtonVariant::Link => Self::Link,
            ButtonVariant::Text => Self::Text,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Rounded {
    None,
    Small,
    #[default]
    Medium,
    Large,
}
impl From<Rounded> for gpui_component::button::ButtonRounded {
    fn from(v: Rounded) -> Self {
        match v {
            Rounded::None => Self::None,
            Rounded::Small => Self::Small,
            Rounded::Medium => Self::Medium,
            Rounded::Large => Self::Large,
        }
    }
}

/// Domain validation belongs in the Rust type and runs before tree publication.
#[derive(Clone, Copy, Debug, Default, Serialize, TS)]
pub struct Percentage(pub f32);
impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = f32::deserialize(d)?;
        if value.is_finite() && (0.0..=100.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(serde::de::Error::custom(
                "percentage must be between 0 and 100",
            ))
        }
    }
}

#[crate::native_module(name = "gpui-component")]
mod controls {
    use super::*;
    use crate::native::{ElementContext, Event};
    use gpui::{IntoElement, ParentElement, Styled};
    use gpui_component::{Disableable, Selectable, Sizable};

    #[component]
    pub fn button(
        #[prop(default)] label: Option<String>,
        #[prop(default)] variant: ButtonVariant,
        #[prop(default)] disabled: bool,
        #[prop(default)] loading: bool,
        #[prop(default)] compact: bool,
        #[prop(default)] outline: bool,
        #[prop(default)] selected: bool,
        #[prop(default)] toggled: Option<bool>,
        #[prop(default)] rounded: Rounded,
        #[prop(default)] size: ControlSize,
        #[prop(default)] tab_index: i32,
        #[prop(default = true)] tab_stop: bool,
        #[prop(default)] accessibility_label: Option<String>,
        #[prop(default)] dropdown_caret: bool,
        #[prop(default)] tooltip: Option<String>,
        on_press: Event<()>,
        on_hover_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        use gpui_component::button::ButtonVariants;
        let mut button = gpui_component::button::Button::new(cx.id())
            .with_variant(variant.into())
            .disabled(disabled)
            .loading(loading)
            .selected(selected)
            .rounded(rounded)
            .with_size(size)
            .tab_index(tab_index as isize)
            .tab_stop(tab_stop)
            .dropdown_caret(dropdown_caret);
        if compact {
            button = button.compact();
        }
        if outline {
            button = button.outline();
        }
        if let Some(v) = toggled {
            button = button.toggled(v);
        }
        if let Some(v) = label {
            button = button.label(v);
        }
        if let Some(v) = accessibility_label {
            button = button.accessibility_label(v);
        }
        if let Some(v) = tooltip {
            button = button.tooltip(v);
        }
        if on_press.is_subscribed() {
            button = button.on_click(move |_, _, _| on_press.emit(()));
        }
        if on_hover_change.is_subscribed() {
            button = button.on_hover(move |v, _, _| on_hover_change.emit(*v));
        }
        cx.children()
            .fold(button, |button, child| button.child(child))
    }
    #[component]
    pub fn checkbox(
        #[prop(default)] checked: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] label: Option<String>,
        #[prop(default)] accessibility_label: Option<String>,
        #[prop(default)] size: ControlSize,
        #[prop(default = true)] tab_stop: bool,
        #[prop(default)] tab_index: i32,
        #[prop(default)] tooltip: Option<String>,
        on_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let mut control = gpui_component::checkbox::Checkbox::new(cx.id())
            .checked(checked)
            .disabled(disabled)
            .with_size(size)
            .tab_stop(tab_stop)
            .tab_index(tab_index as isize);
        if let Some(v) = label {
            control = control.label(v);
        }
        if let Some(v) = accessibility_label {
            control = control.accessibility_label(v);
        }
        if let Some(v) = tooltip {
            control = control.tooltip(v);
        }
        if on_change.is_subscribed() {
            control = control.on_click(move |v, _, _| on_change.emit(*v));
        }
        cx.children()
            .fold(control, |control, child| control.child(child))
    }
    #[component(children = false)]
    pub fn switch(
        #[prop(default)] checked: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] label: Option<String>,
        #[prop(default)] accessibility_label: Option<String>,
        #[prop(default)] size: ControlSize,
        #[prop(default)] tooltip: Option<String>,
        on_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let mut control = gpui_component::switch::Switch::new(cx.id())
            .checked(checked)
            .disabled(disabled)
            .with_size(size);
        if let Some(v) = label {
            control = control.label(v);
        }
        if let Some(v) = accessibility_label {
            control = control.accessibility_label(v);
        }
        if let Some(v) = tooltip {
            control = control.tooltip(v);
        }
        if on_change.is_subscribed() {
            control = control.on_click(move |v, _, _| on_change.emit(*v));
        }
        control
    }
    #[component(children = false)]
    pub fn progress(
        #[prop(default)] value: Percentage,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::progress::Progress::new(cx.id()).value(value.0)
    }
}
/// Initialize the shared component theme and interaction infrastructure.
pub fn initialize(cx: &mut gpui::App) {
    gpui_component::init(cx);
    theme::initialize(cx);
    cx.text_system()
        .add_fonts(vec![
            std::borrow::Cow::Borrowed(include_bytes!("../../fonts/MapleMono-Regular.ttf")),
            std::borrow::Cow::Borrowed(include_bytes!("../../fonts/MapleMono-Italic.ttf")),
            std::borrow::Cow::Borrowed(include_bytes!("../../fonts/MapleMono-Bold.ttf")),
            std::borrow::Cow::Borrowed(include_bytes!("../../fonts/MapleMono-BoldItalic.ttf")),
        ])
        .expect("bundled Maple Mono fonts must be valid");
    cx.global_mut::<gpui_component::Theme>().mono_font_family = "Maple Mono".into();
    gpui_component::Theme::sync_base(cx);
}

pub fn native_module() -> crate::native::ModuleDefinition {
    input::definitions()
        .into_iter()
        .chain(value_controls::definitions())
        .chain(pickers::definitions())
        .chain(choices::definitions())
        .chain(scroll_views::definitions())
        .chain(resizable::definitions())
        .chain(overlays::definitions())
        .chain(menus::definitions())
        .chain(settings::definitions())
        .chain(charts::definitions())
        .chain(plot::definitions())
        .fold(
            controls::native_module()
                .include(primitives::native_module())
                .include(groups::native_module())
                .include(table_elements::native_module())
                .include(content::native_module())
                .include(extra_elements::native_module())
                .include(sidebar::native_module())
                .include(overlays::native_module())
                .include(popups::native_module())
                .include(plot_math::native_module())
                .include(theme::native_module())
                .with_component(rich_text::definition())
                .with_component(data_list::definition())
                .with_component(tree_view::definition())
                .with_component(data_table::definition())
                .with_component(command_palette::definition())
                .with_component(dock::definition())
                .with_component(native_menu::definition())
                .with_component(notifications::definition()),
            |module, definition| module.with_component(definition),
        )
}
