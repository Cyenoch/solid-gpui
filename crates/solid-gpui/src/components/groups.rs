//! Compound controls receive concrete native children; no render-time JSX callback is needed.
use super::icon_source::ComponentIcon;
use super::primitives::Orientation;
use super::validation::{GridColumns, GridLine, GridSpan, LogicalPixels};
use super::{ButtonVariant, ControlSize};

#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TabVariant {
    #[default]
    Tab,
    Pill,
    Outline,
    Segmented,
    Underline,
}
impl From<TabVariant> for gpui_component::tab::TabVariant {
    fn from(v: TabVariant) -> Self {
        match v {
            TabVariant::Tab => Self::Tab,
            TabVariant::Pill => Self::Pill,
            TabVariant::Outline => Self::Outline,
            TabVariant::Segmented => Self::Segmented,
            TabVariant::Underline => Self::Underline,
        }
    }
}

#[crate::native_module(name = "gpui-component")]
mod exports {
    use super::*;
    use crate::native::{ElementContext, Event, NativeItems, NativeSlot};
    use gpui::{IntoElement, ParentElement, Styled, prelude::FluentBuilder, px};
    use gpui_component::{Disableable, Selectable, Sizable};
    use gpui_component::{accordion, avatar, breadcrumb, button, form, radio, stepper, tab};

