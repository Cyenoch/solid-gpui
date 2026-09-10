use std::{ops::Range, rc::Rc};

use gpui::{
    App, AppContext, Context, ElementId, Entity, Focusable, IntoElement, ParentElement, Render,
    RenderOnce, SharedString, StyleRefinement, Styled, Subscription, WeakEntity, Window,
    prelude::FluentBuilder, px,
};
use rust_i18n::t;

use gpui_base::{Pagination as BasePagination, PaginationItem as PageItem, PaginationState};

use crate::{
    Disableable, Icon, Sizable, Size, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    icon::IconName,
    input::{Input, InputEvent, InputState},
    popover::{Popover, PopoverState},
    v_flex,
};

/// Pagination with page navigation, next and previous links.
#[derive(IntoElement)]
pub struct Pagination {
    id: ElementId,
    style: StyleRefinement,
    size: Size,
    current_page: usize,
    total_pages: usize,
    disabled: bool,
    compact: bool,
    visible_pages: usize,
    on_click: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl Pagination {
    /// Create a new Pagination component with the given ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            size: Size::default(),
            current_page: 1,
            total_pages: 1,
            visible_pages: 5,
            disabled: false,
            compact: false,
            on_click: None,
        }
    }

    /// Set the current page number (1-based).
    ///
    /// The value will be clamped between 1 and total_pages when total_pages is set.
    pub fn current_page(mut self, page: usize) -> Self {
        self.current_page = page.max(1);
        self
    }

    /// Set the total number of pages.
    pub fn total_pages(mut self, pages: usize) -> Self {
        self.total_pages = pages.max(1);
        if self.current_page > self.total_pages {
            self.current_page = self.total_pages;
        }
        self
    }

    /// Set the handler for page change (when clicking on page numbers, prev, or next).
    ///
    /// This handler receives the new page number to navigate to.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// Pagination::new("my-pagination")
    ///     .current_page(current_page)
    ///     .total_pages(total_pages)
    ///     .on_click(|page, _, cx| {
    ///         // Handle page change
    ///     })
    /// ```
    pub fn on_click(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Set to display as compact style.
    ///
    /// If true, only the prev, next buttons with only icon.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Set viewable maximum number of page buttons, default
    pub fn visible_pages(mut self, max: usize) -> Self {
        self.visible_pages = max;
        self
    }

    fn render_nav_button(&self, state: &PaginationState, is_prev: bool) -> Button {
        let (id, label, icon) = if is_prev {
            ("prev", t!("Pagination.previous"), IconName::ChevronLeft)
        } else {
            ("next", t!("Pagination.next"), IconName::ChevronRight)
        };

        let target_page = if is_prev {
            state.previous_page()
        } else {
            state.next_page()
        };

        Button::new(id)
            .ghost()
            .compact()
            .with_size(self.size)
            .disabled(target_page.is_none())
            .tooltip(label.clone())
            .when(self.compact, |this| this.icon(icon.clone()))
            .when(!self.compact, |this| {
                this.child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .flex_nowrap()
                        .when(is_prev, |this| this.flex_row_reverse())
                        .child(SharedString::from(label))
                        .child(Icon::new(icon)),
                )
            })
            .when_some(
                target_page.filter(|_| state.has_on_change()),
                |this, target_page| {
                    let state = state.clone();
                    this.on_click(move |_, window, cx| {
                        state.request_page(target_page, window, cx);
                    })
                },
            )
    }
}

impl Disableable for Pagination {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Sizable for Pagination {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Styled for Pagination {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Pagination {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut state = PaginationState::new(self.current_page, self.total_pages)
            .visible_pages(self.visible_pages)
            .disabled(self.disabled);
        if let Some(on_click) = self.on_click.clone() {
            state = state.on_change(move |page, window, cx| on_click(&page, window, cx));
        }
        let page_numbers = (!self.compact).then(|| state.items()).unwrap_or_default();

        let current_page = state.current_page();
        let is_disabled = self.disabled;
        let item_state = state.clone();

