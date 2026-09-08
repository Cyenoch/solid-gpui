//! Scroll handles belong to one mounted native view; only visible rows are built.
use super::primitives::{Color, Orientation};
use crate::native::{
    ComponentDefinition, Event, NativeChildren, NativeSlot, NativeView, ViewCommand,
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
    children: NativeSlot,
    handle: ScrollHandle,
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
            event,
            children: children.content(),
            handle: ScrollHandle::default(),
            last: Rc::default(),
        }
    }
    fn update(&mut self, props: Self::Props, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.props = props;
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("scrollTo", |this, p: ScrollPosition, _, cx| {
                if !p.x.is_finite() || !p.y.is_finite() || p.x < 0. || p.y < 0. {
                    return Err("scroll positions must be finite and nonnegative".into());
                }
                this.handle.set_offset(point(px(-p.x), px(-p.y)));
                cx.notify();
                Ok(())
            }),
            ViewCommand::new("getScrollPosition", |this, (): (), _, _| {
                let p = this.handle.offset();
                Ok(ScrollPosition {
                    x: -p.x.as_f32(),
                    y: -p.y.as_f32(),
                })
            }),
        ]
    }
}
impl Render for Scrollable {
    fn render(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl IntoElement {
        let content = div().size_full().child(self.children.clone());
        let area = match self.props.axis {
            ScrollAxis::Horizontal => content.overflow_x_scrollbar(),
            ScrollAxis::Vertical => content.overflow_y_scrollbar(),
            ScrollAxis::Both => content.overflow_scrollbar(),
        }
        .id("scroll-area")
        .track_scroll(&self.handle)
        .scrollbar_visible(false);
        let handle = self.handle.clone();
        let event = self.event.clone();
        let last = self.last.clone();
        div()
            .relative()
            .size_full()
            .min_size_0()
            .child(area)
            .when(!self.props.hide_scrollbar, |v| {
                v.child(
                    Scrollbar::new(&self.handle)
                        .id("scrollbar")
                        .axis(self.props.axis)
                        .mode(self.props.scrollbar_visibility.into()),
                )
            })
            .when(
                self.props.contain_scroll && self.props.axis != ScrollAxis::Horizontal,
                |v| v.child(ScrollableMask::new(gpui::Axis::Vertical, &self.handle).id("y-mask")),
            )
            .when(
                self.props.contain_scroll && self.props.axis != ScrollAxis::Vertical,
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
        Self {
            props,
            event,
            children,
            ids,
            sizes,
            handle: gpui_component::VirtualListScrollHandle::new(),
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
            !self.props.hide_scrollbar,
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