    #[component]
    pub fn accordion(
        items: NativeItems<accordion::AccordionItem>,
        #[prop(default)] multiple: bool,
        #[prop(default)] bordered: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] size: ControlSize,
        on_change: Event<Vec<u32>>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        items.into_iter().fold(
            accordion::Accordion::new(cx.id())
                .multiple(multiple)
                .bordered(bordered)
                .disabled(disabled)
                .with_size(size)
                .when(on_change.is_subscribed(), |v| {
                    v.on_toggle_click(move |indices, _, _| {
                        on_change.emit(indices.iter().map(|i| *i as u32).collect())
                    })
                }),
            |v, item| v.child(item),
        )
    }
    #[component]
    pub fn accordion_item(
        #[prop(default)] open: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] icon: Option<ComponentIcon>,
        title: NativeSlot,
        on_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        accordion::AccordionItem::new()
            .open(open)
            .disabled(disabled)
            .when(on_change.is_subscribed(), |v| {
                v.on_toggle_click(move |open, _, _| on_change.emit(*open))
            })
            .when(!title.is_empty(), |v| v.title(title))
            .when_some(icon, |v, p| v.icon(p.native()))
            .children(cx.children())
    }
    #[component]
    pub fn avatar_group(
        items: NativeItems<avatar::Avatar>,
        #[prop(default)] limit: Option<u32>,
        #[prop(default)] ellipsis: bool,
        #[prop(default)] size: ControlSize,
    ) -> impl IntoElement + Styled {
        avatar::AvatarGroup::new()
            .with_size(size)
            .when_some(limit, |v, n| v.limit(n as usize))
            .when(ellipsis, |v| v.ellipsis())
            .children(items)
    }
    #[component]
    pub fn breadcrumb(items: NativeItems<breadcrumb::BreadcrumbItem>) -> impl IntoElement + Styled {
        breadcrumb::Breadcrumb::new().children(items)
    }
    #[component(children = false)]
    pub fn breadcrumb_item(
        label: String,
        #[prop(default)] disabled: bool,
        on_press: Event<()>,
    ) -> impl IntoElement + Styled {
        breadcrumb::BreadcrumbItem::new(label)
            .disabled(disabled)
            .when(on_press.is_subscribed(), |v| {
                v.on_click(move |_, _, _| on_press.emit(()))
            })
    }
    #[component]
    pub fn button_group(
        items: NativeItems<button::Button>,
        #[prop(default)] multiple: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] compact: bool,
        #[prop(default)] outline: bool,
        #[prop(default)] orientation: Orientation,
        #[prop(default)] variant: ButtonVariant,
        #[prop(default)] size: ControlSize,
        on_change: Event<Vec<u32>>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        use gpui_component::button::ButtonVariants;
        button::ButtonGroup::new(cx.id())
            .multiple(multiple)
            .disabled(disabled)
            .layout(orientation.into())
            .with_size(size)
            .with_variant(variant.into())
            .when(compact, |v| v.compact())
            .when(outline, |v| v.outline())
            .children(items)
            .when(on_change.is_subscribed(), |v| {
                v.on_click(move |indices, _, _| {
                    on_change.emit(indices.iter().map(|i| *i as u32).collect())
                })
            })
    }
    #[component]
    pub fn toggle_group(
        items: NativeItems<button::Toggle>,
        #[prop(default)] segmented: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] size: ControlSize,
        on_change: Event<Vec<bool>>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        button::ToggleGroup::new(cx.id())
            .disabled(disabled)
            .with_size(size)
            .when(segmented, |v| v.segmented())
            .children(items)
            .when(on_change.is_subscribed(), |v| {
                v.on_click(move |values, _, _| on_change.emit(values.clone()))
            })
    }
    #[component]
    pub fn radio_group(
        items: NativeItems<radio::Radio>,
        #[prop(default)] selected_index: Option<u32>,
        #[prop(default)] orientation: Orientation,
        #[prop(default)] disabled: bool,
        #[prop(default)] size: ControlSize,
        on_change: Event<u32>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        radio::RadioGroup::horizontal(cx.id())
            .layout(orientation.into())
            .selected_index(selected_index.map(|n| n as usize))
            .disabled(disabled)
            .children(items.into_iter().map(|item| item.with_size(size)))
            .when(on_change.is_subscribed(), |v| {
                v.on_click(move |i, _, _| on_change.emit(*i as u32))
            })
    }
    #[component]
    pub fn tab(
        #[prop(default)] label: Option<String>,
        #[prop(default)] aria_label: Option<String>,
        #[prop(default)] icon: Option<ComponentIcon>,
        #[prop(default)] variant: TabVariant,
        #[prop(default)] disabled: bool,
        #[prop(default)] selected: bool,
        #[prop(default)] size: ControlSize,
        prefix: NativeSlot,
        suffix: NativeSlot,
        on_press: Event<()>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        tab::Tab::new()
            .with_variant(variant.into())
            .disabled(disabled)
            .selected(selected)
            .with_size(size)
            .when_some(label, |v, s| v.label(s))
            .when_some(aria_label, |v, s| v.aria_label(s))
            .when_some(icon, |v, p| v.icon(p.native()))
            .when(!prefix.is_empty(), |v| v.prefix(prefix))
            .when(!suffix.is_empty(), |v| v.suffix(suffix))
            .when(on_press.is_subscribed(), |v| {
                v.on_click(move |_, _, _| on_press.emit(()))
            })
            .children(cx.children())
    }
    #[component]
    pub fn tab_bar(
        items: NativeItems<tab::Tab>,
        #[prop(default)] selected_index: u32,
        #[prop(default)] variant: TabVariant,
        #[prop(default)] menu: bool,
        #[prop(default)] size: ControlSize,
        prefix: NativeSlot,
        suffix: NativeSlot,
        on_change: Event<u32>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        tab::TabBar::new(cx.id())
            .with_variant(variant.into())
            .selected_index(selected_index as usize)
            .menu(menu)
            .with_size(size)
            .when(!prefix.is_empty(), |v| v.prefix(prefix))
            .when(!suffix.is_empty(), |v| v.suffix(suffix))
            .children(items)
            .when(on_change.is_subscribed(), |v| {
                v.on_click(move |i, _, _| on_change.emit(*i as u32))
            })
    }
    #[component]
    pub fn stepper(
        items: NativeItems<stepper::StepperItem>,
        #[prop(default)] selected_index: u32,
        #[prop(default)] orientation: Orientation,
        #[prop(default)] disabled: bool,
        #[prop(default)] text_center: bool,
        #[prop(default)] size: ControlSize,
        on_change: Event<u32>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        stepper::Stepper::new(cx.id())
            .layout(orientation.into())
            .selected_index(selected_index as usize)
            .disabled(disabled)
            .text_center(text_center)
            .with_size(size)
            .items(items)
            .when(on_change.is_subscribed(), |v| {
                v.on_click(move |i, _, _| on_change.emit(*i as u32))
            })
    }
    #[component]
    pub fn stepper_item(
        #[prop(default)] disabled: bool,
        #[prop(default)] icon: Option<ComponentIcon>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        stepper::StepperItem::new()
            .disabled(disabled)
            .when_some(icon, |v, p| v.icon(p.native()))
            .children(cx.children())
    }
    #[component]
    pub fn form(
        items: NativeItems<form::Field>,
        #[prop(default)] orientation: Orientation,
        #[prop(default = GridColumns(1))] columns: GridColumns,
        #[prop(default = LogicalPixels(100.0))] label_width: LogicalPixels,
        #[prop(default)] size: ControlSize,
    ) -> impl IntoElement + Styled {
        form::Form::vertical()
            .layout(orientation.into())
            .columns(columns.0 as usize)
            .label_width(px(label_width.0))
            .with_size(size)
            .children(items)
    }
    #[component]
    pub fn field(
        #[prop(default)] required: bool,
        #[prop(default = true)] visible: bool,
        #[prop(default = GridSpan(1))] col_span: GridSpan,
        #[prop(default)] col_start: Option<GridLine>,
        #[prop(default)] col_end: Option<GridLine>,
        #[prop(default)] label_indent: bool,
        label: NativeSlot,
        description: NativeSlot,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        form::Field::new()
            .required(required)
            .visible(visible)
            .col_span(col_span.0)
            .label_indent(label_indent)
            .when_some(col_start, |v, n| v.col_start(n.0))
            .when_some(col_end, |v, n| v.col_end(n.0))
            .when(!label.is_empty(), |v| v.label_fn(move |_, _| label.clone()))
            .when(!description.is_empty(), |v| {
                v.description_fn(move |_, _| description.clone())
            })
            .children(cx.children())
    }
}
pub(super) use exports::native_module;

