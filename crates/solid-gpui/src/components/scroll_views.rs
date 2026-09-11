//! Scroll handles belong to one mounted native view; only visible rows are built.
use super::primitives::{Color, Orientation};
use crate::native::{
    ComponentDefinition, Event, NativeChildren, NativeSlot, NativeView, ScrollDecoration,
    ScrollViewport, ViewCommand,
};
use gpui::{
    AppContext, Context, Entity, FocusHandle, IntoElement, ParentElement, Render, ScrollHandle,
    Styled, div, point, prelude::FluentBuilder, px, size,
};
use gpui_component::{
    FocusTrapElement,
    scroll::{ScrollableElement, ScrollableMask, Scrollbar, ScrollbarAxis, ScrollbarMode},
};
use std::{cell::RefCell, ops::Range, rc::Rc};

#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScrollAxis {
    Horizontal,
    #[default]
    Vertical,
    Both,
}
impl From<ScrollAxis> for ScrollbarAxis {
    fn from(v: ScrollAxis) -> Self {
        match v {
            ScrollAxis::Horizontal => Self::Horizontal,
            ScrollAxis::Vertical => Self::Vertical,
            ScrollAxis::Both => Self::Both,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScrollbarVisibility {
    #[default]
    Scrolling,
    Hover,
    Always,
}
impl From<ScrollbarVisibility> for ScrollbarMode {
    fn from(v: ScrollbarVisibility) -> Self {
        match v {
            ScrollbarVisibility::Scrolling => Self::Scrolling,
            ScrollbarVisibility::Hover => Self::Hover,
            ScrollbarVisibility::Always => Self::Always,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollPosition {
    pub x: f32,
    pub y: f32,
}
#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibleRange {
    pub start: usize,
    pub end: usize,
}
impl From<Range<usize>> for VisibleRange {
    fn from(v: Range<usize>) -> Self {
        Self {
            start: v.start,
            end: v.end,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ScrollableProps {
    pub axis: ScrollAxis,
    pub hide_scrollbar: bool,
    pub scrollbar_visibility: ScrollbarVisibility,
    pub contain_scroll: bool,
}
pub struct Scrollable {
    props: ScrollableProps,
    state: ScrollState,
}
struct ScrollState {
    children: NativeSlot,
    handle: ScrollHandle,
    viewport: ScrollViewport,
    decoration: Option<ScrollDecoration>,
    event: Event<ScrollPosition>,
    last: Rc<RefCell<Option<ScrollPosition>>>,
}
impl NativeView for Scrollable {
    type Props = ScrollableProps;
    type Event = ScrollPosition;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "scroll"
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) -> Self {
        Self {
            props,
            state: ScrollState::new(event, children),
        }
    }
    fn update(&mut self, props: Self::Props, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.props = props;
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        scroll_commands::<Self>()
    }
}
trait ScrollView: Sized + 'static {
    fn scroll_state(&mut self) -> &mut ScrollState;
}
impl ScrollView for Scrollable {
    fn scroll_state(&mut self) -> &mut ScrollState {
        &mut self.state
    }
}
fn scroll_commands<V: ScrollView>() -> Vec<ViewCommand<V>> {
    vec![
        ViewCommand::new("scrollTo", |this: &mut V, p: ScrollPosition, _, cx| {
            if !p.x.is_finite() || !p.y.is_finite() || p.x < 0. || p.y < 0. {
                return Err("scroll positions must be finite and nonnegative".into());
            }
            this.scroll_state()
                .active_viewport()
                .set_offset(point(px(-p.x), px(-p.y)));
            cx.notify();
            Ok(())
        }),
        ViewCommand::new("getScrollPosition", |this: &mut V, (): (), _, _| {
            let p = this.scroll_state().active_viewport().offset();
            Ok(ScrollPosition {
                x: -p.x.as_f32(),
                y: -p.y.as_f32(),
            })
        }),
    ]
}
impl Render for Scrollable {
    fn render(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl IntoElement {
        self.state
            .render_content(&self.props, self.state.children.clone(), None)
    }
}
impl ScrollState {
    fn new(event: Event<ScrollPosition>, children: NativeChildren) -> Self {
        let handle = ScrollHandle::default();
        Self {
            event,
            children: children.content(),
            viewport: ScrollViewport::new(handle.clone(), gpui::Axis::Vertical),
            handle,
            decoration: None,
            last: Rc::default(),
        }
    }
    fn active_viewport(&self) -> &ScrollViewport {
        self.decoration
            .as_ref()
            .map_or(&self.viewport, ScrollDecoration::viewport)
    }
    fn render_content(
        &self,
        props: &ScrollableProps,
        children: impl IntoElement,
        shadow: Option<ScrollFade>,
    ) -> gpui::AnyElement {
        let content = div()
            .size_full()
            .when(props.axis == ScrollAxis::Horizontal, |v| v.h_auto())
            .child(children);
        let area = match props.axis {
            ScrollAxis::Horizontal => content.overflow_x_scrollbar(),
            ScrollAxis::Vertical => content.overflow_y_scrollbar(),
            ScrollAxis::Both => content.overflow_scrollbar(),
        }
        .id("scroll-area")
        .track_scroll(&self.handle)
        .scrollbar_visible(false);
        self.render_viewport(props, area, shadow, props.axis == ScrollAxis::Horizontal)
    }
    fn render_viewport(
        &self,
        props: &ScrollableProps,
        area: impl IntoElement,
        shadow: Option<ScrollFade>,
        auto_height: bool,
    ) -> gpui::AnyElement {
        let handle = self.active_viewport().clone();
        let event = self.event.clone();
        let last = self.last.clone();
        div()
            .relative()
            .size_full()
            .when(auto_height, |v| v.h_auto())
            .min_size_0()
            .child(area)
            .when_some(shadow, |v, fade| {
                let handle = self.active_viewport().clone();
                v.child(
                    gpui::canvas(
                        |_, _, _| {},
                        move |_, (), window, _| {
                            let bounds = handle.viewport_bounds();
                            let horizontal = fade.axis == Orientation::Horizontal;
                            let offset = handle.offset();
                            let max = handle.max_offset();
                            let (leading, trailing) = scroll_fade_extents(
                                if horizontal { offset.x } else { offset.y }.as_f32(),
                                if horizontal { max.x } else { max.y }.as_f32(),
                                if horizontal {
                                    bounds.size.width
                                } else {
                                    bounds.size.height
                                }
                                .as_f32(),
                                fade.size,
                            );
                            for (extent, end) in [(leading, false), (trailing, true)] {
                                if extent <= 0. {
                                    continue;
                                }
                                let mut opaque = fade.color;
                                opaque.a *= extent / fade.size;
                                let mut transparent = opaque;
                                transparent.a = 0.;
                                let mut edge = bounds;
                                if horizontal {
                                    edge.size.width = px(extent);
                                    if end {
                                        edge.origin.x = bounds.right() - px(extent);
                                    }
                                } else {
                                    edge.size.height = px(extent);
                                    if end {
                                        edge.origin.y = bounds.bottom() - px(extent);
                                    }
                                }
                                window.paint_quad(gpui::fill(
                                    edge,
                                    gpui::linear_gradient(
                                        match (horizontal, end) {
                                            (true, true) => 90.,
                                            (true, false) => 270.,
                                            (false, true) => 180.,
                                            (false, false) => 0.,
                                        },
                                        gpui::linear_color_stop(transparent, 0.),
                                        gpui::linear_color_stop(opaque, 1.),
                                    ),
                                ));
                            }
                        },
                    )
                    .absolute()
                    .inset_0()
                    .size_full(),
                )
            })
            .when(!props.hide_scrollbar, |v| {
                v.child(
                    Scrollbar::new(self.active_viewport())
                        .id("scrollbar")
                        .axis(props.axis)
                        .mode(props.scrollbar_visibility.into()),
                )
            })
            .when(
                props.contain_scroll && props.axis != ScrollAxis::Horizontal,
                |v| v.child(ScrollableMask::new(gpui::Axis::Vertical, &self.handle).id("y-mask")),
            )
            .when(
                props.contain_scroll && props.axis != ScrollAxis::Vertical,
                |v| v.child(ScrollableMask::new(gpui::Axis::Horizontal, &self.handle).id("x-mask")),
            )
            .when(self.event.is_subscribed(), |v| {
                v.child(
                    gpui::canvas(
                        move |_, _, _| {
                            let p = handle.offset();
                            let next = ScrollPosition {
                                x: -p.x.as_f32(),
                                y: -p.y.as_f32(),
                            };
                            if last.borrow().as_ref() != Some(&next) {
                                *last.borrow_mut() = Some(next);
                                event.emit(next);
                            }
                        },
                        |_, (), _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
            })
            .into_any_element()
    }
}

// Read after scroll layout so dragging, commands and resize share the same geometry.
fn scroll_fade_extents(
    offset: f32,
    max_offset: f32,
    viewport_extent: f32,
    fade_size: f32,
) -> (f32, f32) {
    let extent = max_offset.max(0.);
    let position = (-offset).clamp(0., extent);
    let width = fade_size.min(viewport_extent.max(0.) / 2.);
    (position.min(width), (extent - position).min(width))
}
struct ScrollFade {
    axis: Orientation,
    color: gpui::Hsla,
    size: f32,
}

#[crate::native_type]
#[derive(Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct ScrollShadowProps {
    pub axis: Orientation,
    /// Match the surrounding surface. Defaults to the theme background.
    pub color: Option<Color>,
    /// Maximum fade extent in pixels; shrinks and becomes transparent near each boundary.
    pub fade_size: u16,
    pub scrollbar_visibility: ScrollbarVisibility,
}
impl Default for ScrollShadowProps {
    fn default() -> Self {
        Self {
            axis: Orientation::Vertical,
            color: None,
            fade_size: 24,
            scrollbar_visibility: ScrollbarVisibility::Always,
        }
    }
}
impl ScrollShadowProps {
    fn viewport(&self) -> ScrollableProps {
        ScrollableProps {
            axis: if self.axis == Orientation::Horizontal {
                ScrollAxis::Horizontal
            } else {
                ScrollAxis::Vertical
            },
            scrollbar_visibility: self.scrollbar_visibility,
            ..Default::default()
        }
    }
}
pub struct ScrollShadow {
    props: ScrollShadowProps,
    state: ScrollState,
}
impl ScrollView for ScrollShadow {
    fn scroll_state(&mut self) -> &mut ScrollState {
        self.reconcile_viewport();
        &mut self.state
    }
}
impl NativeView for ScrollShadow {
    type Props = ScrollShadowProps;
    type Event = ScrollPosition;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "scroll"
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) -> Self {
        Self {
            props,
            state: ScrollState::new(event, children),
        }
    }
    fn update(&mut self, props: Self::Props, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.state.decoration = None;
        self.props = props;
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        scroll_commands::<Self>()
    }
}
impl Render for ScrollShadow {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.reconcile_viewport();
        if self.state.decoration.is_some() {
            self.state.render_viewport(
                &self.props.viewport(),
                div()
                    .size_full()
                    .min_size_0()
                    .children(self.state.children.elements(cx)),
                Some(self.fade(cx)),
                false,
            )
        } else {
            self.render_content(self.state.children.clone(), cx)
        }
    }
}
impl ScrollShadow {
    fn reconcile_viewport(&mut self) {
        self.state.decoration = None;
        self.state.decoration = self
            .state
            .children
            .scroll_viewport()
            .filter(|viewport| viewport.axis() == self.props.axis.into())
            .map(ScrollViewport::decorate);
    }
    fn fade(&self, cx: &gpui::App) -> ScrollFade {
        use gpui_component::ActiveTheme;
        ScrollFade {
            axis: self.props.axis,
            color: self
                .props
                .color
                .as_ref()
                .map(Color::native)
                .unwrap_or_else(|| cx.theme().background),
            size: self.props.fade_size as f32,
        }
    }
    fn render_content(&self, children: impl IntoElement, cx: &gpui::App) -> gpui::AnyElement {
        self.state
            .render_content(&self.props.viewport(), children, Some(self.fade(cx)))
    }
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FocusTrapProps {
    pub auto_focus: bool,
}
pub struct FocusTrap {
    props: FocusTrapProps,
    children: NativeSlot,
    focus: FocusHandle,
}
impl NativeView for FocusTrap {
    type Props = FocusTrapProps;
    type Event = ();
    fn accepts_children() -> bool {
        true
    }
    fn mount(
        props: Self::Props,
        _: Event<()>,
        children: NativeChildren,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus = cx.focus_handle();
        if props.auto_focus {
            focus.focus(window, cx);
        }
        Self {
            props,
            children: children.content(),
            focus,
        }
    }
    fn update(&mut self, p: Self::Props, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if p.auto_focus && !self.props.auto_focus {
            self.focus.focus(window, cx);
        }
        self.props = p;
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![ViewCommand::new("focus", |this, (): (), window, cx| {
            this.focus.focus(window, cx);
            Ok(())
        })]
    }
}
impl Render for FocusTrap {
    fn render(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .focus_trap("focus-trap", &self.focus)
            .child(self.children.clone())
    }
}

#[crate::native_type]
#[derive(Clone, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct VirtualListProps {
    pub orientation: Orientation,
    pub item_size: f32,
    pub item_sizes: Option<Vec<f32>>,
    pub measure_index: usize,
    pub hide_scrollbar: bool,
}
impl Default for VirtualListProps {
    fn default() -> Self {
        Self {
            orientation: Orientation::Vertical,
            item_size: 32.,
            item_sizes: None,
            measure_index: 0,
            hide_scrollbar: false,
        }
    }
}
pub struct VirtualList {
    props: VirtualListProps,
    children: NativeSlot,
    ids: Rc<Vec<u32>>,
    sizes: Rc<Vec<gpui::Size<gpui::Pixels>>>,
    handle: gpui_component::VirtualListScrollHandle,
    viewport: ScrollViewport,
    event: Event<VisibleRange>,
    last: Rc<RefCell<Option<VisibleRange>>>,
}
impl VirtualList {
    fn sizes(props: &VirtualListProps, count: usize) -> Rc<Vec<gpui::Size<gpui::Pixels>>> {
        Rc::new(
            (0..count)
                .map(|i| {
                    let extent = props.item_sizes.as_ref().map_or(props.item_size, |s| s[i]);
                    size(px(extent), px(extent))
                })
                .collect(),
        )
    }
}
impl NativeView for VirtualList {
    type Props = VirtualListProps;
    type Event = VisibleRange;
    fn accepts_children() -> bool {
        true
    }
    fn scroll_viewport(&self) -> Option<ScrollViewport> {
        Some(
            self.viewport
                .clone()
                .with_axis(self.props.orientation.into()),
        )
    }
    fn event_name() -> &'static str {
        "visibleRangeChange"
    }
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        if !p.item_size.is_finite()
            || p.item_size <= 0.
            || p.item_size > 1_000_000.
            || p.item_sizes.as_ref().is_some_and(|s| {
                s.len() > 1_000_000
                    || s.iter()
                        .any(|v| !v.is_finite() || *v <= 0. || *v > 1_000_000.)
            })
        {
            return Err("item extents must be positive, finite, and at most 1000000".into());
        }
        Ok(())
    }
    fn validate_children(p: &Self::Props, c: &crate::ExtensionChildSummary) -> Result<(), String> {
        if c.count > 1_000_000 || p.item_sizes.as_ref().is_some_and(|s| s.len() != c.count) {
            return Err("itemSizes must contain one extent per child (at most 1000000)".into());
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) -> Self {
        let children = children.content();
        let ids = children.node_ids();
        let sizes = Self::sizes(&props, ids.len());
        let handle = gpui_component::VirtualListScrollHandle::new();
        let viewport = ScrollViewport::new(handle.base_handle().clone(), props.orientation.into());
        Self {
            props,
            event,
            children,
            ids,
            sizes,
            handle,
            viewport,
            last: Rc::default(),
        }
    }
    fn update(&mut self, p: Self::Props, _: &mut gpui::Window, cx: &mut Context<Self>) {
        let ids = self.children.node_ids();
        if p.orientation != self.props.orientation
            || p.item_sizes != self.props.item_sizes
            || p.item_size != self.props.item_size
            || ids.len() != self.ids.len()
        {
            self.sizes = Self::sizes(&p, ids.len());
        }
        self.ids = ids;
        self.props = p;
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("scrollToItem", |this, index: usize, _, cx| {
                if index >= this.ids.len() {
                    return Err("item index outside committed children".into());
                }
                this.handle.scroll_to_item(index, gpui::ScrollStrategy::Top);
                cx.notify();
                Ok(())
            }),
            ViewCommand::new("getVisibleRange", |this, (): (), _, _| {
                Ok(VisibleRange::from(this.handle.visible_range()))
            }),
        ]
    }
}
impl Render for VirtualList {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let children = self.children.clone();
        let last = self.last.clone();
        let event = self.event.clone();
        let list = gpui_base::virtual_list(
            cx.entity(),
            "native-virtual-list",
            self.props.orientation.into(),
            self.sizes.clone(),
            move |_, range, _, _| range.map(|i| children.item(i)).collect::<Vec<_>>(),
        )
        .track_scroll(&self.handle)
        .with_item_to_measure_index(self.props.measure_index)
        .on_visible_range(move |range, _, _| {
            let range = VisibleRange::from(range);
            if last.borrow().as_ref() != Some(&range) {
                *last.borrow_mut() = Some(range.clone());
                event.emit(range);
            }
        });
        div().relative().size_full().min_size_0().child(list).when(
            !self.props.hide_scrollbar && !self.viewport.is_decorated(),
            |v| {
                v.child(Scrollbar::new(&self.handle).id("list-scrollbar").axis(
                    match self.props.orientation {
                        Orientation::Vertical => ScrollbarAxis::Vertical,
                        Orientation::Horizontal => ScrollbarAxis::Horizontal,
                    },
                ))
            },
        )
    }
}

#[crate::native_type]
#[derive(Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct MessageScrollerProps {
    pub hide_scrollbar: bool,
    pub hide_jump_button: bool,
    pub jump_label: String,
    pub row_gap: f32,
    pub bottom_fade: Option<Color>,
}
impl Default for MessageScrollerProps {
    fn default() -> Self {
        Self {
            hide_scrollbar: false,
            hide_jump_button: false,
            jump_label: "Jump to latest".into(),
            row_gap: 16.,
            bottom_fade: None,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageScrollState {
    pub following_tail: bool,
    pub scrolled_up: bool,
    pub first_index: usize,
    pub offset_in_item: f32,
}
pub struct MessageScroller {
    props: MessageScrollerProps,
    children: NativeSlot,
    ids: Rc<Vec<u32>>,
    state: Entity<gpui_component::message_scroller::MessageScrollerState>,
    _subscription: gpui::Subscription,
}
impl MessageScroller {
    fn snapshot(s: &gpui_component::message_scroller::MessageScrollerState) -> MessageScrollState {
        let anchor = s.anchor();
        MessageScrollState {
            following_tail: s.is_following_tail(),
            scrolled_up: s.is_scrolled_up(),
            first_index: anchor.item_ix,
            offset_in_item: anchor.offset_in_item.as_f32(),
        }
    }
    fn reconcile(&mut self, ids: Rc<Vec<u32>>, cx: &mut Context<Self>) {
        if Rc::ptr_eq(&self.ids, &ids) {
            return;
        }
        let old = &self.ids;
        let prefix = old
            .iter()
            .zip(ids.iter())
            .take_while(|(a, b)| a == b)
            .count();
        let suffix = old[prefix..]
            .iter()
            .rev()
            .zip(ids[prefix..].iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        self.state.update(cx, |state, cx| {
            let following = state.is_following_tail();
            let anchor = state.anchor();
            let anchor_key = old.get(anchor.item_ix).copied();
            state.splice(prefix..old.len() - suffix, ids.len() - prefix - suffix, cx);
            if !following
                && let Some(index) = anchor_key.and_then(|key| ids.iter().position(|v| *v == key))
            {
                state.restore_anchor(
                    gpui::ListOffset {
                        item_ix: index,
                        offset_in_item: anchor.offset_in_item,
                    },
                    cx,
                );
            }
        });
        self.ids = ids;
    }
}
impl NativeView for MessageScroller {
    type Props = MessageScrollerProps;
    type Event = MessageScrollState;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "scrollStateChange"
    }
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        if !p.row_gap.is_finite() || p.row_gap < 0. || p.row_gap > 10000. {
            return Err("rowGap must be between 0 and 10000".into());
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        _: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let children = children.content();
        let ids = children.node_ids();
        let state =
            cx.new(|cx| gpui_component::message_scroller::MessageScrollerState::new(ids.len(), cx));
        let mut previous = None;
        let subscription = cx.observe(&state, move |_, state, cx| {
            let next = Self::snapshot(state.read(cx));
            if previous.as_ref() != Some(&next) {
                previous = Some(next.clone());
                event.emit(next);
            }
            cx.notify();
        });
        Self {
            props,
            children,
            ids,
            state,
            _subscription: subscription,
        }
    }
    fn update(&mut self, p: Self::Props, _: &mut gpui::Window, cx: &mut Context<Self>) {
        let ids = self.children.node_ids();
        self.reconcile(ids, cx);
        let changed = self.children.changed_indices();
        if !changed.is_empty() {
            self.state.update(cx, |state, cx| {
                for &index in changed.iter() {
                    state.remeasure_items(index..index + 1, cx);
                }
            });
        }
        // Explicit prop changes that affect every row invalidate the measured heights.
        if p.row_gap != self.props.row_gap {
            self.state.update(cx, |s, cx| s.remeasure(cx));
        }
        self.props = p;
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("scrollToItem", |this, index: usize, _, cx| {
                if this.state.update(cx, |s, cx| s.scroll_to_item(index, cx)) {
                    Ok(())
                } else {
                    Err("item index outside committed children".into())
                }
            }),
            ViewCommand::new("scrollToEnd", |this, (): (), _, cx| {
                this.state.update(cx, |s, cx| s.scroll_to_end(cx));
                Ok(())
            }),
            ViewCommand::new("getScrollState", |this, (): (), _, cx| {
                Ok(Self::snapshot(this.state.read(cx)))
            }),
            ViewCommand::new("remeasure", |this, range: Option<VisibleRange>, _, cx| {
                this.state.update(cx, |s, cx| {
                    if let Some(r) = range {
                        if !s.remeasure_items(r.start..r.end, cx) {
                            return Err("remeasure range outside committed children".into());
                        }
                    } else {
                        s.remeasure(cx);
                    }
                    Ok(())
                })
            }),
        ]
    }
}
impl Render for MessageScroller {
    fn render(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl IntoElement {
        let children = self.children.clone();
        let last = self.ids.len().saturating_sub(1);
        let gap = self.props.row_gap;
        gpui_component::message_scroller::MessageScroller::new(
            "messages",
            self.state.clone(),
            move |i, _, _| {
                div()
                    .pb(px(if i == last { 0. } else { gap }))
                    .child(children.item(i))
            },
        )
        .scrollbar(!self.props.hide_scrollbar)
        .jump_button(!self.props.hide_jump_button)
        .with_jump_button_label(self.props.jump_label.clone())
        .with_row_style(div().pb_0().style().clone())
        .when_some(self.props.bottom_fade.clone(), |v, c| {
            v.with_bottom_fade(c.native())
        })
    }
}
pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::view::<Scrollable>("Scrollable"),
        ComponentDefinition::view::<ScrollShadow>("ScrollShadow"),
        ComponentDefinition::view::<FocusTrap>("FocusTrap"),
        ComponentDefinition::view::<VirtualList>("VirtualList"),
        ComponentDefinition::view::<MessageScroller>("MessageScroller"),
    ]
    .into_iter()
    .map(|d| d.with_contract(include_str!("scroll_views.rs")))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    use gpui::{
        Axis, InteractiveElement, ScrollHandle, StatefulInteractiveElement, TestAppContext, Window,
        point, size,
    };
    use gpui_base::InteractiveElementExt as _;
    use gpui_base::VirtualListScrollHandle;
    #[test]
    fn scroll_fades_follow_available_content() {
        assert_eq!(scroll_fade_extents(0., 200., 100., 24.), (0., 24.));
        assert_eq!(scroll_fade_extents(-100., 200., 100., 24.), (24., 24.));
        assert_eq!(scroll_fade_extents(-200., 200., 100., 24.), (24., 0.));
        assert_eq!(scroll_fade_extents(-195., 200., 100., 24.), (24., 5.));
        assert_eq!(scroll_fade_extents(-200., 0., 300., 24.), (0., 0.));
        assert_eq!(scroll_fade_extents(-5., 10., 8., 24.), (4., 4.));
    }
    struct ScrollShadowHarness {
        view: Entity<ScrollShadow>,
        width: f32,
        content_width: f32,
    }
    impl Render for ScrollShadowHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = self.view.read(cx);
            let horizontal = view.props.axis == Orientation::Horizontal;
            let area = view.render_content(
                div()
                    .w(px(self.content_width))
                    .h(px(400.))
                    .debug_selector(|| "shadow-content".into()),
                cx,
            );
            div().size_full().child(
                div()
                    .w(px(self.width))
                    .flex_shrink_0()
                    .when(!horizontal, |v| v.h(px(180.)))
                    .child(area)
                    .debug_selector(|| "shadow-viewport".into()),
            )
        }
    }
    #[gpui::test]
    fn scroll_shadow_sizes_and_scrolls_in_both_directions(cx: &mut TestAppContext) {
        for axis in [Orientation::Horizontal, Orientation::Vertical] {
            let fixture = Fixture::<ScrollShadow>::new(
                ScrollShadowProps {
                    axis,
                    ..Default::default()
                },
                cx,
            );
            let state = fixture.view.clone();
            let horizontal = axis == Orientation::Horizontal;
            let (view, visual) = cx.add_window_view(move |_, _| ScrollShadowHarness {
                view: fixture.view.clone(),
                width: 300.,
                content_width: if horizontal { 500. } else { 300. },
            });
            visual.update(|window, cx| window.draw(cx).clear(cx));
            let viewport = visual.debug_bounds("shadow-viewport").unwrap();
            assert_eq!(
                viewport.size,
                size(px(300.), px(if horizontal { 400. } else { 180. }))
            );
            visual.update(|window, _| {
                let fades: Vec<_> = window
                    .painted_quads()
                    .into_iter()
                    .filter(|quad| quad.background.as_solid().is_none())
                    .collect();
                assert_eq!(
                    fades.len(),
                    1,
                    "only the trailing edge should fade at the start"
                );
                let fade = &fades[0];
                assert_eq!(
                    if horizontal {
                        fade.bounds.size.width
                    } else {
                        fade.bounds.size.height
                    }
                    .as_f32(),
                    24. * window.scale_factor()
                );
                let mut expected = viewport;
                if horizontal {
                    expected.origin.x = viewport.right() - px(24.);
                    expected.size.width = px(24.);
                } else {
                    expected.origin.y = viewport.bottom() - px(24.);
                    expected.size.height = px(24.);
                }
                assert_eq!(
                    fade.bounds,
                    expected.scale(window.scale_factor()),
                    "fades must overlay the viewport, not follow its content"
                );
            });
            let before = visual.debug_bounds("shadow-content").unwrap();
            visual.simulate_event(gpui::ScrollWheelEvent {
                position: viewport.origin + point(px(50.), px(50.)),
                delta: gpui::ScrollDelta::Pixels(if horizontal {
                    point(px(-40.), px(0.))
                } else {
                    point(px(0.), px(-40.))
                }),
                ..Default::default()
            });
            visual.update(|window, cx| window.draw(cx).clear(cx));
            let after = visual.debug_bounds("shadow-content").unwrap();
            visual.update(|window, _| {
                assert_eq!(
                    window
                        .painted_quads()
                        .iter()
                        .filter(|quad| quad.background.as_solid().is_none())
                        .count(),
                    2,
                    "both edges should fade while scrolled between boundaries"
                );
            });
            if horizontal {
                assert!(after.left() < before.left());
                assert_eq!(after.top(), before.top());
            } else {
                assert!(after.top() < before.top());
                assert_eq!(after.left(), before.left());
            }
            visual.update(|_, cx| {
                state.update(cx, |view, _| {
                    let max = view.state.handle.max_offset();
                    view.state.handle.set_offset(point(-max.x, -max.y));
                });
                view.update(cx, |_, cx| cx.notify());
            });
            visual.update(|window, cx| window.draw(cx).clear(cx));
            visual.update(|window, _| {
                let fades: Vec<_> = window
                    .painted_quads()
                    .into_iter()
                    .filter(|quad| quad.background.as_solid().is_none())
                    .collect();
                assert_eq!(fades.len(), 1, "the trailing edge must be clear at the end");
                assert_eq!(
                    fades[0].bounds.origin,
                    viewport.origin.scale(window.scale_factor())
                );
            });
            // A growing viewport must clear stale overflow and restore the leading content.
            if horizontal {
                visual.update(|_, cx| {
                    view.update(cx, |view, cx| {
                        view.width = 600.;
                        cx.notify();
                    })
                });
                visual.update(|window, cx| window.draw(cx).clear(cx));
                visual.update(|window, cx| {
                    assert!(
                        window
                            .painted_quads()
                            .iter()
                            .all(|quad| quad.background.as_solid().is_some()),
                        "content that fits must have no fades"
                    );
                    assert_eq!(state.read(cx).state.handle.max_offset().x, px(0.));
                    assert_eq!(state.read(cx).state.handle.offset().x, px(0.));
                });
            }
        }
    }
    #[derive(Clone, Copy, Debug)]
    enum ScrollKind {
        Plain,
        Virtual,
        Variable,
    }

    struct NestedScrollHarness {
        outer: ScrollHandle,
        inner: VirtualListScrollHandle,
        kind: ScrollKind,
        axis: Axis,
        count: usize,
        list: gpui::ListState,
        wheel_events: Rc<std::cell::Cell<usize>>,
    }

    impl Render for NestedScrollHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let wheel_events = self.wheel_events.clone();
            let inner = if matches!(self.kind, ScrollKind::Variable) {
                gpui::list(self.list.clone(), |_, _, _| {
                    div().h(px(20.)).into_any_element()
                })
                .w(px(100.))
                .h(px(100.))
                .into_any_element()
            } else if matches!(self.kind, ScrollKind::Virtual) {
                gpui_base::virtual_list(
                    cx.entity(),
                    "inner",
                    self.axis,
                    Rc::new(vec![
                        if self.axis == Axis::Vertical {
                            size(px(100.), px(20.))
                        } else {
                            size(px(20.), px(100.))
                        };
                        self.count
                    ]),
                    |_, range, _, _| {
                        range
                            .map(|_| div().w(px(20.)).h(px(20.)))
                            .collect::<Vec<_>>()
                    },
                )
                .track_scroll(&self.inner)
                .w(px(100.))
                .h(px(100.))
                .into_any_element()
            } else {
                div()
                    .id("inner")
                    .w(px(100.))
                    .h(px(100.))
                    .overflow_scroll()
                    .lock_scroll_axis()
                    .on_scroll_wheel(move |_, _, _| wheel_events.set(wheel_events.get() + 1))
                    .track_scroll(self.inner.as_ref())
                    .child(
                        div()
                            .w(if self.axis == Axis::Horizontal {
                                px(self.count as f32 * 20.)
                            } else {
                                px(100.)
                            })
                            .h(if self.axis == Axis::Vertical {
                                px(self.count as f32 * 20.)
                            } else {
                                px(100.)
                            })
                            .flex_shrink_0(),
                    )
                    .into_any_element()
            };
            div()
                .id("outer")
                .w(px(200.))
                .h(px(200.))
                .overflow_y_scroll()
                .track_scroll(&self.outer)
                .child(div().h(px(1000.)).flex_shrink_0().child(inner))
        }
    }

    #[gpui::test]
    fn nested_scroll_moves_only_the_consuming_viewport(cx: &mut TestAppContext) {
        for kind in [ScrollKind::Plain, ScrollKind::Virtual, ScrollKind::Variable] {
            let wheel_events = Rc::new(std::cell::Cell::new(0));
            let outer = ScrollHandle::new();
            let inner = VirtualListScrollHandle::new();
            let list = gpui::ListState::new(50, gpui::ListAlignment::Top, px(0.)).measure_all();
            let (_, visual) = cx.add_window_view({
                let outer = outer.clone();
                let inner = inner.clone();
                let list = list.clone();
                let wheel_events = wheel_events.clone();
                move |_, _| NestedScrollHarness {
                    outer,
                    inner,
                    kind,
                    axis: Axis::Vertical,
                    count: 50,
                    list,
                    wheel_events,
                }
            });
            visual.update(|window, cx| window.draw(cx).clear(cx));
            visual.simulate_event(gpui::ScrollWheelEvent {
                position: point(px(50.), px(50.)),
                delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-40.))),
                ..Default::default()
            });
            if matches!(kind, ScrollKind::Plain) {
                assert_eq!(wheel_events.get(), 1, "local wheel listener still fires");
            }
            if matches!(kind, ScrollKind::Variable) {
                let top = list.logical_scroll_top();
                assert_eq!(px(top.item_ix as f32 * 20.) + top.offset_in_item, px(40.));
            } else {
                assert_eq!(inner.offset().y, px(-40.), "inner moves: {kind:?}");
            }
            assert_eq!(outer.offset().y, px(0.), "page stays still: {kind:?}");
            // Reaching an edge consumes the current event, without overscrolling either viewport.
            visual.simulate_event(gpui::ScrollWheelEvent {
                position: point(px(50.), px(50.)),
                delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-2000.))),
                ..Default::default()
            });
            assert_eq!(outer.offset().y, px(0.), "reaching edge: {kind:?}");
            visual.update(|window, cx| window.draw(cx).clear(cx));
            visual.simulate_event(gpui::ScrollWheelEvent {
                position: point(px(50.), px(50.)),
                delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-40.))),
                ..Default::default()
            });
            assert_eq!(outer.offset().y, px(-40.), "edge hands off: {kind:?}");
        }
    }

    #[gpui::test]
    fn horizontal_and_non_overflowing_children_route_wheels_to_the_right_viewport(
        cx: &mut TestAppContext,
    ) {
        for kind in [ScrollKind::Plain, ScrollKind::Virtual, ScrollKind::Variable] {
            for axis in [Axis::Vertical, Axis::Horizontal] {
                if matches!(kind, ScrollKind::Variable) && axis == Axis::Horizontal {
                    continue;
                }
                let count = if axis == Axis::Vertical { 2 } else { 50 };
                let outer = ScrollHandle::new();
                let inner = VirtualListScrollHandle::new();
                let (_, visual) = cx.add_window_view({
                    let outer = outer.clone();
                    let inner = inner.clone();
                    move |_, _| NestedScrollHarness {
                        outer,
                        inner,
                        kind,
                        axis,
                        count,
                        wheel_events: Rc::new(std::cell::Cell::new(0)),
                        list: gpui::ListState::new(count, gpui::ListAlignment::Top, px(0.))
                            .measure_all(),
                    }
                });
                visual.update(|window, cx| window.draw(cx).clear(cx));
                if axis == Axis::Horizontal {
                    visual.simulate_event(gpui::ScrollWheelEvent {
                        position: point(px(50.), px(50.)),
                        delta: gpui::ScrollDelta::Pixels(point(px(-40.), px(0.))),
                        ..Default::default()
                    });
                    assert_eq!(inner.offset().x, px(-40.), "horizontal child: {kind:?}");
                    assert_eq!(outer.offset().y, px(0.));
                }
                visual.simulate_event(gpui::ScrollWheelEvent {
                    position: point(px(50.), px(50.)),
                    delta: gpui::ScrollDelta::Lines(point(0., -2.)),
                    ..Default::default()
                });
                assert!(
                    outer.offset().y < px(0.),
                    "vertical wheel reaches page: {kind:?}, {axis:?}"
                );
                assert_eq!(inner.offset().y, px(0.));
            }
        }
    }

    #[gpui::test]
    fn message_append_prepend_and_reorder_keep_native_follow_mode_and_reading_anchor(
        cx: &mut gpui::TestAppContext,
    ) {
        let fixture = Fixture::<MessageScroller>::new(MessageScrollerProps::default(), cx);
        fixture.update(cx, |view, _, cx| {
            let state_id = view.state.entity_id();
            view.reconcile(Rc::new(vec![1, 2, 3]), cx);
            assert!(view.state.read(cx).is_following_tail());
            view.reconcile(Rc::new(vec![1, 2, 3, 4]), cx);
            assert!(
                view.state.read(cx).is_following_tail(),
                "appending at the tail keeps following"
            );
            view.state.update(cx, |s, cx| {
                assert!(s.scroll_to_item(1, cx));
                s.restore_anchor(
                    gpui::ListOffset {
                        item_ix: 1,
                        offset_in_item: px(7.),
                    },
                    cx,
                );
            });
            view.reconcile(Rc::new(vec![0, 1, 2, 3, 4]), cx);
            let s = view.state.read(cx);
            assert!(!s.is_following_tail());
            assert_eq!(s.anchor().item_ix, 2);
            assert_eq!(s.anchor().offset_in_item, px(7.));
            view.reconcile(Rc::new(vec![3, 4, 0, 1, 2]), cx);
            let s = view.state.read(cx);
            assert_eq!(s.anchor().item_ix, 4);
            assert_eq!(s.anchor().offset_in_item, px(7.));
            assert_eq!(s.item_count(), 5);
            assert_eq!(view.state.entity_id(), state_id);
            view.state.update(cx, |s, cx| s.scroll_to_end(cx));
            view.reconcile(Rc::new(vec![3, 4, 0, 1, 2, 5]), cx);
            assert!(view.state.read(cx).is_following_tail());
        });
    }
}