        BasePagination::new(self.id.clone(), state.clone())
            .h_flex()
            .px_2()
            .py_2()
            .gap_1()
            .items_center()
            .refine_style(&self.style)
            .child(self.render_nav_button(&state, true))
            .children({
                page_numbers.into_iter().map(|item| match item {
                    PageItem::Page(page) => {
                        let is_selected = page == current_page;

                        Button::new(page)
                            .with_size(self.size)
                            .map(|this| {
                                if is_selected {
                                    this.outline()
                                } else {
                                    this.ghost()
                                }
                            })
                            .label(page.to_string())
                            .compact()
                            .disabled(is_disabled)
                            .when(!is_selected && item_state.has_on_change(), |this| {
                                let state = item_state.clone();
                                this.on_click(move |_, window, cx| {
                                    state.request_page(page, window, cx);
                                })
                            })
                            .into_any_element()
                    }
                    PageItem::Ellipsis(range) => {
                        let id =
                            SharedString::from(format!("ellipsis-{}-{}", range.start, range.end));
                        let state = item_state.clone();
                        let size = self.size;
                        Popover::new(id)
                            .trigger(
                                Button::new("trigger")
                                    .ghost()
                                    .with_size(size)
                                    .compact()
                                    .disabled(self.disabled)
                                    .icon(IconName::Ellipsis),
                            )
                            .content(move |popover, window, cx| {
                                let owner = cx.entity().downgrade();
                                let jump = window.use_keyed_state("jump", cx, |window, cx| {
                                    PageJump::new(
                                        state.clone(),
                                        range.clone(),
                                        owner.clone(),
                                        size,
                                        window,
                                        cx,
                                    )
                                });
                                jump.update(cx, |jump, _| {
                                    jump.state = state.clone();
                                    jump.range = range.clone();
                                    jump.popup = owner;
                                    jump.size = size;
                                });
                                popover.track_focus(Some(
                                    jump.read(cx).input.read(cx).focus_handle(cx),
                                ));
                                jump
                            })
                            .into_any_element()
                    }
                })
            })
            .child(self.render_nav_button(&state, false))
    }
}

/// The omitted page interval is represented by one input, independent of total pages.
struct PageJump {
    state: PaginationState,
    range: Range<usize>,
    popup: WeakEntity<PopoverState>,
    input: Entity<InputState>,
    size: Size,
    _subscription: Subscription,
}
impl PageJump {
    fn new(
        state: PaginationState,
        range: Range<usize>,
        popup: WeakEntity<PopoverState>,
        size: Size,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!("Pagination.go_to_page")));
        input.read(cx).focus_handle(cx).focus(window, cx);
        let subscription =
            cx.subscribe_in(&input, window, |this, _, event, window, cx| match event {
                InputEvent::PressEnter { .. } => this.submit(window, cx),
                InputEvent::Change => cx.notify(),
                _ => {}
            });
        Self {
            state,
            range,
            popup,
            input,
            size,
            _subscription: subscription,
        }
    }
    fn page(&self, cx: &App) -> Option<usize> {
        self.input
            .read(cx)
            .value()
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|p| self.range.contains(p))
    }
    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(page) = self.page(cx) else {
            return;
        };
        if self.state.is_disabled() {
            return;
        }
        let _ = self.popup.update(cx, |popup, cx| popup.dismiss(window, cx));
        self.state.request_page(page, window, cx);
    }
}
impl Render for PageJump {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let valid =
            self.page(cx).is_some() && self.state.has_on_change() && !self.state.is_disabled();
        v_flex()
            .p_2()
            .gap_2()
            .w(px(200.))
            .child(format!("{}–{}", self.range.start, self.range.end - 1))
            .child(
                h_flex()
                    .gap_2()
                    .child(Input::new(&self.input).with_size(self.size).flex_1())
                    .child(
                        Button::new("go")
                            .icon(IconName::ArrowRight)
                            .with_size(self.size)
                            .disabled(!valid)
                            .tooltip(t!("Pagination.go_to_page"))
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use std::cell::RefCell;
    #[gpui::test]
    fn large_page_range_uses_one_input_and_validates_keyboard_jump(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let chosen = Rc::new(RefCell::new(vec![]));
        let total = u32::MAX as usize;
        let state = PaginationState::new(1, total).on_change({
            let chosen = chosen.clone();
            move |page, _, _| chosen.borrow_mut().push(page)
        });
        assert!(state.items().len() <= 7);
        let popup_owner = Rc::new(RefCell::new(None));
        let (jump, cx) = cx.add_window_view(|window, cx| {
            let popup = cx.new(|cx| PopoverState::new(true, cx));
            let weak = popup.downgrade();
            *popup_owner.borrow_mut() = Some(popup);
            PageJump::new(state, 2..total, weak, Size::Medium, window, cx)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let input = jump.read_with(cx, |jump, _| jump.input.clone());
        cx.update(|window, cx| input.update(cx, |input, cx| input.set_value("0", window, cx)));
        cx.simulate_keystrokes("enter");
        assert!(chosen.borrow().is_empty());
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                input.set_value((total - 1).to_string(), window, cx)
            })
        });
        cx.simulate_keystrokes("enter");
        assert_eq!(*chosen.borrow(), vec![total - 1]);
        assert!(
            !popup_owner
                .borrow()
                .as_ref()
                .unwrap()
                .read_with(cx, |popup, _| popup.is_open())
        );
    }
}