#[cfg(test)]
mod tests {
    use gpui::{ParentElement, RenderOnce, Styled};
    use std::{cell::Cell, rc::Rc};

    #[gpui::test]
    fn explicit_field_labels_render_without_placeholder_indentation(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let visual = cx.add_empty_window();
        let rendered = Rc::new(Cell::new(false));
        visual.update(|window, cx| {
            let marker = rendered.clone();
            let field = gpui_component::form::Field::new()
                .label_indent(false)
                .label_fn(move |_, _| {
                    marker.set(true);
                    gpui::div().child("Name")
                })
                .child(gpui::div().h(gpui::px(32.)));
            let _ = field.render(window, cx);
        });
        assert!(
            rendered.get(),
            "an explicit label is independent of empty-label indentation"
        );
    }
    struct FieldVisibility {
        visible: bool,
        bounds: Rc<Cell<gpui::Bounds<gpui::Pixels>>>,
    }
    impl gpui::Render for FieldVisibility {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            use gpui_component::ElementExt;
            let bounds = self.bounds.clone();
            gpui::div().child(
                gpui::div()
                    .w(gpui::px(200.))
                    .child(
                        gpui_component::form::Field::new()
                            .visible(self.visible)
                            .child(gpui::div().h(gpui::px(32.))),
                    )
                    .on_prepaint(move |b, _, _| bounds.set(b)),
            )
        }
    }
    #[gpui::test]
    fn conditional_fields_leave_no_layout_space_when_hidden(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        for visible in [false, true] {
            let bounds = Rc::new(Cell::new(gpui::Bounds::default()));
            let (_, visual) = cx.add_window_view({
                let bounds = bounds.clone();
                move |_, _| FieldVisibility { visible, bounds }
            });
            visual.update(|window, cx| window.draw(cx).clear(cx));
            if visible {
                assert!(bounds.get().size.height >= gpui::px(32.));
            } else {
                assert_eq!(bounds.get().size.height, gpui::px(0.));
            }
        }
    }
}
