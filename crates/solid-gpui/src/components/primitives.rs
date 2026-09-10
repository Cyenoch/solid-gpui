//! Small native controls. Events carry semantic values; all interaction remains GPUI-owned.
use super::icon_source::ComponentIcon;
use super::validation::{PageButtons, RatingMax};
use super::{ControlSize, Percentage};
use crate::native::{Deserialize, Serialize, TS};

/// An sRGB color in #RRGGBB or #RRGGBBAA form, validated before publication.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
pub struct Color(String);
impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = String::deserialize(d)?;
        if !matches!(value.len(), 7 | 9)
            || !value.starts_with('#')
            || !value[1..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(serde::de::Error::custom(
                "color must be #RRGGBB or #RRGGBBAA",
            ));
        }
        Ok(Self(value))
    }
}
impl Color {
    pub(super) fn from_native(value: gpui::Hsla) -> Self {
        use gpui_component::Colorize;
        Self(value.to_hex())
    }
    pub(super) fn native(&self) -> gpui::Hsla {
        use gpui_component::Colorize;
        gpui::Hsla::parse_hex(&self.0).expect("validated color")
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}
impl From<Orientation> for gpui::Axis {
    fn from(value: Orientation) -> Self {
        match value {
            Orientation::Horizontal => Self::Horizontal,
            Orientation::Vertical => Self::Vertical,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertVariant {
    #[default]
    Default,
    Info,
    Success,
    Warning,
    Error,
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GroupBoxVariant {
    #[default]
    Normal,
    Fill,
    Outline,
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TagVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Success,
    Warning,
    Info,
}

#[crate::native_module(name = "gpui-component")]
mod exports {
    use super::*;
    use crate::native::{ElementContext, Event, NativeSlot};
    use gpui::{IntoElement, ParentElement, Styled, prelude::FluentBuilder};
    use gpui_component::{Disableable, Sizable};

    #[component(children = false)]
    pub fn alert(
        message: String,
        #[prop(default)] title: Option<String>,
        #[prop(default)] variant: AlertVariant,
        #[prop(default)] banner: bool,
        #[prop(default = true)] visible: bool,
        #[prop(default)] icon: Option<ComponentIcon>,
        #[prop(default)] size: ControlSize,
        on_close: Event<()>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let variant = match variant {
            AlertVariant::Default => gpui_component::alert::AlertVariant::Default,
            AlertVariant::Info => gpui_component::alert::AlertVariant::Info,
            AlertVariant::Success => gpui_component::alert::AlertVariant::Success,
            AlertVariant::Warning => gpui_component::alert::AlertVariant::Warning,
            AlertVariant::Error => gpui_component::alert::AlertVariant::Error,
        };
        gpui_component::alert::Alert::new(cx.id(), message)
            .with_variant(variant)
            .visible(visible)
            .with_size(size)
            .when(banner, |v| v.banner())
            .when_some(title, |v, title| v.title(title))
            .when_some(icon, |v, path| v.icon(path.native()))
            .when(on_close.is_subscribed(), |v| {
                v.on_close(move |_, _, _| on_close.emit(()))
            })
    }
    #[component(children = false)]
    pub fn avatar(
        #[prop(default)] src: Option<String>,
        #[prop(default)] name: Option<String>,
        #[prop(default)] placeholder: Option<ComponentIcon>,
        #[prop(default)] size: ControlSize,
    ) -> impl IntoElement + Styled {
        gpui_component::avatar::Avatar::new()
            .with_size(size)
            .when_some(src, |v, src| v.src(src))
            .when_some(name, |v, name| v.name(name))
            .when_some(placeholder, |v, path| v.placeholder(path.native()))
    }
    #[component]
    pub fn badge(
        #[prop(default)] count: Option<u32>,
        #[prop(default = 99)] max: u32,
        #[prop(default)] dot: bool,
        #[prop(default)] color: Option<Color>,
        #[prop(default)] icon: Option<ComponentIcon>,
        cx: &mut ElementContext,
    ) -> impl IntoElement {
        gpui_component::badge::Badge::new()
            .max(max as usize)
            .when(dot, |v| v.dot())
            .when_some(count, |v, count| v.count(count as usize))
            .when_some(color, |v, c| v.color(c.native()))
            .when_some(icon, |v, path| v.icon(path.native()))
            .children(cx.children())
    }
    #[component(children = false)]
    pub fn clipboard(
        value: String,
        #[prop(default)] tooltip: Option<String>,
        on_copied: Event<String>,
        cx: &mut ElementContext,
    ) -> impl IntoElement {
        gpui_component::clipboard::Clipboard::new(cx.id())
            .value(value)
            .when_some(tooltip, |v, s| v.tooltip(s))
            .when(on_copied.is_subscribed(), |v| {
                v.on_copied(move |value, _, _| on_copied.emit(value.to_string()))
            })
    }
    #[component]
    pub fn collapsible(
        #[prop(default)] open: bool,
        trigger: NativeSlot,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::collapsible::Collapsible::new()
            .motion_id(cx.id())
            .open(open)
            .child(trigger)
            .content(cx.content())
    }
    #[component]
    pub fn group_box(
        #[prop(default)] variant: GroupBoxVariant,
        title: NativeSlot,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        use gpui_component::group_box::GroupBoxVariants;
        let variant = match variant {
            GroupBoxVariant::Normal => gpui_component::group_box::GroupBoxVariant::Normal,
            GroupBoxVariant::Fill => gpui_component::group_box::GroupBoxVariant::Fill,
            GroupBoxVariant::Outline => gpui_component::group_box::GroupBoxVariant::Outline,
        };
        gpui_component::group_box::GroupBox::new()
            .id(cx.id())
            .with_variant(variant)
            .when(!title.is_empty(), |v| v.title(title))
            .children(cx.children())
    }
    #[component(children = false)]
    pub fn label(
        text: String,
        #[prop(default)] secondary: Option<String>,
        #[prop(default)] masked: bool,
        #[prop(default)] highlight: Option<String>,
        #[prop(default)] highlight_prefix: bool,
    ) -> impl IntoElement + Styled {
        gpui_component::label::Label::new(text)
            .masked(masked)
            .when_some(secondary, |v, s| v.secondary(s))
            .when_some(highlight, |v, s| {
                v.highlights(if highlight_prefix {
                    gpui_component::label::HighlightsMatch::Prefix(s.into())
                } else {
                    gpui_component::label::HighlightsMatch::Full(s.into())
                })
            })
    }
    #[component]
    pub fn link(
        #[prop(default)] href: Option<String>,
        #[prop(default)] disabled: bool,
        on_press: Event<()>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::link::Link::new(cx.id())
            .disabled(disabled)
            .when_some(href, |v, s| v.href(s))
            .when(on_press.is_subscribed(), |v| {
                v.on_click(move |_, _, _| on_press.emit(()))
            })
            .children(cx.children())
    }
    #[component(children = false)]
    pub fn pagination(
        #[prop(default = 1)] current_page: u32,
        #[prop(default = 1)] total_pages: u32,
        #[prop(default = PageButtons(5))] visible_pages: PageButtons,
        #[prop(default)] disabled: bool,
        #[prop(default)] compact: bool,
        #[prop(default)] size: ControlSize,
        on_change: Event<u32>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::pagination::Pagination::new(cx.id())
            .current_page(current_page as usize)
            .total_pages(total_pages as usize)
            .visible_pages(visible_pages.0 as usize)
            .disabled(disabled)
            .with_size(size)
            .when(compact, |v| v.compact())
            .when(on_change.is_subscribed(), |v| {
                v.on_click(move |n, _, _| on_change.emit(*n as u32))
            })
    }
    #[component]
    pub fn progress_circle(
        #[prop(default)] value: Percentage,
        #[prop(default)] loading: bool,
        #[prop(default)] color: Option<Color>,
        #[prop(default)] size: ControlSize,
        #[prop(default)] accessibility_label: Option<String>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::progress::ProgressCircle::new(cx.id())
            .value(value.0)
            .loading(loading)
            .with_size(size)
            .when_some(color, |v, c| v.color(c.native()))
            .when_some(accessibility_label, |v, s| v.accessibility_label(s))
            .children(cx.children())
    }
    #[component]
    pub fn radio(
        #[prop(default)] checked: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] label: Option<String>,
        #[prop(default)] accessibility_label: Option<String>,
        #[prop(default)] tooltip: Option<String>,
        #[prop(default)] size: ControlSize,
        #[prop(default)] tab_index: i32,
        #[prop(default = true)] tab_stop: bool,
        on_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::radio::Radio::new(cx.id())
            .checked(checked)
            .disabled(disabled)
            .with_size(size)
            .tab_index(tab_index as isize)
            .tab_stop(tab_stop)
            .when_some(label, |v, s| v.label(s))
            .when_some(accessibility_label, |v, s| v.accessibility_label(s))
            .when_some(tooltip, |v, s| v.tooltip(s))
            .when(on_change.is_subscribed(), |v| {
                v.on_click(move |checked, _, _| on_change.emit(*checked))
            })
            .children(cx.children())
    }
    #[component(children = false)]
    pub fn rating(
        #[prop(default)] value: u32,
        #[prop(default = RatingMax(5))] max: RatingMax,
        #[prop(default)] disabled: bool,
        #[prop(default)] color: Option<Color>,
        #[prop(default)] size: ControlSize,
        on_change: Event<u32>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::rating::Rating::new(cx.id())
            .max(max.0 as usize)
            .value(value as usize)
            .disabled(disabled)
            .with_size(size)
            .when_some(color, |v, c| v.color(c.native()))
            .when(on_change.is_subscribed(), |v| {
                v.on_click(move |n, _, _| on_change.emit(*n as u32))
            })
    }
    #[component(children = false)]
    pub fn separator(
        #[prop(default)] orientation: Orientation,
        #[prop(default)] dashed: bool,
        #[prop(default)] label: Option<String>,
        #[prop(default)] color: Option<Color>,
    ) -> impl IntoElement + Styled {
        (match orientation {
            Orientation::Horizontal => gpui_component::separator::Separator::horizontal(),
            Orientation::Vertical => gpui_component::separator::Separator::vertical(),
        })
        .when(dashed, |v| v.dashed())
        .when_some(label, |v, s| v.label(s))
        .when_some(color, |v, c| v.color(c.native()))
    }
    #[component(children = false)]
    pub fn shimmer_text(
        text: String,
        #[prop(default = 1500)] duration_ms: u32,
        #[prop(default)] reverse: bool,
        #[prop(default)] once: bool,
        #[prop(default)] color: Option<Color>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::shimmer::ShimmerText::new(text)
            .id(cx.id())
            .duration(std::time::Duration::from_millis(duration_ms as u64))
            .reverse(reverse)
            .once(once)
            .when_some(color, |v, c| v.highlight_color(c.native()))
    }
    #[component(children = false)]
    pub fn skeleton(#[prop(default)] secondary: bool) -> impl IntoElement + Styled {
        gpui_component::skeleton::Skeleton::new().when(secondary, |v| v.secondary())
    }
    #[component(children = false)]
    pub fn spinner(
        #[prop(default)] color: Option<Color>,
        #[prop(default)] icon: Option<ComponentIcon>,
        #[prop(default)] size: ControlSize,
    ) -> impl IntoElement {
        gpui_component::spinner::Spinner::new()
            .with_size(size)
            .when_some(color, |v, c| v.color(c.native()))
            .when_some(icon, |v, s| v.icon(s.native()))
    }
    #[component(children = false)]
    pub fn status_bar(left: NativeSlot, right: NativeSlot) -> impl IntoElement + Styled {
        gpui_component::status_bar::StatusBar::new()
            .left(left)
            .right(right)
    }
    #[component]
    pub fn tag(
        #[prop(default)] variant: TagVariant,
        #[prop(default)] outline: bool,
        #[prop(default)] rounded: bool,
        #[prop(default)] size: ControlSize,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let variant = match variant {
            TagVariant::Primary => gpui_component::tag::TagVariant::Primary,
            TagVariant::Secondary => gpui_component::tag::TagVariant::Secondary,
            TagVariant::Danger => gpui_component::tag::TagVariant::Danger,
            TagVariant::Success => gpui_component::tag::TagVariant::Success,
            TagVariant::Warning => gpui_component::tag::TagVariant::Warning,
            TagVariant::Info => gpui_component::tag::TagVariant::Info,
        };
        gpui_component::tag::Tag::new()
            .with_variant(variant)
            .with_size(size)
            .when(outline, |v| v.outline())
            .when(rounded, |v| v.rounded_full())
            .children(cx.children())
    }
    #[component]
    pub fn toggle(
        #[prop(default)] checked: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] label: Option<String>,
        #[prop(default)] icon: Option<ComponentIcon>,
        #[prop(default)] outline: bool,
        #[prop(default)] tooltip: Option<String>,
        #[prop(default)] size: ControlSize,
        on_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        use gpui_component::button::ToggleVariants;
        gpui_component::button::Toggle::new(cx.id())
            .checked(checked)
            .disabled(disabled)
            .with_size(size)
            .when(outline, |v| v.outline())
            .when_some(label, |v, s| v.label(s))
            .when_some(tooltip, |v, s| v.tooltip(s))
            .when_some(icon, |v, s| v.icon(s.native()))
            .when(on_change.is_subscribed(), |v| {
                v.on_click(move |checked, _, _| on_change.emit(*checked))
            })
            .children(cx.children())
    }
    #[component(children = false)]
    pub fn icon(
        source: ComponentIcon,
        #[prop(default)] size: ControlSize,
        #[prop(default)] color: Option<Color>,
    ) -> impl IntoElement + Styled {
        source
            .native()
            .with_size(size)
            .when_some(color, |v, c| v.text_color(c.native()))
    }
}
pub(super) use exports::native_module;
